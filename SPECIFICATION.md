# Linux Logitech Control — Implementation Specification

Version: 1.0  
Date: 14 October 2026  
Status: implementation baseline; hardware capabilities remain subject to verification  
Owner: Christopher

## 1. Purpose and interpretation

Build a dependable, local Linux application providing the useful configuration and input functions of Logitech G-Hub for two initial devices: the wired Logitech G502 HERO mouse and wired Logitech G815 keyboard. The primary requirements are button/key reassignment and reliable macros. This is an independent implementation, with its own interface and configuration format.

The implementation language is **Rust**, with a **Qt 6 / QML** graphical interface. The primary release target is **Ubuntu 26.04 LTS, x86-64, GNOME on Wayland**, using the distribution kernel. Record the exact point release, kernel, GNOME, Qt and device firmware versions used for qualification. Other distributions, desktops, architectures and X11 sessions are not release-blocking targets for version 1.

MUST, MUST NOT, SHOULD and MAY are normative. MUST is a release requirement; SHOULD permits a documented, technically justified exception; MAY is optional. Requirement identifiers are stable and must appear in the implementation traceability document.

Hardware uncertainty is not permission to invent capabilities. A required feature that cannot be demonstrated must be reported as blocked or unsupported, with evidence. It must not be quietly removed from the acceptance criteria.

## 2. Scope

### 2.1 Version 1 requirements

| ID | Requirement |
|---|---|
| S-01 | Detect and manage both initial devices, including reconnect and suspend/resume. |
| S-02 | Reassign every electronically programmable G502 HERO control, including wheel tilt and the DPI/profile controls, after verifying its actual input path. |
| S-03 | Reassign G815 G1–G5, with three selectable software banks associated with M1–M3. |
| S-04 | Reassign ordinary G815 keyboard keys, including modifiers, function keys, navigation keys and numeric keypad keys. |
| S-05 | Support keyboard output, mouse-button output, wheel output, disabled controls, pass-through and profile/bank selection. |
| S-06 | Provide software macros with explicit presses/releases, delays, sequences, bounded repeat, repeat while held and toggle repeat. |
| S-07 | Provide reusable macros, profile editing, a macro editor/recorder, manual activation, import/export and a CLI. |
| S-08 | Support a software modifier layer equivalent in purpose to G-Shift, including a keyboard modifier changing mouse assignments. |
| S-09 | Configure G502 DPI stages, active/default stage, temporary DPI shift and supported polling rates where hardware verification establishes these controls. |
| S-10 | Provide capability-based basic lighting: off, static colour and brightness on verified zones/keys. See section 7 for the required support classification. |
| S-11 | Keep mappings working while the GUI is closed, with explicit user control over login activation. |
| S-12 | Recover safely from disconnect, invalid configuration, service failure and session changes. |
| S-13 | Ship a native Ubuntu package, source, tests, diagnostics and operational documentation. |

Both devices must work simultaneously. Profiles must be usable when one is absent. A device feature panel must distinguish unavailable hardware, unsupported firmware capability, permission failure and temporarily disconnected hardware.

### 2.2 Explicitly outside version 1

- General support for other Logitech devices, including G502 X, Lightspeed and G915 variants.
- Firmware flashing, bootloader support and kernel-driver development.
- Arbitrary macro sequences stored in device memory; portable operation without the service.
- Compatibility with G-Hub databases, cloud profiles, account services or Lua scripts.
- Full LIGHTSYNC effects, screen/audio sampling or game SDK integration.
- Automatic foreground-application profiles, GNOME extensions and game auto-detection.
- Launching arbitrary processes, shell commands, plugins or executable scripts from profiles.
- Text/Unicode injection, clipboard macros, absolute pointer placement and pointer-movement recording.
- General-purpose system-wide remapping of unrelated input devices.
- Simultaneous input ownership by multiple logged-in users or multiple seats.

These exclusions must not prevent later additions through documented interfaces. Do not implement them to compensate for incomplete required features.

## 3. Evidence and initial engineering gate

### 3.1 Known starting evidence

Upstream libratbag definitions identify the G502 HERO as USB `046d:c08b`, with its HID++ 2.0 backend and an index-offset quirk, and the G815 as `046d:c33f`, also using that backend. These are starting identifiers, not a complete whitelist of every firmware revision or regional variant.

Linux provides hidraw for vendor protocol access and evdev/uinput for input interception and virtual devices. Existing projects are useful implementation references, but their device listings do not prove complete support. Solaar documents G/M/MR diversion and also Wayland-specific limitations. Do not adopt its character-based simulation model without evaluating those limitations.

Source references and their permitted use are in section 22. Pin upstream commit IDs when relying on source behaviour. Check licensing before incorporating code.

### 3.2 Mandatory hardware probe, before GUI implementation

Create a Rust CLI probe and a capability report using the actual devices. Default probing must be read-only. Temporary device changes require an explicit probe option, must be recorded, and must be restored where the protocol permits. No firmware or persistent profile writes during this phase.

The probe MUST establish:

1. USB identity, firmware identity where obtainable, report descriptors, interfaces, hidraw nodes and evdev nodes belonging to each physical device.
2. The precise control-to-event map, including press and release, normal mode, any host/diverted mode, hardware bank keys, wheel tilt and DPI/profile buttons.
3. Whether each programmable control can be delivered uniquely without also generating its original action.
4. Whether the G815 G-keys duplicate F-key events, and how duplication is suppressed without suppressing the real F-keys.
5. The separation between G815 ordinary input, media controls, G-keys and M/MR controls.
6. Required configuration/mode operations, feature discovery, transport response matching and timeouts.
7. What happens after unplug/replug, host-service termination, device idle and suspend/resume.
8. Read/write/restore capabilities for DPI, report rate, lighting and host/onboard mode.
9. Whether changing the mode has side effects on ordinary typing, pointing or existing onboard settings.

