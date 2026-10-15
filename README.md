# Linux Logitech Control

Independent Linux configuration and input software planned for the wired Logitech G502 HERO mouse and G815 keyboard.

**Status:** repository setup only. No application code, executable, device probe or package has been implemented. No hardware capabilities or acceptance tests have been verified.

The implementation baseline is [SPECIFICATION.md](SPECIFICATION.md). [IMPLEMENTATION_PROMPT.md](IMPLEMENTATION_PROMPT.md) describes the implementation workflow. Planned technology: Rust and Qt 6/QML, targeting Ubuntu 26.04 LTS on x86-64 with GNOME/Wayland.

## Repository layout

| Path | Purpose |
|---|---|
| `crates/` | Rust core, protocol, input, service, session agent, GUI and CLI components |
| `qml/` | Qt Quick interface |
| `docs/` | Build, configuration, operation, testing and project evidence |
| `docs/architecture/` | Architecture decisions and component boundaries |
| `docs/hardware/` | Device investigation and capability reports |
| `packaging/debian/` | Ubuntu package metadata |
| `packaging/systemd/` | Backend and opt-in session integration |
| `packaging/dbus/` | Versioned D-Bus interface and access policy |
| `schemas/` | Configuration and IPC/CLI output schemas |
| `tests/fixtures/` | Sanitized hardware captures and separately identified simulated data |

See the [documentation index](docs/README.md), [implementation status](docs/KNOWN-LIMITATIONS.md) and [acceptance register](docs/ACCEPTANCE.md).

## First implementation phase

Select and pin the Rust toolchain and Qt bridge, initialize the Cargo workspace, then implement the read-only hardware probe required by specification section 3.2. Record actual device observations before claiming input support or implementing the GUI. Independent core work may proceed with explicitly simulated data while hardware gate G0 remains open.

There are no build or installation commands yet. Cargo manifests, dependency locks, CI and package definitions will be added with their implementations.

## License

The owner has not selected an application license. No open-source license is granted by this repository setup. Record dependency licenses before incorporating third-party code or distributing builds.
