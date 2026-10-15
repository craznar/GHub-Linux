# Requirement traceability

Repository setup only. All requirements below remain unimplemented and unverified. Requirement text comes from the normative specification; add source locations and evidence as implementation progresses.

| ID | Requirement | Implementation | Verification |
|---|---|---|---|
| S-01 | Detect and manage both initial devices, including reconnect and suspend/resume. | Not implemented | Untested |
| S-02 | Reassign every electronically programmable G502 HERO control, including wheel tilt and the DPI/profile controls, after verifying its actual input path. | Not implemented | Untested |
| S-03 | Reassign G815 G1–G5, with three selectable software banks associated with M1–M3. | Not implemented | Untested |
| S-04 | Reassign ordinary G815 keyboard keys, including modifiers, function keys, navigation keys and numeric keypad keys. | Not implemented | Untested |
| S-05 | Support keyboard output, mouse-button output, wheel output, disabled controls, pass-through and profile/bank selection. | Not implemented | Untested |
| S-06 | Provide software macros with explicit presses/releases, delays, sequences, bounded repeat, repeat while held and toggle repeat. | Not implemented | Untested |
| S-07 | Provide reusable macros, profile editing, a macro editor/recorder, manual activation, import/export and a CLI. | Not implemented | Untested |
| S-08 | Support a software modifier layer equivalent in purpose to G-Shift, including a keyboard modifier changing mouse assignments. | Not implemented | Untested |
| S-09 | Configure G502 DPI stages, active/default stage, temporary DPI shift and supported polling rates where hardware verification establishes these controls. | Not implemented | Untested |
| S-10 | Provide capability-based basic lighting: off, static colour and brightness on verified zones/keys. See section 7 for the required support classification. | Not implemented | Untested |
| S-11 | Keep mappings working while the GUI is closed, with explicit user control over login activation. | Not implemented | Untested |
| S-12 | Recover safely from disconnect, invalid configuration, service failure and session changes. | Not implemented | Untested |
| S-13 | Ship a native Ubuntu package, source, tests, diagnostics and operational documentation. | Not implemented | Untested |
| M-01 | Identify left, right, middle, back, forward, DPI shift, DPI up/down, profile control and left/right wheel tilt using physical labels and observed identities. | Not implemented | Untested |
| M-02 | Allow each electronically programmable control to use the common assignment model. Preserve ordinary pointing and scrolling. | Not implemented | Untested |
| M-03 | Exclude the mechanical wheel free-spin/ratchet switch if it produces no electronic event. Do not count it as an unimplemented programmable button. | Not implemented | Untested |
| M-04 | Expose supported DPI bounds, increments and stage counts from evidence; reject invalid values and verify accepted changes by readback when available. | Not implemented | Untested |
| M-05 | Temporary DPI shift restores the correct current base DPI on release, cancellation, profile change and recovery; nested software requests use defined ownership. | Not implemented | Untested |
| M-06 | Preserve or explicitly represent onboard/host-mode changes. Do not destroy existing onboard profiles to enable runtime assignments. | Not implemented | Untested |
| M-07 | Verify hold-to-modifier assignments, wheel tilt and profile/DPI controls individually. No assumption that all controls appear as normal evdev buttons. | Not implemented | Untested |
| K-01 | G1–G5 are independently assignable in each of three software banks. M1/M2/M3 select those banks if their events are verified; GUI/CLI selection remains available. | Not implemented | Untested |
| K-02 | Bank indicator LEDs follow software state where supported. Hardware bank changes must not silently override active software mapping. | Not implemented | Untested |
| K-03 | Ordinary keys, including left/right modifiers, F-keys, navigation and keypad keys, are independently remappable through the input route. | Not implemented | Untested |
| K-04 | Real F1–F5 must keep their configured ordinary-key behaviour when G1–G5 are remapped. No duplicate F-key action from G-keys. | Not implemented | Untested |
| K-05 | Preserve typing rollover, typematic behaviour, Caps/Num/Scroll lock feedback, media keys and volume wheel. Qualify ANSI/ISO layout mappings against the actual device. | Not implemented | Untested |
| K-06 | MR must never start hidden recording. If interceptable, allow its assignment to the explicit recording UI; otherwise show its hardware behaviour accurately. | Not implemented | Untested |
| K-07 | Distinguish profile selection from G-key bank selection; M keys change the G-key bank, not the entire profile, unless explicitly assigned otherwise. | Not implemented | Untested |

Lifecycle rules (section 10) and macro rules (section 8) also remain unimplemented and unverified. Add rule-level mappings alongside the corresponding state machines and tests; this initial register does not establish complete rule coverage.