Deliver `docs/hardware/G502-HERO.md`, `docs/hardware/G815.md`, sanitized descriptor/protocol fixtures and a human-readable capability table. Record exact hardware, firmware, regional keyboard layout, commands, observed results and untested cases.

**Gate G0:** every required input control must have an evidenced implementation route. If a route is unknown, continue independent work with simulated devices, but report the hardware-dependent item as blocked. Mocks and source inspection cannot pass G0. Do not substitute another device model for actual qualification.

## 4. Architecture

### 4.1 Components

Use a Cargo workspace with these logical boundaries; exact crate names may vary if documented:

| Component | Responsibility |
|---|---|
| `llc-core` | Typed configuration, validation, profile compilation, macro semantics, state machines; no device or GUI dependency. |
| `llc-protocol` | HID++ discovery, bounded transport, device adapters, capability translation and fixtures. |
| `llc-input` | evdev/uinput, physical-state tracking, output ownership, forwarding and recovery. |
| `llc-service` | Session authorization, live configuration, device lifecycle, scheduler and IPC. |
| Session agent | Lightweight ordinary-user process that loads the user's saved configuration and requests authorized login activation; no Qt or visible window required. |
| `llc-gui` | Qt Quick Controls/QML interface with Rust models via CXX-Qt or an equivalently maintained bridge. |
| `llc-cli` | Thin service client plus explicit offline validation and hardware probe commands. |

Keep privileged device access out of Qt/QML. Allow only minimal C++ glue required for Qt integration. Select compatible stable versions, pin the Rust toolchain and lock dependencies. Record the chosen Qt integration and system dependencies in an architecture decision record (ADR).

### 4.2 Runtime and permission boundary

Use a systemd-managed service for the input/device backend and ordinary-user GUI/CLI clients. The service may use a privileged broker plus an unprivileged engine if this improves isolation. Keep the privileged surface limited to opening authorized devices, input forwarding/output and validated device operations.

Use the system D-Bus with a versioned interface such as `au.craznar.LinuxLogitechControl1`. Inspect sender credentials and logind session state. Only an active, local, non-greeter session on the supported seat may obtain runtime input ownership. Reject remote, inactive and cross-user control. Restrict configuration operations to the caller's configuration context. Do not trust a UID supplied as a method argument.

The GUI must never require root. Do not grant all desktop applications unrestricted access to `/dev/input/*` or `/dev/uinput`, or add the user to a general input-reading group. Package only the permissions actually needed by the backend. Document service privileges, sandbox settings and the reason for each device-access exception.

Login activation must work without starting the GUI. Package a per-user session agent or equivalent systemd user integration to supply the saved configuration after local-session authorization. Keep its lifecycle separate from the GUI; the system service must not scan arbitrary home directories to discover profiles. The session agent must reacquire authorization after backend restart, without automatically restarting recording or macros. A missing agent leaves the backend passive rather than applying another user's profile.

### 4.3 Input routing

Use verified HID++ diversion for controls that support it, and evdev interception where required. An evdev grab requires forwarding every unaffected event from that interface through a suitable virtual device. The backend must be the sole reader/owner of its routing decisions; the GUI must not read raw keyboard streams.

Keep ordinary mouse motion and scrolling on the native path where possible. If interception is necessary, preserve relative movement, vertical/horizontal wheel data and high-resolution wheel semantics. Do not double-emit paired legacy/high-resolution wheel reports. Mirror appropriate keyboard/media capabilities and LEDs; do not advertise capabilities that cannot be handled. Never write back to the physical evdev input stream.

Tag and exclude virtual devices created by this application to prevent recursive capture. Track the whole USB composite device rather than relying on node numbering. Do not hard-code `/dev/input/eventN` or `/dev/hidrawN`.

Use physical Linux key codes as the internal keyboard representation, not X11 key symbols or Unicode characters. A displayed letter is a layout-aware label for a physical key; the output follows the active desktop layout. Qt is not the global input-injection mechanism.

Define one typematic source for each forwarded keyboard route. Do not combine physical `EV_KEY` repeat events with an independently repeating virtual device and thereby double repeat. Preserve ordinary desktop key-repeat behaviour and settings, while never treating repeat events as fresh macro triggers. Document the selected repeat strategy and test held characters, held navigation keys and held modifiers in native Wayland and XWayland clients.

### 4.4 Device adapter contract

Each adapter exposes identity, firmware information, named controls, supported actions/settings, readback, transient setting application, original-state capture and restoration where supported. Device-specific quirks belong in adapters with fixtures and references, not in GUI conditionals.

Every control declares its event source, suppression route, down/up support and runtime/onboard limitations. A wheel step is an impulse, not a held button. Mechanical controls that generate no electronic event must be labelled unassignable.

HID++ transactions must be serialized per physical device unless independent concurrency is demonstrated. Bound read lengths, pending requests and retry counts; distinguish replies from asynchronous notifications; validate report IDs, feature IDs, payload sizes and device indices. Do not blindly retry non-idempotent writes. Discovery/malformed reports must not block input forwarding.

## 5. Device-specific requirements

### 5.1 G502 HERO

