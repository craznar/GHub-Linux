//! Physical input safety state, separate from the platform-neutral macro engine.
#[cfg(target_os = "linux")]
pub mod capture;
use std::collections::BTreeSet;

pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_RIGHTCTRL: u16 = 97;
pub const KEY_LEFTALT: u16 = 56;
pub const KEY_RIGHTALT: u16 = 100;
pub const KEY_BACKSPACE: u16 = 14;

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Forward,
    IgnoreRepeat,
    SuppressUntilRelease,
    EmergencyBypass,
}

/// Receives original physical state before any assignment/recording logic.
#[derive(Default)]
pub struct PhysicalState {
    down: BTreeSet<u16>,
    suppressed: BTreeSet<u16>,
    pub bypass_latched: bool,
}
impl PhysicalState {
    pub fn synchronize(&mut self, currently_down: BTreeSet<u16>) {
        // Reattach/SYN_DROPPED never invents triggers for keys already down.
        self.suppressed = currently_down.clone();
        self.down = currently_down;
    }
    pub fn event(&mut self, code: u16, value: i32) -> Decision {
        if value == 2 {
            return Decision::IgnoreRepeat;
        }
        if value == 0 {
            self.down.remove(&code);
            if self.suppressed.remove(&code) {
                return Decision::SuppressUntilRelease;
            }
            return Decision::Forward;
        }
        if value != 1 || !self.down.insert(code) {
            return Decision::IgnoreRepeat;
        }
        let ctrl = self.down.contains(&KEY_LEFTCTRL) || self.down.contains(&KEY_RIGHTCTRL);
        let alt = self.down.contains(&KEY_LEFTALT) || self.down.contains(&KEY_RIGHTALT);
        if ctrl && alt && self.down.contains(&KEY_BACKSPACE) {
            self.bypass_latched = true;
            self.suppressed.extend(self.down.iter().filter(|c| {
                matches!(
                    **c,
                    KEY_LEFTCTRL | KEY_RIGHTCTRL | KEY_LEFTALT | KEY_RIGHTALT | KEY_BACKSPACE
                )
            }));
            return Decision::EmergencyBypass;
        }
        if self.suppressed.contains(&code) {
            Decision::SuppressUntilRelease
        } else {
            Decision::Forward
        }
    }
    pub fn resume(&mut self) {
        self.bypass_latched = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emergency_is_physical_and_latched() {
        let mut state = PhysicalState::default();
        state.event(KEY_LEFTCTRL, 1);
        state.event(KEY_RIGHTALT, 1);
        assert_eq!(state.event(KEY_BACKSPACE, 1), Decision::EmergencyBypass);
        assert_eq!(
            state.event(KEY_BACKSPACE, 0),
            Decision::SuppressUntilRelease
        );
        assert!(state.bypass_latched);
        state.resume();
        assert!(!state.bypass_latched);
    }
    #[test]
    fn resync_and_repeat_do_not_retrigger() {
        let mut state = PhysicalState::default();
        state.synchronize(BTreeSet::from([30]));
        assert_eq!(state.event(30, 1), Decision::IgnoreRepeat);
        assert_eq!(state.event(30, 2), Decision::IgnoreRepeat);
        assert_eq!(state.event(30, 0), Decision::SuppressUntilRelease);
        assert_eq!(state.event(30, 1), Decision::Forward);
    }
}
