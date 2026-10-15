use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Down { output: Output },
    Up { output: Output },
    Scroll { horizontal: i32, vertical: i32 },
}

/// Aggregate ownership prevents one source from releasing another source's key.
#[derive(Default)]
pub struct Ledger {
    holds: BTreeMap<Output, BTreeSet<String>>,
    order: BTreeMap<String, Vec<Output>>,
}
impl Ledger {
    pub fn down(&mut self, owner: &str, output: Output) -> Option<Event> {
        let owners = self.holds.entry(output).or_default();
        let was_empty = owners.is_empty();
        let inserted = owners.insert(owner.to_owned());
        if inserted {
            self.order.entry(owner.into()).or_default().push(output);
        }
        if inserted && was_empty {
            Some(Event::Down { output })
        } else {
            None
        }
    }
    pub fn up(&mut self, owner: &str, output: Output) -> Option<Event> {
        let owners = self.holds.get_mut(&output)?;
        let removed = owners.remove(owner);
        if removed {
            if let Some(order) = self.order.get_mut(owner) {
                order.retain(|k| *k != output);
                if order.is_empty() {
                    self.order.remove(owner);
                }
            }
        }
        if removed && owners.is_empty() {
            self.holds.remove(&output);
            Some(Event::Up { output })
        } else {
            None
        }
    }
    pub fn release(&mut self, owner: &str) -> Vec<Event> {
        let keys = self.order.get(owner).cloned().unwrap_or_default();
        keys.into_iter()
            .rev()
            .filter_map(|key| self.up(owner, key))
            .collect()
    }
    pub fn held_count(&self) -> usize {
        self.holds.len()
    }
}
struct Instance {
    source: String,
    owner: String,
    program: CompiledMacro,
    cursor: usize,
    deadline: u64,
    remaining: Option<u32>,
    mode: Execution,
    restart: Retrigger,
}