| ID | Requirement |
|---|---|
| M-01 | Identify left, right, middle, back, forward, DPI shift, DPI up/down, profile control and left/right wheel tilt using physical labels and observed identities. |
| M-02 | Allow each electronically programmable control to use the common assignment model. Preserve ordinary pointing and scrolling. |
| M-03 | Exclude the mechanical wheel free-spin/ratchet switch if it produces no electronic event. Do not count it as an unimplemented programmable button. |
| M-04 | Expose supported DPI bounds, increments and stage counts from evidence; reject invalid values and verify accepted changes by readback when available. |
| M-05 | Temporary DPI shift restores the correct current base DPI on release, cancellation, profile change and recovery; nested software requests use defined ownership. |
| M-06 | Preserve or explicitly represent onboard/host-mode changes. Do not destroy existing onboard profiles to enable runtime assignments. |
| M-07 | Verify hold-to-modifier assignments, wheel tilt and profile/DPI controls individually. No assumption that all controls appear as normal evdev buttons. |

Primary-click reassignment is allowed, but Apply must offer a keyboard-accessible timed revert: retain the previous profile, require confirmation within 15 seconds, otherwise restore it. This guard also applies to disabling primary click. The emergency bypass is independent of this confirmation.

### 5.2 G815

| ID | Requirement |
|---|---|
| K-01 | G1–G5 are independently assignable in each of three software banks. M1/M2/M3 select those banks if their events are verified; GUI/CLI selection remains available. |
| K-02 | Bank indicator LEDs follow software state where supported. Hardware bank changes must not silently override active software mapping. |
| K-03 | Ordinary keys, including left/right modifiers, F-keys, navigation and keypad keys, are independently remappable through the input route. |
| K-04 | Real F1–F5 must keep their configured ordinary-key behaviour when G1–G5 are remapped. No duplicate F-key action from G-keys. |
| K-05 | Preserve typing rollover, typematic behaviour, Caps/Num/Scroll lock feedback, media keys and volume wheel. Qualify ANSI/ISO layout mappings against the actual device. |
| K-06 | MR must never start hidden recording. If interceptable, allow its assignment to the explicit recording UI; otherwise show its hardware behaviour accurately. |
| K-07 | Distinguish profile selection from G-key bank selection; M keys change the G-key bank, not the entire profile, unless explicitly assigned otherwise. |

Normal typing pass-through must be the initial state. Do not activate recording, remapping or mode changes merely because the configuration GUI was opened.

## 6. Assignment model

### 6.1 Actions

| Action | Semantics |
|---|---|
| Pass-through | Preserve the verified original action, including supported device defaults. |
| Disabled | Suppress original action and emit nothing. |
| Key/button | Output held while the source is held; release with its source. |
| Chord | Acquire listed keys in order on source down; release in reverse order on source up. |
| Wheel | Emit configured signed vertical or horizontal wheel units per activation. |
| Macro | Run a reusable macro under its configured execution mode. |
| Layer hold/toggle | Activate the secondary software assignment layer. |
| Bank/profile select | Change explicitly selected bank/profile, or cycle a defined ordered list. |
| DPI operation | Select stage, cycle stages or temporarily shift, where supported. |

An impulse control supports tap/chord-tap, wheel, one-shot macro, toggle and selection actions; it cannot directly hold a modifier or run repeat-while-held. Reject invalid combinations at validation time.

Pass-through for a diverted vendor control must reproduce its evidenced native function or restore a native route; forwarding an HID++ packet into uinput is not an implementation. If that function cannot be reproduced while another control on the interface is managed, expose the limitation as a G0 blocker for the affected required combination.

### 6.2 Layers and resolution

Each profile has base and secondary assignments. G815 G-keys additionally have bank 1–3 assignments. Resolve a control using: selected bank (G-keys only), then active layer, then explicit binding; an absent secondary binding inherits base. An absent base binding is pass-through. Bank changes do not alter ordinary-key mappings.

Hold-layer activators are reference-counted per physical source. The toggle state is independent; the layer is active if toggle is on or any hold source is active. Toggling off does not cancel a physically held layer source. Multiple devices can contribute layer holds.

Capture the resolved action on source down. A layer/bank change must not reinterpret its release. Held keys and running actions retain their captured binding until release, except an explicit profile change or bypass, which cancels them according to section 10. A layer activation must not recursively trigger itself through generated output.

Profiles must support a keyboard button activating the secondary mouse layer. If the keyboard disconnects while holding the layer, release its layer ownership immediately.

## 7. Hardware settings and lighting

Hardware settings are separate from software input mappings. Each setting has a support state: verified read/write, read-only, unsupported, untested, or temporarily unavailable. The GUI must not display a successful change based only on sending a request.

V1 MUST include setting models, UI and adapters for G502 DPI/report rate and basic lighting on both devices. Actual write support is required for features demonstrated by G0. Any requested basic setting not supported by the verified protocol must be explicitly documented in the release capability matrix with evidence; this exception does not extend to required key/macro functions.

Prefer transient host settings. Persistent hardware writes are not part of automatic profile application. Never continuously write device flash. Capture original readable values before changes and restore them when disabling management where feasible. Do not claim restoration for fields that cannot be read or verified.

Apply a software profile atomically within the engine. Hardware writes may be non-atomic: report software and hardware results separately, including partial failure. A failed lighting request must not stop keyboard/mouse input. Keep HID++ traffic off the input-forwarding critical path.

Basic lighting UI may provide off/static RGB/brightness, per verified device zone and per-key G815 lighting where supported. No animation requirement. Preserve hardware lighting when no software setting is selected.

## 8. Macro engine

### 8.1 Stored operations and limits

