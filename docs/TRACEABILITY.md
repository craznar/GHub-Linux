# Requirement traceability

Engineering implementation only. No S/M/K requirement is claimed complete on hardware. Paths below are relative to crates/ unless stated otherwise. Core evidence is automated/simulated; all physical qualification remains untested. See TESTING.md.

| ID | Requirement | Implementation | Verification |
|---|---|---|---|
| S-01 | Detect and manage both initial devices, including reconnect and suspend/resume. | Partial: llc-protocol/src/lib.rs inventory only; lifecycle routing blocked | Synthetic sysfs inventory test; actual models absent |
| S-02 | Reassign every electronically programmable G502 HERO control, including wheel tilt and the DPI/profile controls, after verifying its actual input path. | Blocked: hardware control routes absent | Physical behaviour untested |
| S-03 | Reassign G815 G1–G5, with three selectable software banks associated with M1–M3. | Partial: llc-core/src/config.rs and engine.rs bank model; no G-key route | llc-core/tests/core.rs; simulated only |
| S-04 | Reassign ordinary G815 keyboard keys, including modifiers, function keys, navigation keys and numeric keypad keys. | Partial: physical-key configuration model; no live remapping | Physical behaviour untested |
| S-05 | Support keyboard output, mouse-button output, wheel output, disabled controls, pass-through and profile/bank selection. | Partial: core Action/Engine and event simulation; no virtual output | llc-core/tests/core.rs; simulated only |
| S-06 | Provide software macros with explicit presses/releases, delays, sequences, bounded repeat, repeat while held and toggle repeat. | Partial: Macro::compile and Scheduler; physical playback blocked | llc-core/tests/core.rs; simulated only |
| S-07 | Provide reusable macros, profile editing, a macro editor/recorder, manual activation, import/export and a CLI. | Partial: llc-cli, persistence, profile commands and simulation; no editor/recorder | llc-core/tests/core.rs; simulated only |
| S-08 | Support a software modifier layer equivalent in purpose to G-Shift, including a keyboard modifier changing mouse assignments. | Partial: Engine layer ownership and captured release; no hardware route | llc-core/tests/core.rs; simulated only |
| S-09 | Configure G502 DPI stages, active/default stage, temporary DPI shift and supported polling rates where hardware verification establishes these controls. | Not implemented: requires evidence | Physical behaviour untested |
| S-10 | Provide capability-based basic lighting: off, static colour and brightness on verified zones/keys. See section 7 for the required support classification. | Not implemented: requires evidence | Physical behaviour untested |
| S-11 | Keep mappings working while the GUI is closed, with explicit user control over login activation. | Partial: passive service and session agent; no active mappings | Authorization facts and isolated IPC; graphical login unqualified |
| S-12 | Recover safely from disconnect, invalid configuration, service failure and session changes. | Partial: core cancellation, persistence and session supervision; live recovery blocked | llc-core/tests/core.rs; simulated only |
| S-13 | Ship a native Ubuntu package, source, tests, diagnostics and operational documentation. | Partial: build/install/package scripts, passive service, CLI and documentation | Build, isolated IPC and WSL package lifecycle; target release unqualified |
| M-01 | Identify left, right, middle, back, forward, DPI shift, DPI up/down, profile control and left/right wheel tilt using physical labels and observed identities. | Blocked: semantic names reserved but no physical evidence | Physical behaviour untested |
| M-02 | Allow each electronically programmable control to use the common assignment model. Preserve ordinary pointing and scrolling. | Partial: common Action model only | Physical behaviour untested |
| M-03 | Exclude the mechanical wheel free-spin/ratchet switch if it produces no electronic event. Do not count it as an unimplemented programmable button. | Untested: mechanical control not claimed assignable | Physical behaviour untested |
| M-04 | Expose supported DPI bounds, increments and stage counts from evidence; reject invalid values and verify accepted changes by readback when available. | Not implemented: no verified bounds/readback | Physical behaviour untested |
| M-05 | Temporary DPI shift restores the correct current base DPI on release, cancellation, profile change and recovery; nested software requests use defined ownership. | Not implemented: no DPI adapter | Physical behaviour untested |
| M-06 | Preserve or explicitly represent onboard/host-mode changes. Do not destroy existing onboard profiles to enable runtime assignments. | No onboard/host writes performed; restoration unimplemented | Physical behaviour untested |
| M-07 | Verify hold-to-modifier assignments, wheel tilt and profile/DPI controls individually. No assumption that all controls appear as normal evdev buttons. | Partial: held-output core tested; physical controls unverified | llc-core/tests/core.rs; simulated only |
| K-01 | G1–G5 are independently assignable in each of three software banks. M1/M2/M3 select those banks if their events are verified; GUI/CLI selection remains available. | Partial: three-bank model; hardware G/M routes blocked | llc-core/tests/core.rs; simulated only |
| K-02 | Bank indicator LEDs follow software state where supported. Hardware bank changes must not silently override active software mapping. | Not implemented: LEDs unverified | Physical behaviour untested |
| K-03 | Ordinary keys, including left/right modifiers, F-keys, navigation and keypad keys, are independently remappable through the input route. | Partial: Linux key-code model only | Physical behaviour untested |
| K-04 | Real F1–F5 must keep their configured ordinary-key behaviour when G1–G5 are remapped. No duplicate F-key action from G-keys. | Blocked: G/F event relationship unverified | Physical behaviour untested |
| K-05 | Preserve typing rollover, typematic behaviour, Caps/Num/Scroll lock feedback, media keys and volume wheel. Qualify ANSI/ISO layout mappings against the actual device. | Blocked: no live keyboard forwarding | Physical behaviour untested |
| K-06 | MR must never start hidden recording. If interceptable, allow its assignment to the explicit recording UI; otherwise show its hardware behaviour accurately. | No recording or MR-mode writes; physical behaviour unverified | Physical behaviour untested |
| K-07 | Distinguish profile selection from G-key bank selection; M keys change the G-key bank, not the entire profile, unless explicitly assigned otherwise. | Partial: separate bank/profile state and methods; physical M keys unverified | llc-core/tests/core.rs; simulated only |

## Lifecycle and macro rules

| Rule | Code/evidence | Remaining work |
|---|---|---|
| Shared ownership and cancellation (8.3) | Ledger, Scheduler; physical_modifier_survives_macro_and_cancellation | Actual unified input/output route |
| Limits, repeat balance and scheduling (8.1–8.2) | Macro::compile, Scheduler; boundary/quota/retrigger tests | Physical timing and performance |
| Captured mapping, layers and banks (6.2, 10.2) | Engine; captured-release/profile-transition/disconnect tests | Native forwarding and physical devices |
| Profile change and bypass (10.2–10.3) | Engine::select_profile/set_bypass; simulated cleanup | Physical chord suppression and watchdog recovery |
| Source disconnect (10.2) | Engine::disconnect; simulated shared-modifier tests | USB reattach, hold state and input restoration |
| Session ownership (4.2, 10.2) | Service authorize/supervise, SessionFacts tests, denied IPC test | Positive graphical-session transitions |
| SYN_DROPPED/reattach (10.4) | PhysicalState::synchronize; synthetic test | Live evdev resynchronization |
| GUI death/recording (10.2) | Recording is unavailable; session ownership is independent of client lifetime | GUI, recorder and authorized-client stream cleanup |
