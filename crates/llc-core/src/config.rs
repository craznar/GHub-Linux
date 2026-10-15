use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    G502Hero,
    G815,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceSelector {
    pub model: Model,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub usb_port: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub revision: u64,
    pub login_activation: bool,
    pub devices: Vec<DeviceSelector>,
    pub profiles: Vec<Profile>,
    pub macros: Vec<Macro>,
    pub selected_profile: Uuid,
}
impl Default for Config {
    fn default() -> Self {
        let profile = Profile::new("Default");
        Self {
            schema_version: 1,
            revision: 0,
            login_activation: false,
            devices: vec![],
            selected_profile: profile.id,
            profiles: vec![profile],
            macros: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub default_bank: u8,
    pub bindings: Vec<Binding>,
}
impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            default_bank: 1,
            bindings: vec![],
        }
    }
    pub fn resolve(&self, control: &str, bank: u8, secondary: bool) -> Action {
        let find = |layer| {
            self.bindings
                .iter()
                .find(|b| {
                    b.control == control && b.secondary == layer && b.bank.is_none_or(|n| n == bank)
                })
                .map(|b| b.action.clone())
        };
        if secondary {
            if let Some(action) = find(true) {
                return action;
            }
        }
        find(false).unwrap_or(Action::PassThrough)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// Semantic identity, e.g. g815.key.29, g815.g1, g502.back.
    pub control: String,
    pub secondary: bool,
    pub bank: Option<u8>,
    pub impulse: bool,
    pub action: Action,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(
    tag = "kind",
    content = "code",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Output {
    Key(u16),
    Button(u16),
}
impl Output {
    pub fn validate(self) -> Result<()> {
        match self {
            Self::Key(n) if (1..=255).contains(&n) || (352..=767).contains(&n) => Ok(()),
            Self::Button(n) if (256..=351).contains(&n) => Ok(()),
            _ => invalid("output must be a valid physical Linux key/button code"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    PassThrough,
    Disabled,
    Hold {
        outputs: Vec<Output>,
    },
    Tap {
        outputs: Vec<Output>,
        hold_ms: u64,
    },
    Scroll {
        horizontal: i32,
        vertical: i32,
    },
    Macro {
        id: Uuid,
        mode: Execution,
        retrigger: Retrigger,
    },
    LayerHold,
    LayerToggle,
    Bank {
        bank: u8,
    },
    Profile {
        id: Uuid,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Retrigger {
    Ignore,
    Restart,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Execution {
    Once,
    Repeat { count: u32 },
    WhileHeld,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Macro {
    pub id: Uuid,
    pub name: String,
    pub interval_ms: u64,
    pub operations: Vec<Operation>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Down { output: Output },
    Up { output: Output },
    Tap { outputs: Vec<Output>, hold_ms: u64 },
    Wait { ms: u64 },
    Scroll { horizontal: i32, vertical: i32 },
    Repeat { count: u32, body: Vec<Operation> },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Down(Output),
    Up(Output),
    Wait(u64),
    Scroll(i32, i32),
}
#[derive(Debug, Clone)]
pub struct CompiledMacro {
    pub steps: Vec<Step>,
    pub duration_ms: u64,
    pub interval_ms: u64,
}

fn name(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        invalid("name must contain 1–128 bytes without control characters")
    } else {
        Ok(())
    }
}
fn delay(ms: u64) -> Result<()> {
    if (1..=MAX_DELAY_MS).contains(&ms) {
        Ok(())
    } else {
        invalid("delay must be 1–600000 ms")
    }
}
fn outputs(values: &[Output]) -> Result<()> {
    if values.is_empty() || values.len() > 32 {
        return invalid("an action must contain 1–32 outputs");
    }
    let mut seen = BTreeSet::new();
    for value in values {
        value.validate()?;
        if !seen.insert(*value) {
            return invalid("duplicate output in chord");
        }
    }
    Ok(())
}
fn scroll(h: i32, v: i32) -> Result<()> {
    if h.unsigned_abs() > 120 || v.unsigned_abs() > 120 || (h == 0 && v == 0) {
        invalid("scroll must contain nonzero steps in range -120..120")
    } else {
        Ok(())
    }
}

impl Macro {
    pub fn compile(&self) -> Result<CompiledMacro> {
        name(&self.name)?;
        if !(MIN_INTERVAL_MS..=MAX_DELAY_MS).contains(&self.interval_ms) {
            return invalid("macro interval must be 10–600000 ms");
        }
        let mut stored = 0;
        fn count(ops: &[Operation], depth: usize, total: &mut usize) -> Result<()> {
            if depth > MAX_DEPTH {
                return invalid("repeat nesting exceeds 4");
            }
            for op in ops {
                *total += 1;
                if *total > MAX_STORED_OPS {
                    return invalid("too many stored macro operations");
                }
                if let Operation::Repeat { count: n, body } = op {
                    if *n == 0 || *n > MAX_REPEAT || body.is_empty() {
                        return invalid("invalid finite repeat");
                    }
                    count(body, depth + 1, total)?;
                }
            }
            Ok(())
        }
        count(&self.operations, 0, &mut stored)?;
        if stored == 0 {
            return invalid("empty macro");
        }
        let mut steps = vec![];
        fn expand(ops: &[Operation], steps: &mut Vec<Step>) -> Result<()> {
            for op in ops {
                match op {
                    Operation::Down { output } => {
                        output.validate()?;
                        steps.push(Step::Down(*output));
                    }
                    Operation::Up { output } => {
                        output.validate()?;
                        steps.push(Step::Up(*output));
                    }
                    Operation::Wait { ms } => {
                        delay(*ms)?;
                        steps.push(Step::Wait(*ms));
                    }
                    Operation::Scroll {
                        horizontal,
                        vertical,
                    } => {
                        scroll(*horizontal, *vertical)?;
                        steps.push(Step::Scroll(*horizontal, *vertical));
                    }
                    Operation::Tap {
                        outputs: keys,
                        hold_ms,
                    } => {
                        outputs(keys)?;
                        delay(*hold_ms)?;
                        steps.extend(keys.iter().map(|k| Step::Down(*k)));
                        steps.push(Step::Wait(*hold_ms));
                        steps.extend(keys.iter().rev().map(|k| Step::Up(*k)));
                    }
                    Operation::Repeat { count, body } => {
                        for _ in 0..*count {
                            expand(body, steps)?;
                        }
                    }
                }
                if steps.len() > MAX_EXPANDED_OPS {
                    return invalid("expanded macro exceeds 100000 operations");
                }
            }
            Ok(())
        }
        expand(&self.operations, &mut steps)?;
        let mut held = BTreeSet::new();
        let mut duration_ms = 0_u64;
        for step in &steps {
            match *step {
                Step::Down(k) if !held.insert(k) => {
                    return invalid("duplicate owned press in macro")
                }
                Step::Up(k) if !held.remove(&k) => {
                    return invalid("release without an owned press")
                }
                Step::Wait(ms) => {
                    duration_ms = duration_ms
                        .checked_add(ms)
                        .ok_or_else(|| Error::Invalid("duration overflow".into()))?;
                    if duration_ms > MAX_DURATION_MS {
                        return invalid("macro exceeds 60 minutes");
                    }
                }
                _ => {}
            }
        }
        if !held.is_empty() {
            return invalid("macro iteration ends with held output");
        }
        Ok(CompiledMacro {
            steps,
            duration_ms,
            interval_ms: self.interval_ms,
        })
    }
}

impl Config {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return invalid("configuration exceeds 4 MiB");
        }
        let config: Self = serde_json::from_slice(bytes)?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return invalid("unsupported schema version; source has not been rewritten");
        }
        if self.profiles.is_empty()
            || self.profiles.len() > MAX_PROFILES
            || self.macros.len() > MAX_MACROS
            || self.devices.len() > 2
        {
            return invalid("configuration entity count exceeds limits");
        }
        let mut selectors = BTreeSet::new();
        for selector in &self.devices {
            if !selectors.insert(&selector.model) {
                return invalid("duplicate model selector");
            }
            for value in [&selector.serial, &selector.usb_port].into_iter().flatten() {
                name(value)?;
            }
        }
        let mut ids = BTreeSet::new();
        let mut compiled = BTreeMap::new();
        for value in &self.macros {
            if value.id.is_nil() || !ids.insert(value.id) {
                return invalid("duplicate or nil entity UUID");
            }
            compiled.insert(value.id, value.compile()?);
        }
        let profile_ids: BTreeSet<_> = self.profiles.iter().map(|p| p.id).collect();
        if !profile_ids.contains(&self.selected_profile) {
            return invalid("selected profile does not exist");
        }
        for profile in &self.profiles {
            name(&profile.name)?;
            if profile.id.is_nil() || !ids.insert(profile.id) {
                return invalid("duplicate or nil entity UUID");
            }
            if !(1..=3).contains(&profile.default_bank) || profile.bindings.len() > 4096 {
                return invalid("invalid default bank or binding count");
            }
            let mut controls = BTreeSet::new();
            for binding in &profile.bindings {
                let c = binding.control.as_str();
                let gkey = matches!(c, "g815.g1" | "g815.g2" | "g815.g3" | "g815.g4" | "g815.g5");
                let valid_key = c
                    .strip_prefix("g815.key.")
                    .and_then(|s| s.parse::<u16>().ok())
                    .is_some_and(|k| Output::Key(k).validate().is_ok());
                let valid_mouse = matches!(
                    c,
                    "g502.left"
                        | "g502.right"
                        | "g502.middle"
                        | "g502.back"
                        | "g502.forward"
                        | "g502.dpi_shift"
                        | "g502.dpi_up"
                        | "g502.dpi_down"
                        | "g502.profile"
                        | "g502.tilt_left"
                        | "g502.tilt_right"
                        | "g502.wheel_up"
                        | "g502.wheel_down"
                );
                if !(gkey || valid_key || valid_mouse) {
                    return invalid("unknown semantic control identity");
                }
                if gkey != binding.bank.is_some()
                    || binding.bank.is_some_and(|n| !(1..=3).contains(&n))
                {
                    return invalid("only G1–G5 require a bank in range 1–3");
                }
                if !controls.insert((&binding.control, binding.secondary, binding.bank)) {
                    return invalid("duplicate binding");
                }
                if c.contains("wheel_") && !binding.impulse {
                    return invalid("wheel steps must be impulse controls");
                }
                match &binding.action {
                    Action::Hold { outputs: keys } => {
                        outputs(keys)?;
                        if binding.impulse {
                            return invalid("impulse cannot hold output");
                        }
                    }
                    Action::Tap {
                        outputs: keys,
                        hold_ms,
                    } => {
                        outputs(keys)?;
                        delay(*hold_ms)?;
                    }
                    Action::Scroll {
                        horizontal,
                        vertical,
                    } => scroll(*horizontal, *vertical)?,
                    Action::LayerHold if binding.impulse => {
                        return invalid("impulse cannot hold a layer")
                    }
                    Action::Bank { bank } if !(1..=3).contains(bank) => {
                        return invalid("bank must be 1–3")
                    }
                    Action::Profile { id } if !profile_ids.contains(id) => {
                        return invalid("profile reference not found")
                    }
                    Action::Macro { id, mode, .. } => {
                        let value = compiled
                            .get(id)
                            .ok_or_else(|| Error::Invalid("macro reference not found".into()))?;
                        if binding.impulse && !matches!(mode, Execution::Once | Execution::Toggle) {
                            return invalid("impulse supports once/toggle macros only");
                        }
                        if let Execution::Repeat { count } = mode {
                            if *count == 0
                                || *count > MAX_REPEAT
                                || value.steps.len().saturating_mul(*count as usize)
                                    > MAX_EXPANDED_OPS
                                || value
                                    .duration_ms
                                    .saturating_mul(*count as u64)
                                    .saturating_add(
                                        value
                                            .interval_ms
                                            .saturating_mul(count.saturating_sub(1) as u64),
                                    )
                                    > MAX_DURATION_MS
                            {
                                return invalid("Repeat N exceeds execution limits");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn redacted_export(&self) -> Self {
        let mut result = self.clone();
        for selector in &mut result.devices {
            selector.serial = None;
            selector.usb_port = None;
        }
        result
    }
}