| Operation | Required semantics |
|---|---|
| `key_down` / `key_up` | Acquire/release a keyboard output owned by this macro instance. |
| `button_down` / `button_up` | Equivalent ownership for mouse buttons. |
| `tap` | Down, explicit hold interval, then up. Default 30 ms; stored in the macro. |
| `chord_tap` | Acquire ordered keys, hold, release in reverse order. |
| `wait` | Cancellable monotonic-clock delay. |
| `scroll` | Signed horizontal/vertical wheel steps. |
| `repeat` | Repeat a finite nested body a specified positive count. |

Do not include arbitrary code, conditions, recursive macro calls or profile switching inside macro bodies in V1. Profile/layer operations are source bindings, not macro instructions.

Limits: 4 MiB per imported configuration, 256 profiles, 1,024 reusable macros, 4,096 stored operations per macro, repeat nesting depth 4, repeat count 1–10,000, delay/hold 1–600,000 ms and 32 simultaneous macro instances. Reject expanded finite executions above 100,000 operations or 60 minutes predicted duration. Limit total macro-generated output to 2,000 events/second with bounded queues; exceeding a limit cancels the offending macro with an actionable error. This quota excludes ordinary physical forwarding and emergency cleanup. Do not drop release events as a throttling strategy. Limits must be constants with boundary tests, not scattered literals.

Validate explicit down/up balance along the finite execution path, including repeats. Reject release-without-owned-press and conflicting duplicate downs within one instance. A macro iteration must end with no held output. Long-lived hold behaviour uses an assignment/chord rather than an unbalanced macro. All terminal paths still perform defensive ownership cleanup.

### 8.2 Execution modes

| Mode | Start/stop semantics |
|---|---|
| Once | Source down starts one finite execution; source up does not stop it. |
| Repeat N | Source down runs the finite macro N times, subject to expanded limits. |
| While held | Source down starts; source up immediately cancels, including during waits or taps. |
| Toggle repeat | First source down starts; next source down cancels. Releases do not toggle. |

Repeating modes have a stored inter-iteration interval, minimum 10 ms. Trigger events generated by keyboard autorepeat must not start additional instances. No thread per operation and no blocking sleeps in input handling. Waiting uses a cancellable scheduler with a monotonic clock.

Per binding, support `ignore` (default) and `restart` when retriggered during an existing one-shot execution. V1 does not queue triggers for later. Different bindings may run concurrently within the global limit. New macro starts beyond the limit are rejected visibly; active release/cleanup work remains accepted.

### 8.3 Shared output ownership

Track output holds by source token: physical forwarding, held assignments and each macro instance. Emit a down only when aggregate ownership changes from zero to one, and an up only from one to zero. One macro must not release a modifier still owned by another action or forwarded physical key.

On a unified grabbed/forwarded keyboard path, physical modifier state participates in the same ownership ledger. When an unrelated keyboard remains native, do not claim global modifier ownership or isolation across devices. Document and test desktop aggregation behaviour; if reliable operation requires intercepting an additional keyboard, that is an explicit future scope decision, not an automatic broad grab.

If a macro taps a key that is already held by another owner, V1 preserves the held state; the tap does not force an up/down cycle and is not guaranteed to produce an additional application keypress. Show this semantic in macro-editor help. Do not release unrelated physical keys to force a chord to match.

For simultaneous events, preserve source ordering and use a deterministic scheduler tie-breaker. Do not promise real-time timing. Record scheduled versus emitted timestamps in opt-in diagnostic tests.

### 8.4 Macro editing and recording

Provide a list/timeline editor with insert, remove, reorder, edit, duplicate, undo/redo and delay normalization. Display explicit down/up and wait operations. Use clear physical key labels including left/right modifiers and keypad distinctions.

Recording starts only through an explicit visible action, stops through an independent GUI control or emergency chord, and excludes the start/stop controls. While recording, suspend mappings for selected sources and record their physical events, not injected output. Suppress selected recorded keys from applications by default; provide a clearly labelled alternative to pass them through. The state before recording must be restored on stop/cancel/failure.

Record keyboard keys, mouse buttons and scroll, with timing. Exclude pointer movement, hardware-mode events and the emergency bypass. No key-event persistence until the user saves the recorded macro. No background recording and no event content in ordinary logs. Provide editable conversion of hold/release sequences to taps/chords. Test playback is limited to a visible focused test area; global execution requires an explicit assigned trigger.

## 9. Profiles and persistence

### 9.1 Data model

Use versioned UTF-8 JSON with strict size/range/enum validation. Provide machine-readable JSON Schema and migrations. Reject unknown future schema versions without rewriting them. Use stable UUIDs for profiles/macros and semantic control IDs for assignments.

Required entities:

- Configuration: schema version, revision, application settings, device selectors, profiles, macro library and last-selected profile per user.
- Device selector: model, optional serial, optional explicit USB-port fallback, matching policy. Never persist event/hidraw node numbers.
- Profile: ID/name, device assignments, three G-key banks, secondary layer, optional verified hardware settings and default bank.
- Binding: physical control ID, action, execution mode, retrigger policy and referenced macro where applicable.
- Macro: ID/name, typed operations, repeat interval and editable timing.

Two identical devices without usable serials must be marked ambiguous until the user assigns port-based identity. Moving a port-bound device must not silently apply a potentially destructive profile to an unrelated unit. Regional layout is separate metadata, not a fabricated model match.

Defaults: pass-through for all controls; no user macros; no hardware settings changed; login activation off until explicitly enabled. After the user enables login activation, restore their validated selected profile on subsequent active-session acquisition.

### 9.2 Storage and application

Store user configuration under `$XDG_CONFIG_HOME/linux-logitech-control/` or the standard fallback. Store optional caches under XDG cache paths. The ordinary-user client writes user configuration; the privileged backend validates supplied structured content and must not read arbitrary client-specified paths.

