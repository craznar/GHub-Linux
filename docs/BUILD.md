# Build

Target: Ubuntu 26.04 LTS, x86-64, GNOME/Wayland, Rust backend/core and Qt 6/QML interface.

There is no application build yet. This setup session ran on Windows; Cargo, rustc and rustup were not available on PATH. No Linux target or physical device was inspected.

Before adding code, select compatible stable Rust and Qt bridge versions, document the decision in `architecture/`, create `rust-toolchain.toml`, initialize the root Cargo workspace and commit `Cargo.lock`. Document exact Ubuntu development packages and reproducible build commands after verification.

Windows can support source editing and platform-neutral core verification. Release qualification requires native Linux. See specification sections 4 and 16.