pub struct Scheduler {
    instances: BTreeMap<u64, Instance>,
    next_id: u64,
    recent: VecDeque<u64>,
    last_time: u64,
    pub ledger: Ledger,
}
impl Default for Scheduler {
    fn default() -> Self {
        Self {
            instances: BTreeMap::new(),
            next_id: 1,
            recent: VecDeque::new(),
            last_time: 0,
            ledger: Ledger::default(),
        }
    }
}
impl Scheduler {
    pub fn active_count(&self) -> usize {
        self.instances.len()
    }
    pub fn next_deadline(&self) -> Option<u64> {
        self.instances.values().map(|i| i.deadline).min()
    }
    pub fn start(
        &mut self,
        source: &str,
        program: CompiledMacro,
        mode: Execution,
        retrigger: Retrigger,
        now: u64,
    ) -> Result<Vec<Event>> {
        if source.len() > 256 || now < self.last_time {
            return invalid("invalid source or non-monotonic clock");
        }
        self.last_time = now;
        if let Execution::Repeat { count } = mode {
            if count == 0
                || count > MAX_REPEAT
                || program.steps.len().saturating_mul(count as usize) > MAX_EXPANDED_OPS
                || program
                    .duration_ms
                    .saturating_mul(count as u64)
                    .saturating_add(
                        program
                            .interval_ms
                            .saturating_mul(count.saturating_sub(1) as u64),
                    )
                    > MAX_DURATION_MS
            {
                return invalid("Repeat N exceeds execution limits");
            }
        }
        let existing: Vec<_> = self
            .instances
            .iter()
            .filter(|(_, i)| i.source == source)
            .map(|(id, _)| *id)
            .collect();
        let mut events = vec![];
        if let Some(id) = existing.first() {
            let running = &self.instances[id];
            if matches!(running.mode, Execution::Toggle) {
                return Ok(self.cancel(*id));
            }
            if running.restart == Retrigger::Ignore {
                return Ok(events);
            }
            for id in existing {
                events.extend(self.cancel(id));
            }
        }
        if self.instances.len() >= MAX_INSTANCES {
            return invalid("32 simultaneous macro instances already running");
        }
        let remaining = match mode {
            Execution::Once => Some(1),
            Execution::Repeat { count } => Some(count),
            _ => None,
        };
        if remaining == Some(0) {
            return invalid("zero repeat count");
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| Error::Invalid("instance counter exhausted".into()))?;
        self.instances.insert(
            id,
            Instance {
                source: source.into(),
                owner: format!("macro:{id}"),
                program,
                cursor: 0,
                deadline: now,
                remaining,
                mode,
                restart: retrigger,
            },
        );
        Ok(events)
    }
    fn cancel(&mut self, id: u64) -> Vec<Event> {
        self.instances
            .remove(&id)
            .map_or_else(Vec::new, |i| self.ledger.release(&i.owner))
    }
    pub fn cancel_source(&mut self, source: &str, only_while_held: bool) -> Vec<Event> {
        let ids: Vec<_> = self
            .instances
            .iter()
            .filter(|(_, i)| {
                i.source == source && (!only_while_held || matches!(i.mode, Execution::WhileHeld))
            })
            .map(|(id, _)| *id)
            .collect();
        ids.into_iter().flat_map(|id| self.cancel(id)).collect()
    }
    pub fn cancel_all(&mut self) -> Vec<Event> {
        let ids: Vec<_> = self.instances.keys().copied().collect();
        ids.into_iter().flat_map(|id| self.cancel(id)).collect()
    }
    /// Caller supplies elapsed monotonic milliseconds. Work and output are bounded.
    pub fn advance(&mut self, now: u64) -> Result<(Vec<Event>, Vec<String>)> {
        if now < self.last_time {
            return invalid("non-monotonic clock");
        }
        self.last_time = now;
        while self
            .recent
            .front()
            .is_some_and(|t| now.saturating_sub(*t) >= 1000)
        {
            self.recent.pop_front();
        }
        let mut events = vec![];
        let mut faults = vec![];
        let mut budget = MAX_EXPANDED_OPS;
        while budget > 0 {
            let next = self
                .instances
                .iter()
                .filter(|(_, i)| i.deadline <= now)
                .min_by_key(|(id, i)| (i.deadline, **id))
                .map(|(id, _)| *id);
            let Some(id) = next else {
                break;
            };
            budget -= 1;
            let instance = self.instances.get_mut(&id).unwrap();
            if instance.cursor == instance.program.steps.len() {
                let done = match instance.remaining.as_mut() {
                    Some(n) => {
                        *n -= 1;
                        *n == 0
                    }
                    None => false,
                };
                if done {
                    events.extend(self.cancel(id));
                } else {
                    instance.cursor = 0;
                    instance.deadline = now.saturating_add(instance.program.interval_ms);
                }
                continue;
            }
            let step = instance.program.steps[instance.cursor].clone();
            instance.cursor += 1;
            if let Step::Wait(ms) = step {
                instance.deadline = now.saturating_add(ms);
                continue;
            }
            // Apply rate limits to logical generated events, including shared holds.
            if self.recent.len() >= MAX_EVENTS_PER_SECOND {
                faults.push(format!(
                    "macro instance {id} cancelled: event quota exceeded"
                ));
                events.extend(self.cancel(id));
                continue;
            }
            self.recent.push_back(now);
            let event = match step {
                Step::Down(output) => self.ledger.down(&instance.owner, output),
                Step::Up(output) => self.ledger.up(&instance.owner, output),
                Step::Scroll(horizontal, vertical) => Some(Event::Scroll {
                    horizontal,
                    vertical,
                }),
                Step::Wait(_) => unreachable!(),
            };
            if let Some(event) = event {
                events.push(event);
            }
        }
        if budget == 0 {
            faults.push("scheduler work budget reached".into());
            events.extend(self.cancel_all());
        }
        Ok((events, faults))
    }
}