Saving must use a same-directory temporary file, flush, atomic rename and directory synchronization where supported. Keep one last-known-good backup and private permissions. Handle full disk, read-only directories, interrupted writes and corrupted primary/backup files. Do not overwrite corrupt source files during recovery.

Distinguish Save from Apply: Save validates and persists edits; Apply validates, saves, then activates the selected revision. Report any activation failure without implying that the old active revision changed. Unsaved edits stay in the GUI process and must never alter running mappings. Export includes a self-contained validated profile bundle and referenced macros; exclude serials/port identity by default.

Use optimistic revisions to prevent GUI and CLI overwriting each other's edits. Reject stale changes with the current revision; do not silently merge conflicting assignments. Cancel running macros and perform the transition policy before swapping an active profile.

## 10. Lifecycle and recovery

### 10.1 Required states

Backend runtime states: `inactive`, `discovering`, `ready-pass-through`, `active`, `recording`, `bypass`, `recovering`, `conflict`, `error`. Devices additionally report `absent`, `permission-denied`, `unsupported` and individual capability health. Configuration validity, physical connection and active mapping status are separate facts.

### 10.2 Transition rules

| Trigger | Required action |
|---|---|
| GUI closes/crashes | Runtime continues; recording terminates and selected recording sources return to their prior state. |
| Profile change | Cancel macros, release software-owned output and layer toggles, select requested bank, swap compiled profile. Do not reinterpret held physical controls as new triggers. |
| Bank/layer change | New presses use new mapping; existing presses keep captured release semantics. |
| Source disconnect | Cancel its actions/macros, release its output/layer ownership, retain unrelated physical ownership. |
| Suspend/session lock | Stop recording/macros, disable mapped output, release software ownership and restore a verified native/pass-through route. |
| Resume/unlock | Rediscover/read state, restore verified modes, then reactivate only if enabled and the session is active; never restart previous macros. |
| User switch/logout | Revoke old owner, cancel runtime work and restore native/pass-through input before another session acquires control. |
| Service restart | Start pass-through, validate persisted configuration, reacquire active-session authorization and device state before enabling mappings. |
| Invalid new profile | Keep the last working active configuration; provide specific validation errors. |
| Output write failure | Cancel relevant actions, enter recovery and return to a verified fallback route. Do not accumulate unbounded pending events. |

For an active profile change, software-generated holds are released immediately. Physical keys held during the transition retain their old forwarded output until release; remapped/consumed sources are suppressed until their next release, then rearmed. Never synthesize a fresh trigger from an already-held source.

### 10.3 Bypass and failure containment

Provide an always-on physical **Ctrl+Alt+Backspace** emergency bypass when managing the G815, interpreted from its original physical keys before mapping/recording and independent of the active profile. Suppress the chord itself while managed. Allow a validated replacement emergency chord only if it remains reachable; do not allow disabling the last physical bypass. Also provide `llc bypass` from another terminal and an accessible GUI control.

Bypass cancels macros/recording, clears toggles, releases generated ownership and restores ordinary input within 250 ms under normal operation. Bypass remains latched until explicitly resumed; input release does not resume it. A CLI bypass still requires authorized local ownership.

Service shutdown must attempt restoration of captured transient modes/settings. Closing an evdev grab releases the grab, but that alone does not prove recovery of firmware-diverted controls. Design and test a supervisor/watchdog or equivalent recovery route for engine failure and missed heartbeats. Never claim a watchdog can restore modes if the process holding all recovery state is killed.

Test separate failure cases: GUI death, engine panic, blocked engine, broker termination and entire service kill. Ordinary typing and pointer movement must return within 2 seconds after a fatal backend failure on the reference system. Any G-key/diverted-control behaviour that needs service restart must be identified separately with its measured recovery time. The UI and documentation must not describe that case as immediate full native restoration.

Do not change boot/login-screen input. While locked, the normal physical keyboard/mouse route must remain usable and no macros may continue. Once bypass/recovery restores a route, prevent both native and virtual paths emitting the same event.

### 10.4 Input synchronization

Handle `SYN_DROPPED` using evdev state resynchronization. Do not replay unknown historical transitions as new macro triggers. Cancel actions whose source state cannot be established. Handle partial report reads, short writes, hung HID transactions and output device recreation. On reattach, wait for source releases before arming controls already down.

## 11. GUI requirements

Use Qt Quick Controls with native desktop behaviour, keyboard navigation, accessible labels, visible focus, standard dialogs and readable scaling at 100%, 150% and 200%. No dependence on a tray extension. Closing the window keeps enabled background management running; provide explicit Disable management and Quit interface commands.

Required screens:

1. **Devices/status:** both devices, connection, active profile/bank, management enabled, bypass and actionable faults.
2. **Assignments:** accurate labelled mouse control map; keyboard/G-key map and list alternative; base/secondary layer and G-key bank selectors; selected control action editor; restore default.
3. **Macros:** reusable library, operations editor, recording, validation, local test area and execution mode.
4. **Profiles:** create, rename, duplicate, delete, select, import/export; show unsaved versus saved versus active revision.
5. **Device settings:** verified DPI/report-rate/basic lighting capabilities and readback/partial failure status.
6. **Settings/diagnostics:** login activation, emergency chord, reset configuration, service status, hardware capability report and export diagnostics.

Do not require graphical device artwork to perform any operation. Repository-native diagrams/vector assets are sufficient; do not copy Logitech UI assets. Controls must have human labels and stable internal identifiers; protocol numbers belong in diagnostics.

An unavailable feature must show its reason, not silently disappear after a failed write. Errors must identify the operation and recovery action. No modal dialog for every unplug/replug. A device disconnect must not discard unsaved edits.

