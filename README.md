# Linux Logitech Control

Independent Linux configuration and input software for the wired Logitech G502 HERO and G815.

**Engineering build: not a usable G-Hub replacement yet.** Implemented: read-only inventory and explicitly requested event capture, validated profiles, offline macro simulation, macro/ownership core, passive D-Bus backend, session agent and build/install/package scripts. Live remapping, HID++ operations, recording and the Qt GUI remain unavailable pending hardware gate G0. No physical capability is verified.

The baseline is [SPECIFICATION.md](SPECIFICATION.md); [IMPLEMENTATION_PROMPT.md](IMPLEMENTATION_PROMPT.md) defines the workflow. Rust 1.90.0 is pinned; dependencies are locked. The release target remains Ubuntu 26.04 x86-64 GNOME/Wayland. Current engineering checks use Ubuntu 24.04 WSL and native Mint 22.3. GUI work follows G0 as required by the specification.

## Build and install

From a source checkout on Ubuntu/Mint:

```sh
./scripts/setup-build-deps.sh
source "$HOME/.cargo/env"
./scripts/build.sh
sudo ./scripts/install.sh --install-deps
```

The build script compiles **all implemented parts**, tests/lints them, verifies the schema and collects dependency licenses. Installation registers a passive system service and installs the CLI, optional user agent, D-Bus policy, schemas, man page and documentation. It changes no input permissions or device settings. See [BUILD.md](docs/BUILD.md) and [OPERATIONS.md](docs/OPERATIONS.md).

```sh
llc config init
llc probe --json
llc config validate examples/config.json
llc macro simulate examples/config.json 00000000-0000-4000-8000-000000000002 --json
```

Simulation prints JSON only and never injects input. Build an engineering Debian package with `./scripts/package.sh`. CI checks the same workspace; it does not qualify hardware.

## Layout

| Path | Purpose |
|---|---|
| `crates/` | Core, protocol, input, backend, CLI and session agent; GUI path reserved |
| `scripts/` | Build, dependency setup, install/remove, integration and packaging |
| `docs/` | Architecture, hardware evidence, operation and qualification records |
| `packaging/` | Debian metadata, systemd, D-Bus, sysusers and man page |
| `schemas/` | Generated configuration schema and CLI envelope |
| `examples/` | Validated configuration and simulation example |
| `tests/` | Integration tests and sanitized fixture paths |
| `qml/` | Reserved until the pre-GUI hardware gate passes |

## Hardware evidence

The architecture uses Linux's existing USB HID, hidraw, evdev and uinput interfaces; custom kernel-driver development and firmware flashing are outside scope. Device-specific host/diversion operations require observed evidence before implementation/enablement. Follow [HARDWARE-TESTING.md](docs/HARDWARE-TESTING.md) when the devices are available.

See [known limitations](docs/KNOWN-LIMITATIONS.md), [test evidence](docs/TESTING.md) and the [documentation index](docs/README.md). Building successfully does not mean the specification's definition of done is met.

## License

The owner has not selected an application license. No open-source license is granted by this engineering setup. Dependency licenses/notices are collected with builds and included in packages.
