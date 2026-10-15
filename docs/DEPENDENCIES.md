# Dependencies and licensing

Rust 1.90.0 is pinned in rust-toolchain.toml. Cargo.lock locks registry dependencies. Primary libraries: serde/serde_json for typed JSON, schemars for schema generation, uuid for stable IDs, fs2 for configuration locks, libc for bounded read-only Linux capture, zbus for system D-Bus and thiserror for core errors.

The native runtime uses distribution glibc/libgcc, D-Bus, systemd/logind and sysfs. No third-party Logitech code, firmware or assets are incorporated. Protocol references remain investigation evidence only.

`scripts/third-party-notices.py` reads locked Cargo metadata and collects registry license/copyright/notice files. It fails if a selected dependency has no license text. Build output goes under target/third-party (or CARGO_TARGET_DIR); installers/packages include it in /usr/share/doc/linux-logitech-control/third-party.

Qt integration remains unselected until G0. Candidate documentation: https://kdab.github.io/cxx-qt/book/ . No Qt bridge dependency or GUI code is included in this build. The owner must select the application license before a release; no application open-source grant is implied by dependency licenses.