No network access, accounts, analytics, update downloads or notifications unrelated to this application's state. Diagnostics export is a user-initiated local file operation.

## 12. IPC and CLI

### 12.1 IPC contract

Provide a versioned D-Bus interface and publish its introspection XML. Methods must include status/capabilities, validate configuration, activate revision, select profile/bank, enable/disable management, bypass/resume, start/stop recording and diagnostics snapshot. Configuration bytes are bounded and parsed into typed values before application. Do not expose arbitrary HID writes, file reads or input injection methods to clients.

Signals must include runtime/device status, active revision/profile/bank, settings results, sanitized validation errors and recording progress. Raw recorded operations may be delivered only to the authorized client that started recording, with bounded buffers; never broadcast global keystrokes. Abandoned recording sessions end automatically when the client disappears.

Include interface version negotiation, structured error codes, request IDs for long operations, explicit completion and bounded timeouts. An accepted asynchronous request is not a successful hardware write. Clients reconnect without replaying destructive requests automatically.

### 12.2 CLI contract

Provide an installed `llc` command with at least:

```text
llc status [--json]
llc devices [--json]
llc capabilities <device> [--json]
llc profiles list [--json]
llc profile activate <id>
llc bank select <1|2|3>
llc config validate <file> [--json]
llc config import <file>
llc config export <file>
llc management enable|disable
llc bypass
llc resume
llc diagnostics export <file>
llc probe [--json] [--allow-transient-changes]
```

Paths are handled by the unprivileged CLI. No required CLI command may depend on the GUI. Define and document exit codes: 0 success; 2 invalid input/configuration; 3 authorization denied; 4 service unavailable; 5 device unavailable/unsupported; 6 operation failed/conflict. JSON output has an explicit schema version and must contain no terminal escape codes. Do not print raw input event content during normal operation.

## 13. Coexistence and device ownership

Only one instance of this backend may own each managed physical device. Detect likely competing managers such as ratbagd, Solaar, input-remapper and lighting tools where feasible; explain ambiguous behaviour. Process-name detection alone is not proof of exclusive ownership.

Never kill, uninstall, mask or reconfigure another manager automatically. Provide a documented manual resolution and a user-controlled retry. Prevent duplicate capture/output within this application through a broker lease and device grouping. If reliable ownership cannot be established, do not enable vendor mode writes or grabs that could disrupt input.

An unrelated keyboard or mouse must remain usable and must not inherit the Logitech profile. Do not intercept the USB passthrough device connected through the G815 merely because it shares an upstream hub.

## 14. Performance and resource requirements

Measure on a documented reference PC using release builds and the target OS; record CPU model, load and measurement method. These are qualification thresholds, not real-time guarantees:

| Metric | Target |
|---|---|
| Backend input-read to output-submit latency | p99 <= 5 ms at idle; p99 <= 10 ms during defined CPU-load test. |
| Macro wait accuracy, waits >= 20 ms | p99 absolute scheduler error <= 5 ms at idle; record under-load error separately. |
| Emergency bypass | <= 250 ms under normal running conditions. |
| Fatal backend failure to ordinary input recovery | <= 2 seconds on reference system. |
| Reattach to configured ready state | <= 3 seconds after udev reports all required interfaces ready, absent protocol faults. |
| Backend idle CPU | <= 1% of one logical core averaged over 60 seconds. |
| Backend resident memory | <= 100 MiB after warm-up under ordinary use; GUI <= 250 MiB. |
| Stability | 8-hour mixed-use run plus 24-hour automated input/macro soak without leaks, stuck outputs or uncontrolled queue growth. |

Do not busy-poll devices, use real-time scheduling by default, or benchmark GUI animation as input latency. Include nonintercepted native-path baseline measurements. A performance miss requires measurement and correction or an explicit owner-approved requirement change; do not silently weaken limits.

## 15. Security, privacy and fault limits

Configuration/imports are untrusted data. Validate lengths, arithmetic, nesting, references, control compatibility and timing before compiling. No dynamic script execution, shell interpolation or executable content in imports. The backend must validate independently of the GUI.

Use least privilege and logind-based ownership. Validate file-descriptor/device identity after opening to avoid path races. Restrict device operations to the supported hardware and verified functions; do not expose a generic privileged HID proxy.

Normal logs include lifecycle, errors, device identity with serials redacted, configuration revisions and operation results. They exclude typed keys, macro contents and arbitrary user paths. Diagnostic event capture is explicit, time-limited and visibly indicated; exports state what they contain. No telemetry or network dependency.

Use bounded input, IPC, recording and diagnostic queues. Fuzz configuration parsing, protocol decoding and macro compilation. A panic or malformed report must not leave a permanent grab or uncontrolled output loop.

## 16. Build and packaging

Provide a reproducible documented native Linux build and a Debian package for the target Ubuntu release. Use distribution Qt libraries where practical; include runtime package dependencies and comply with Qt/upstream license obligations. Application license remains an owner decision before distribution; do not claim that using open-source dependencies determines the application's license automatically.

The package must install GUI, CLI, backend, D-Bus policy, systemd units, desktop file, icons, schema, man/help documentation and only necessary device rules. Validate unit sandboxing against real device access. Do not ship a Flatpak-only release because host input access is part of the core feature set.

Installation starts the backend in passive state. User login activation remains opt-in. Package upgrades must quiesce active output, preserve configuration, back up before schema migration and report failed migrations. Uninstall must stop recording/macros, release grabs and attempt device mode restoration; preserve user configuration unless explicitly purged.

