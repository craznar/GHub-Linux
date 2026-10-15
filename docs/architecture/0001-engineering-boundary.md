# ADR 0001: engineering build and qualification boundary

Status: accepted for the pre-G0 engineering build, 15 October 2026.

## Context

The implementation baseline requires actual G502 HERO/G815 evidence before GUI work and before claiming reliable input support. The available native Linux build host has neither device attached. Independent implementation is allowed; hardware evidence may not be invented.

## Decision

Pin Rust 1.90.0 and lock Cargo dependencies. Build/test the platform-neutral core, read-only inventory/capture, CLI and passive D-Bus/session foundation now. Defer the Qt bridge/version choice and GUI until G0. No executable kernel modules are required: Linux HID, evdev and uinput provide the intended OS interfaces.

The engineering backend runs as a dedicated unprivileged account with PrivateDevices, no capabilities and no home access. It receives bounded validated configuration bytes, never client-supplied file paths. Bus credentials and logind determine authorization; client arguments cannot nominate a UID. Ownership is session-based; loss of authorization clears staged state. The user agent re-stages validated data after restart/unlock and stores login opt-in separately from GUI lifetime.

No live input grab, virtual output or HID mode writes are exposed. Enabled-management requests return DeviceUnavailable with G0 as the reason. Capability reports say untested. The input emergency/resynchronization model and scheduler are simulated primitives, not a claimed watchdog or physical bypass.

## Consequences

The engineering tools can be installed and exercised without changing ordinary input. Successful compilation does not complete phases 0–4. After hardware probing, create separate ADRs for verified routing, repeat strategy, privileged device opening/recovery and the maintained Qt bridge. Package privilege exceptions must be justified by that evidence before adding them.
