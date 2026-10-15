use llc_core::persistence;
use llc_service::Client;
use std::{thread, time::Duration};
fn main() {
    if std::env::args().any(|a| a == "--version") {
        println!("llc-session-agent {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let mut applied: Option<(String, u64)> = None;
    let mut previous_error = String::new();
    loop {
        let attempt = (|| -> Result<(), Box<dyn std::error::Error>> {
            let path = persistence::config_path()?;
            if !path.exists() {
                applied = None;
                return Ok(());
            }
            let (config, recovered) = persistence::recover(&path)?;
            if recovered {
                return Err(
                    "configuration needs explicit backup recovery; login activation skipped".into(),
                );
            }
            let client = Client::connect()?;
            let owner = client.owner()?;
            let identity = (owner, config.revision);
            let status: serde_json::Value = serde_json::from_str(&client.json("Status")?)?;
            if applied.as_ref() == Some(&identity)
                && status["staged_revision"].as_u64() == Some(config.revision)
            {
                return Ok(());
            }
            // Always supply validated user data; never ask the privileged service to read a path.
            client.call("StageConfiguration", &(serde_json::to_string(&config)?,))?;
            client.call("SetManagement", &(config.login_activation,))?;
            applied = Some(identity);
            Ok(())
        })();
        match attempt {
            Ok(()) => previous_error.clear(),
            Err(error) => {
                let message = error.to_string();
                if message != previous_error {
                    eprintln!("llc-session-agent: {message}");
                    previous_error = message;
                }
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}