Windows may be used to edit source and run platform-neutral core tests where supported. Hardware validation and release qualification require native Linux; WSL/container/VM-only tests do not establish target input, Wayland, suspend or service behaviour.

CI must compile and test core/service/CLI, compile the QML interface, run formatting/linting/schema checks and build the package on a suitable Linux runner. Separate simulated CI results from hardware qualification. A clean build must work without an attached device and without contacting a proprietary Logitech service.

## 17. Verification strategy

### 17.1 Unit and property tests

Cover macro modes/retrigger, scheduler cancellation at every operation boundary, repeat limits, shared-output ownership, physical modifier interaction, layer/bank precedence, held-source transitions, strict parsing/migrations, invalid references and device report decoding.

Use a controllable monotonic test clock. Property tests must demonstrate that cancelling all sources returns software ownership to zero, that an owner cannot release another owner's hold and that injected events never reenter matching. Avoid wall-clock sleeps in core tests.

### 17.2 Integration tests

Use simulated transport plus virtual source devices in an isolated test environment. Verify suppression/forwarding, composite-device grouping, SYN_DROPPED handling, IPC authorization, client death during recording, device removal, failed hardware writes, service restart, corrupt configuration and resource limits. Fixture tests must include truncated, unexpected, delayed and out-of-order HID++ replies/notifications.

Run authorization tests for inactive/remote/different-user callers and session transitions. Test config import independently through GUI and CLI. Service tests may require privileges on the dedicated runner, not general workstation-wide input permissions.

### 17.3 Required physical acceptance cases

Record pass/fail/blocked, evidence and exact environment for every case:

| ID | Scenario and pass condition |
|---|---|
| A-01 | Fresh install: both devices type/point normally, no mode change or management until enabled. |
| A-02 | Every G502 programmable control uniquely triggers its assigned action; original action is suppressed; mechanical wheel switch excluded with evidence. |
| A-03 | G1–G5 in all three banks execute distinct actions; actual F1–F5 retain independent behaviour. |
| A-04 | Ordinary G815 keys remap correctly, including left/right modifiers, keypad, media and lock LED pass-through. |
| A-05 | Mouse button mapped to held Ctrl permits Ctrl+physical-key use and releases Ctrl exactly when expected. Repeat for Shift, Alt and Meta. |
| A-06 | Physical G815 Ctrl held during a macro remains held after macro completion/cancellation; test overlapping macro owners. |
| A-07 | Once, repeat N, while-held and toggle modes match section 8, including cancellation during waits and chord holds. |
| A-08 | G815 layer activator changes G502 assignments; release/disconnect restores base; held actions release under their captured mappings. |
| A-09 | Profile switching while keys/macros are held produces no stuck output or newly synthesized macro trigger. |
| A-10 | Close/crash GUI: mappings continue; active recording ends without leaving captured input suppressed. |
| A-11 | Emergency physical chord works despite reassigned modifiers/Backspace, active loops and recording; bypass remains latched. |
| A-12 | Unplug either device during held output: its actions stop; the other device remains usable; reconnect does not restart macros. |
| A-13 | At least 20 reconnects per device and 20 suspend/resume cycles preserve mappings, ordinary input and recovery rules. |
| A-14 | Lock/unlock, logout/login and a second-user session transition revoke/restore ownership correctly; no macros at login/lock screen. |
| A-15 | Panic, kill and hang each backend component; measure ordinary input restoration and separately document diverted-control recovery. |
| A-16 | Verified DPI stages/shift/report-rate/settings read back correctly; rejected writes never show success. |
| A-17 | Verified lighting controls work; disconnected/unsupported/write-failed states remain distinct; input continues during failure. |
| A-18 | Save/export/import on a clean user account preserves all software assignments/macros/banks/layers, excluding identity by default. |
| A-19 | Corrupt primary/backup, disk full and interrupted save do not erase last-good active state or silently replace source files. |
| A-20 | Competing manager yields a clear conflict or documented supported configuration; no automatic disabling of third-party services. |
| A-21 | Normal scrolling, horizontal tilt, high-resolution data and mouse motion remain correct without duplication under load. |
| A-22 | Native Wayland and at least one XWayland application receive equivalent physical-key/mouse output. |
| A-23 | With owner-provided installations, test WoW and Diablo IV through their actual Wine/Proton setup: modifier hold, G-key action and a short macro in an appropriate input context. No downloads/accounts assumed. If unavailable, mark game qualification untested, never passed. |
| A-24 | Qt UI works with keyboard-only navigation and 100/150/200% scaling; dangerous click reassignment timed revert succeeds. |
| A-25 | Package upgrade/uninstall preserves configuration and restores usable ordinary input; automatic activation follows the user's setting. |
| A-26 | Performance and soak thresholds in section 14 met with reports. |

Game qualification A-23 is an intended-use check, not a claim of application-specific integration or universal game compatibility. It may remain explicitly untested for a general release; all required device/input/recovery cases must pass on the supported target. Mark untested keyboard regional variants separately.

## 18. Implementation phases and exit conditions

### Phase 0 — Environment and hardware evidence

Inspect repository instructions and available Linux hardware. Build probe, capture descriptors, map controls and complete G0. Select stable dependencies, create ADRs for Qt bridge, ownership, device modes and input routing. Deliver a written blocked-items list if hardware is unavailable. Do not substitute a speculative protocol implementation for observations.

### Phase 1 — Minimal safe input service

Implement authorization, discovery, one-device-at-a-time routing, virtual output, identity filtering, pass-through, cancellation and bypass. Demonstrate native input recovery on failures before broad remapping. Then support both devices together. Exit: A-01 and relevant parts of A-10–A-15 pass.

