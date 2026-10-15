use llc_core::{persistence, *};
use llc_service::Client;
use serde_json::{json, Value};
use std::{fs, path::Path};

const HELP: &str = "Linux Logitech Control 0.1.0 — engineering build, hardware gate G0 unverified

llc status [--json]
llc devices [--json]
llc capabilities <g502_hero|g815> [--json]
llc probe [--json]                      Read-only USB/interface inventory
llc probe events <g502_hero|g815> <seconds> <new-file>
llc macro simulate <config-file> <macro-uuid> [--json]
llc config init                        Create passive default configuration
llc config path
llc config validate <file> [--json]
llc config import <file>                Validate and save with revision check
llc config export <file>                Excludes serials and port identity
llc config schema                      Print JSON Schema
llc profiles list [--json]
llc profile create|rename|duplicate|delete <name-or-uuid> [name]
llc profile activate <uuid>             Saves selection, then requests activation
llc bank select <1|2|3>
llc login enable|disable                Opt-in activation setting
llc management enable|disable
llc bypass | resume
llc diagnostics export <file>

Exit codes: 0 success, 2 invalid input, 3 authorization denied,
4 service unavailable, 5 device unavailable/unverified, 6 failure/conflict.
No command enables unqualified device routes or performs persistent HID writes.";

#[derive(Debug)]
struct Failure {
    code: i32,
    message: String,
}
type Result<T> = std::result::Result<T, Failure>;
fn fail<T>(code: i32, message: impl Into<String>) -> Result<T> {
    Err(Failure {
        code,
        message: message.into(),
    })
}
fn core<T>(result: llc_core::Result<T>) -> Result<T> {
    result.map_err(|e| Failure {
        code: if matches!(e, Error::Revision(_) | Error::Io(_)) {
            6
        } else {
            2
        },
        message: e.to_string(),
    })
}
fn bus<T>(result: std::result::Result<T, impl std::fmt::Display>) -> Result<T> {
    result.map_err(|e| {
        let message = e.to_string();
        let code = if message.contains("AuthorizationDenied") || message.contains("AccessDenied") {
            3
        } else if message.contains("DeviceUnavailable") {
            5
        } else if message.contains("InvalidInput") {
            2
        } else if message.contains("Conflict") {
            6
        } else {
            4
        };
        Failure { code, message }
    })
}
fn value<T: serde_json::value::Index>(v: &Value, key: T) -> String {
    v[key].as_str().unwrap_or("").to_owned()
}
fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}
fn load() -> Result<Config> {
    let path = core(persistence::config_path())?;
    let (config, backup) = core(persistence::recover(&path))?;
    if backup {
        return fail(6, "primary configuration is corrupt; backup exists and was left untouched; restore it explicitly before editing");
    }
    Ok(config)
}
fn save(config: &Config) -> Result<Config> {
    core(persistence::save(
        &core(persistence::config_path())?,
        config,
        config.revision,
    ))
}
fn client() -> Result<Client> {
    bus(Client::connect())
}
fn uuid(text: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(text).map_err(|_| Failure {
        code: 2,
        message: "invalid UUID".into(),
    })
}

