# Architecture and decisions

Specification section 4 defines the architecture. Planned crate paths:

| Path | Responsibility |
|---|---|
| `crates/llc-core/` | Configuration, validation, compilation and state machines |
| `crates/llc-protocol/` | HID++ transport, adapters and capability translation |
| `crates/llc-input/` | evdev/uinput, physical state, forwarding and recovery |
| `crates/llc-service/` | Authorization, lifecycle, scheduling and D-Bus |
| `crates/llc-session-agent/` | Ordinary-user saved configuration and login activation |
| `crates/llc-gui/` | Rust Qt integration and view models |
| `crates/llc-cli/` | Service client, offline validation and hardware probe |

These are reserved source paths, not implemented Cargo crates. `qml/` is reserved for the GUI.

Record decisions as numbered Markdown ADRs with context, status, decision, consequences and verification evidence. Initial decisions must cover the Rust/Qt bridge and versions, privilege boundary, device modes, routing and repeat strategy. No implementation ADR has been accepted yet.