### Phase 2 — Assignments and macro core

Implement typed configuration, key/button/chord/scroll assignments, banks/layers, macro semantics, ownership and deterministic tests. Add CLI activation. Exit: core functional acceptance A-02–A-09 and integration failure tests pass.

### Phase 3 — Hardware settings and complete GUI

Implement verified settings, profiles, Qt/QML editors, recorder, import/export, diagnostics and timed revert. Exit: applicable A-16–A-24 pass with capability exceptions explicitly documented only where permitted by section 7.

### Phase 4 — Package and qualify

Create native package, run clean install/upgrade/uninstall, full physical acceptance, latency/resource measurements and soak tests. Complete documentation and traceability. Exit: all mandatory cases passed, release capability matrix accurate, no unresolved stuck-output/duplicate-input/recovery defects.

Each phase must leave a buildable repository. Provide concise progress evidence and the next actual blocker. Do not stop after scaffolding, a GUI mockup, simulated tests or passing compilation and call the project complete.

## 19. Required repository deliverables

```text
SPECIFICATION.md
IMPLEMENTATION_PROMPT.md
README.md
Cargo.toml
Cargo.lock
rust-toolchain.toml
crates/                         # Rust components and minimal Qt bridge
qml/                            # GUI, unless colocated with gui crate
packaging/debian/
packaging/systemd/
packaging/dbus/
schemas/                        # JSON and IPC/CLI output schemas
tests/fixtures/                 # Sanitized protocol/input fixtures
docs/architecture/              # ADRs, trust boundary, lifecycle and routing
docs/hardware/G502-HERO.md
docs/hardware/G815.md
docs/CONFIGURATION.md
docs/MACROS.md
docs/BUILD.md
docs/OPERATIONS.md
docs/TESTING.md
docs/ACCEPTANCE.md
docs/TRACEABILITY.md
docs/DEPENDENCIES.md
docs/KNOWN-LIMITATIONS.md
```

`TRACEABILITY.md` maps S/M/K requirements and lifecycle/macro rules to implementation locations and verification evidence. `ACCEPTANCE.md` records actual results and environment, distinguishing automated, simulated and physical tests. `OPERATIONS.md` documents enable/disable, bypass, restoring device modes, service logs, conflict resolution and package removal. List all unsupported features explicitly.

## 20. Definition of done

Version 1 is complete only when:

1. Both actual device models pass mandatory input/macro/recovery acceptance on the primary target.
2. No reproducible stuck modifier/button, duplicate original event, runaway macro or unusable ordinary input remains unresolved.
3. GUI, CLI and background service implement the same validated configuration semantics.
4. Native package installs/upgrades/uninstalls correctly and defaults remain passive.
5. Build, meaningful automated tests, hardware reports, performance and soak evidence are present.
6. Capabilities and limitations accurately reflect measurements; no untested claim is presented as working.
7. Dependency licenses/notices are documented and the owner has selected the application license before distribution.

Implementation can make substantial progress without hardware, but it cannot meet this definition of done without physical qualification.

## 21. Future extension boundaries

Keep explicit extension points for onboard-profile writers, other device adapters, a foreground-application provider, additional lighting effects and other platform packages. Do not expose executable plugin loading in V1.

A future foreground provider must use a supported desktop integration and treat process identity, focused window and Wine/Proton executable identity separately. Onboarding and permission requirements must be specified before implementation. Focus information is not a prerequisite for manual profiles.

A future onboard writer must have a separate capability matrix, read/backup/verify/restore plan, storage limits and hardware acceptance tests. Software macro support does not imply portable onboard macro support.

## 22. Primary references and evidence policy

References checked 14 October 2026. Links are starting evidence, not immutable API contracts. Record the exact revision or installed version when implementation begins.

- [Ubuntu desktop release information](https://ubuntu.com/download/desktop) and [26.04 release summary](https://documentation.ubuntu.com/release-notes/26.04/summary-for-lts-users/): target platform selection.
- [Linux hidraw documentation](https://docs.kernel.org/hid/hidraw.html): userspace access to vendor HID reports and dynamic device discovery.
- [Linux uinput documentation](https://docs.kernel.org/input/uinput.html): virtual keyboard/mouse output from userspace.
- [G502 HERO upstream device definition](https://github.com/libratbag/libratbag/blob/master/data/devices/logitech-g502-hero.device): initial USB ID/backend/quirk evidence.
- [G815 upstream device definition](https://github.com/libratbag/libratbag/blob/master/data/devices/logitech-g815.device): initial USB ID/backend evidence.
- [libratbag HID++ backend](https://github.com/libratbag/libratbag/blob/master/src/driver-hidpp20.c): protocol/configuration implementation reference; investigate actual macro-writing limits.
- [Solaar rules documentation](https://pwr-solaar.github.io/Solaar/rules/): notification/diversion model and documented Wayland limitations.
- [G815 diversion issue](https://github.com/pwr-Solaar/Solaar/issues/2916): reported duplicate/default-key problem requiring reproduction; not proof every unit is affected.
- [G502 multi-key macro issue](https://github.com/libratbag/piper/issues/949): existing reported limitation; not a current universal support verdict.
- [Input Remapper](https://github.com/sezanzeb/input-remapper): reference for Linux runtime remapping and macro architecture.
- [CXX-Qt documentation](https://kdab.github.io/cxx-qt/book/): Rust/Qt integration candidate; verify supported versions when selecting the toolchain.

Only use legally reusable source and document its license/attribution. Protocol captures must be sanitized before inclusion. Do not bundle Logitech software, firmware, trademarks as application branding, private configuration data or copied proprietary UI assets.