fn run(args: &[String], json_output: bool) -> Result<()> {
    let a: Vec<_> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        [] | ["--help"] | ["help"] | ["-h"] => println!("{HELP}"),
        ["--version"] => println!("llc {}", env!("CARGO_PKG_VERSION")),
        ["probe", "events", model, seconds, output] => {
            let product = match *model {
                "g502_hero" => 0xc08b,
                "g815" => 0xc33f,
                _ => return fail(2, "model must be g502_hero or g815"),
            };
            let seconds: u64 = seconds.parse().map_err(|_| Failure {
                code: 2,
                message: "seconds must be 1–30".into(),
            })?;
            if !(1..=30).contains(&seconds) {
                return fail(2, "seconds must be 1–30");
            }
            if Path::new(output).exists() {
                return fail(6, "output already exists; no capture started");
            }
            let report = llc_protocol::probe().map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
            let devices: Vec<_> = report
                .devices
                .iter()
                .filter(|d| d.product_id == format!("{product:04x}"))
                .collect();
            if devices.len() != 1 {
                return fail(
                    5,
                    "selected model is absent or ambiguous; attach exactly one selected model",
                );
            }
            let paths: Vec<_> = devices[0]
                .input_nodes
                .iter()
                .map(|n| n.path.clone())
                .collect();
            eprintln!("Explicit read-only capture for {seconds} seconds. Physical events still reach applications; capture contains key/button activity. No grabs or writes.");
            #[cfg(target_os = "linux")]
            {
                let capture =
                    llc_input::capture::capture(&paths, product, seconds).map_err(|e| Failure {
                        code: 5,
                        message: e.to_string(),
                    })?;
                llc_input::capture::write_new(
                    Path::new(output),
                    &serde_json::to_vec_pretty(&capture).unwrap(),
                )
                .map_err(|e| Failure {
                    code: 6,
                    message: e.to_string(),
                })?;
            }
            #[cfg(not(target_os = "linux"))]
            return fail(5, "event capture requires Linux");
        }
        ["macro", "simulate", path, id] => {
            let config = core(persistence::read(Path::new(path)))?;
            let id = uuid(id)?;
            let program = core(
                config
                    .macros
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| Failure {
                        code: 2,
                        message: "macro not found".into(),
                    })?
                    .compile(),
            )?;
            let mut scheduler = llc_core::engine::Scheduler::default();
            core(scheduler.start(
                "offline-simulation",
                program,
                Execution::Once,
                Retrigger::Ignore,
                0,
            ))?;
            let mut trace = vec![];
            while let Some(now) = scheduler.next_deadline() {
                let (events, faults) = core(scheduler.advance(now))?;
                for event in events {
                    trace.push(json!({"at_ms":now,"event":event}));
                }
                if !faults.is_empty() {
                    return fail(6, faults.join("; "));
                }
            }
            print_json(
                &json!({"schema_version":1,"simulated":true,"physical_output":false,"events":trace}),
            );
        }
        ["probe", "--allow-transient-changes"] => return fail(
            5,
            "transient changes are not implemented before hardware qualification; run llc probe",
        ),
        ["probe"] => {
            let report = llc_protocol::probe().map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
            let report = serde_json::to_value(report).unwrap();
            if json_output {
                print_json(&report);
            } else {
                println!("Read-only inventory; hardware gate G0 unverified");
                for device in report["devices"].as_array().unwrap() {
                    println!(
                        "{}  {}:{}  USB {}  {} input, {} hidraw interfaces",
                        value(device, "model"),
                        value(device, "vendor_id"),
                        value(device, "product_id"),
                        value(device, "usb_port"),
                        device["input_nodes"].as_array().unwrap().len(),
                        device["hidraw_nodes"].as_array().unwrap().len()
                    );
                }
                if report["devices"].as_array().unwrap().is_empty() {
                    println!("No wired G502 HERO or G815 found.");
                }
            }
        }
        ["status"] | ["devices"] => {
            let method = if a[0] == "status" {
                "Status"
            } else {
                "Devices"
            };
            let output = bus(client()?.json(method))?;
            let parsed: Value = serde_json::from_str(&output).map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
            if !json_output && a[0] == "status" {
                println!(
                    "{}; management disabled; {}",
                    value(&parsed, "state"),
                    value(&parsed, "hardware_gate")
                );
            } else {
                print_json(&parsed);
            }
        }
        ["capabilities", model] => {
            if !matches!(*model, "g502_hero" | "g815") {
                return fail(2, "model must be g502_hero or g815");
            }
            let report = llc_protocol::probe().map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
            let present = report
                .devices
                .iter()
                .any(|d| serde_json::to_value(&d.model).unwrap() == *model);
            print_json(
                &json!({"schema_version":1,"model":model,"present":present,"capabilities":{"input_routes":"untested","dpi":"untested","polling_rate":"untested","lighting":"untested"},"hardware_gate":"G0-unverified"}),
            );
        }
        ["config", "path"] => println!("{}", core(persistence::config_path())?.display()),
        ["config", "init"] => {
            let path = core(persistence::config_path())?;
            if path.exists() {
                return fail(6, "configuration already exists; no files changed");
            }
            let config = core(persistence::save(&path, &Config::default(), 0))?;
            print_json(&json!({"schema_version":1,"saved_revision":config.revision}));
        }
        ["config", "schema"] => {
            print_json(&serde_json::to_value(schemars::schema_for!(Config)).unwrap())
        }
        ["config", "validate", path] => {
            let config = core(persistence::read(Path::new(path)))?;
            print_json(
                &json!({"schema_version":1,"valid":true,"revision":config.revision,"profiles":config.profiles.len(),"macros":config.macros.len()}),
            );
        }
        ["config", "import", path] => {
            let mut imported = core(persistence::read(Path::new(path)))?;
            let destination = core(persistence::config_path())?;
            let expected = if destination.exists() {
                load()?.revision
            } else {
                0
            };
            imported.revision = expected;
            let saved = core(persistence::save(&destination, &imported, expected))?;
            print_json(
                &json!({"schema_version":1,"saved_revision":saved.revision,"active_revision_changed":false}),
            );
        }
        ["config", "export", path] => {
            core(persistence::export(Path::new(path), &load()?))?;
        }
        ["profiles", "list"] => {
            let config = load()?;
            print_json(
                &json!({"schema_version":1,"revision":config.revision,"selected_profile":config.selected_profile,"profiles":config.profiles.iter().map(|p| json!({"id":p.id,"name":p.name,"default_bank":p.default_bank})).collect::<Vec<_>>()}),
            );
        }
        ["profile", "create", name] => {
            let mut config = load()?;
            let profile = Profile::new(name);
            let id = profile.id;
            config.profiles.push(profile);
            save(&config)?;
            print_json(&json!({"schema_version":1,"id":id}));
        }
        ["profile", "rename", id, name] => {
            let mut config = load()?;
            let id = uuid(id)?;
            config
                .profiles
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| Failure {
                    code: 2,
                    message: "profile not found".into(),
                })?
                .name = name.to_string();
            save(&config)?;
        }
        ["profile", "duplicate", id, name] => {
            let mut config = load()?;
            let id = uuid(id)?;
            let mut profile = config
                .profiles
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .ok_or_else(|| Failure {
                    code: 2,
                    message: "profile not found".into(),
                })?;
            profile.id = uuid::Uuid::new_v4();
            profile.name = name.to_string();
            config.profiles.push(profile);
            save(&config)?;
        }
        ["profile", "delete", id] => {
            let mut config = load()?;
            let id = uuid(id)?;
            let count = config.profiles.len();
            config.profiles.retain(|p| p.id != id);
            if count == config.profiles.len() {
                return fail(2, "profile not found");
            }
            if config.selected_profile == id {
                config.selected_profile = config
                    .profiles
                    .first()
                    .ok_or_else(|| Failure {
                        code: 2,
                        message: "cannot delete the last profile".into(),
                    })?
                    .id;
            }
            save(&config)?;
        }
        ["profile", "activate", id] => {
            let mut config = load()?;
            config.selected_profile = uuid(id)?;
            let config = save(&config)?;
            let client = client()?;
            bus(client.call(
                "StageConfiguration",
                &(serde_json::to_string(&config).unwrap(),),
            ))?;
            bus(client.call("SetManagement", &(true,)))?;
        }
        ["bank", "select", bank] => {
            let bank: u8 = bank.parse().map_err(|_| Failure {
                code: 2,
                message: "bank must be 1–3".into(),
            })?;
            if !(1..=3).contains(&bank) {
                return fail(2, "bank must be 1–3");
            }
            bus(client()?.call("SelectBank", &(bank,)))?;
        }
        ["login", setting] if matches!(*setting, "enable" | "disable") => {
            let mut config = load()?;
            config.login_activation = *setting == "enable";
            save(&config)?;
        }
        ["management", setting] if matches!(*setting, "enable" | "disable") => {
            let client = client()?;
            if *setting == "enable" {
                let config = load()?;
                bus(client.call(
                    "StageConfiguration",
                    &(serde_json::to_string(&config).unwrap(),),
                ))?;
            }
            bus(client.call("SetManagement", &(*setting == "enable",)))?;
        }
        ["bypass"] | ["resume"] => {
            bus(client()?.call("SetBypass", &(a[0] == "bypass",)))?;
        }
        ["diagnostics", "export", path] => {
            let output = bus(client()?.json("Diagnostics"))?;
            let mut options = fs::OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            use std::io::Write;
            let mut file = options.open(path).map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
            file.write_all(output.as_bytes()).map_err(|e| Failure {
                code: 6,
                message: e.to_string(),
            })?;
        }
        _ => return fail(2, "unknown command or arguments; run llc --help"),
    }
    Ok(())
}
fn main() {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if let Err(error) = run(&args, json_output) {
        if json_output {
            print_json(
                &json!({"schema_version":1,"error":{"exit_code":error.code,"message":error.message}}),
            );
        } else {
            eprintln!("llc: {}", error.message);
        }
        std::process::exit(error.code);
    }
}