#[derive(Default)]
struct Layers {
    holds: BTreeSet<String>,
    toggle: bool,
}
impl Layers {
    fn active(&self) -> bool {
        self.toggle || !self.holds.is_empty()
    }
}
struct Captured {
    action: Action,
    device: String,
    physical: bool,
}
pub struct Engine {
    config: Config,
    profile: Uuid,
    bank: u8,
    layers: Layers,
    captured: BTreeMap<String, Captured>,
    suppressed: BTreeSet<String>,
    pub scheduler: Scheduler,
    pub bypass: bool,
}
impl Engine {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        let profile = config.selected_profile;
        let bank = config
            .profiles
            .iter()
            .find(|p| p.id == profile)
            .unwrap()
            .default_bank;
        Ok(Self {
            config,
            profile,
            bank,
            layers: Layers::default(),
            captured: BTreeMap::new(),
            suppressed: BTreeSet::new(),
            scheduler: Scheduler::default(),
            bypass: false,
        })
    }
    pub fn bank(&self) -> u8 {
        self.bank
    }
    pub fn secondary(&self) -> bool {
        self.layers.active()
    }
    pub fn select_bank(&mut self, bank: u8) -> Result<()> {
        if !(1..=3).contains(&bank) {
            return invalid("bank must be 1–3");
        }
        self.bank = bank;
        Ok(())
    }
    pub fn press(
        &mut self,
        device: &str,
        control: &str,
        native: Option<Output>,
        repeat: bool,
        now: u64,
    ) -> Result<Vec<Event>> {
        let source = format!("{device}/{control}");
        if source.len() > 256 || self.captured.len() >= 2048 {
            return invalid("source identity or held input limit");
        }
        if repeat || self.captured.contains_key(&source) || self.suppressed.contains(&source) {
            return Ok(vec![]);
        }
        let action = if self.bypass {
            Action::PassThrough
        } else {
            self.config
                .profiles
                .iter()
                .find(|p| p.id == self.profile)
                .unwrap()
                .resolve(control, self.bank, self.layers.active())
        };
        let mut events = vec![];
        let mut captured = action.clone();
        let physical = action == Action::PassThrough;
        match action {
            Action::PassThrough => {
                if let Some(output) = native {
                    captured = Action::Hold {
                        outputs: vec![output],
                    };
                    if let Some(e) = self.scheduler.ledger.down(&source, output) {
                        events.push(e);
                    }
                }
            }
            Action::Disabled => {}
            Action::Hold { outputs } => {
                for output in outputs {
                    if let Some(e) = self.scheduler.ledger.down(&source, output) {
                        events.push(e);
                    }
                }
            }
            Action::Tap { outputs, hold_ms } => {
                let program = Macro {
                    id: Uuid::nil(),
                    name: "binding tap".into(),
                    interval_ms: 10,
                    operations: vec![Operation::Tap { outputs, hold_ms }],
                }
                .compile()?;
                events.extend(self.scheduler.start(
                    &source,
                    program,
                    Execution::Once,
                    Retrigger::Ignore,
                    now,
                )?);
            }
            Action::Scroll {
                horizontal,
                vertical,
            } => events.push(Event::Scroll {
                horizontal,
                vertical,
            }),
            Action::Macro {
                id,
                mode,
                retrigger,
            } => {
                let program = self
                    .config
                    .macros
                    .iter()
                    .find(|m| m.id == id)
                    .unwrap()
                    .compile()?;
                events.extend(
                    self.scheduler
                        .start(&source, program, mode, retrigger, now)?,
                );
            }
            Action::LayerHold => {
                self.layers.holds.insert(source.clone());
            }
            Action::LayerToggle => {
                self.layers.toggle = !self.layers.toggle;
            }
            Action::Bank { bank } => self.bank = bank,
            Action::Profile { id } => events.extend(self.select_profile(id)?),
        }
        self.captured.insert(
            source,
            Captured {
                action: captured,
                device: device.into(),
                physical,
            },
        );
        Ok(events)
    }
    pub fn release(&mut self, device: &str, control: &str) -> Vec<Event> {
        let source = format!("{device}/{control}");
        self.suppressed.remove(&source);
        self.layers.holds.remove(&source);
        let mut events = self.scheduler.cancel_source(&source, true);
        if let Some(captured) = self.captured.remove(&source) {
            if let Action::Hold { outputs } = captured.action {
                for output in outputs.into_iter().rev() {
                    if let Some(e) = self.scheduler.ledger.up(&source, output) {
                        events.push(e);
                    }
                }
            }
        }
        events
    }
    pub fn select_profile(&mut self, id: Uuid) -> Result<Vec<Event>> {
        let bank = self
            .config
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::Invalid("profile not found".into()))?
            .default_bank;
        let events = self.cancel_software();
        self.profile = id;
        self.bank = bank;
        Ok(events)
    }
    fn cancel_software(&mut self) -> Vec<Event> {
        let mut events = self.scheduler.cancel_all();
        self.layers = Layers::default();
        let sources: Vec<_> = self
            .captured
            .iter()
            .filter(|(_, c)| !c.physical)
            .map(|(s, _)| s.clone())
            .collect();
        for source in sources {
            events.extend(self.scheduler.ledger.release(&source));
            self.suppressed.insert(source.clone());
            self.captured.remove(&source);
        }
        events
    }
    pub fn set_bypass(&mut self, enabled: bool) -> Vec<Event> {
        let events = self.cancel_software();
        self.bypass = enabled;
        events
    }
    pub fn disconnect(&mut self, device: &str) -> Vec<Event> {
        let sources: Vec<_> = self
            .captured
            .iter()
            .filter(|(_, c)| c.device == device)
            .map(|(s, _)| s.clone())
            .collect();
        let mut events = vec![];
        for source in sources {
            events.extend(self.scheduler.cancel_source(&source, false));
            events.extend(self.scheduler.ledger.release(&source));
            self.layers.holds.remove(&source);
            self.captured.remove(&source);
            self.suppressed.remove(&source);
        }
        // Once/toggle macros may outlive the source's release.
        let sources: BTreeSet<_> = self
            .scheduler
            .instances
            .values()
            .filter(|i| i.source.starts_with(&format!("{device}/")))
            .map(|i| i.source.clone())
            .collect();
        for source in sources {
            events.extend(self.scheduler.cancel_source(&source, false));
        }
        self.suppressed
            .retain(|s| !s.starts_with(&format!("{device}/")));
        events
    }
}
