# Build

The workspace builds `llc`, `llc-service`, `llc-session-agent` and core/protocol/input libraries. The GUI remains outside the workspace until the specification's pre-GUI G0 gate passes.

Prerequisites: x86-64 Ubuntu/Mint/Debian, C toolchain, pkg-config, curl/CA certificates, Python 3, and Rust 1.90.0 with rustfmt/clippy. Test helpers use dbus-run-session and shellcheck. Runtime uses D-Bus, systemd, udev and kmod; no Qt runtime is needed by this pre-GUI build.

```sh
./scripts/setup-build-deps.sh
source "$HOME/.cargo/env"
./scripts/build.sh
./scripts/test-integration.sh release
./scripts/package.sh
```

Run setup/build as your ordinary user. The dependency helper uses sudo for apt packages and installs Rust in your account. Initial dependency fetching needs Internet access. `build.sh` uses Cargo.lock and runs formatting, strict clippy, workspace tests, release compilation, schema comparison and dependency-notice collection. `--debug` selects a development build. Use an absolute `CARGO_TARGET_DIR` consistently across build/package/install if overriding the default.

Package output: `dist/linux-logitech-control_0.1.0_amd64.deb`. Install with `sudo apt install ./dist/linux-logitech-control_0.1.0_amd64.deb`, or use the source installer described in OPERATIONS.md. Remove the previous installation method before switching between source installation and Debian packaging.

Ubuntu 24.04 WSL and native Mint 22.3 are engineering verification environments. Neither qualifies the Ubuntu 26.04 GNOME/Wayland target. No Logitech device was attached. Exact results are recorded in TESTING.md.
