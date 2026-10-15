# Engineering test evidence

Recorded 15 October 2026 for the initial Rust engineering implementation. All results below concern the implemented passive tools and simulated core. No physical acceptance case is marked passed.

## Environments

| Environment | Details |
|---|---|
| WSL build/test | Ubuntu 24.04.4, x86-64, WSL2 kernel 6.6.87.2-microsoft-standard-WSL2 |
| Native build/test | Linux Mint 22.3 (Ubuntu noble base), x86-64, kernel 7.0.0-31-generic |
| Rust | 1.90.0, pinned toolchain and Cargo.lock |
| Physical devices | No target G502 HERO or G815 attached |
| Release desktop | Ubuntu 26.04 GNOME/Wayland not tested |

## Results

| Check | Command | Observed result |
|---|---|---|
| Formatting | cargo fmt --all -- --check | Passed WSL/native |
| Strict lint | cargo clippy --workspace --all-targets --locked -- -D warnings | Passed WSL/native |
| Core tests | cargo test --workspace --locked | 21 passed, 0 failed, on both hosts |
| Release build | scripts/build.sh | CLI/backend/agent and libraries built on both hosts; schema comparison and dependency notices passed |
| Shell validation | shellcheck scripts/*.sh tests/integration/session.sh | Passed in WSL; shellcheck unavailable on native host |
| Isolated IPC/CLI | scripts/test-integration.sh release | Passed on WSL/native: passive startup, profiles/import/export, JSON, denied nonlocal mutation, service loss |
| Debian package | scripts/package.sh | Built on WSL/native |
| Source installer lifecycle | scripts/test-package.sh as root in WSL | Install/start/remove passed; dedicated service account and systemd startup worked; configuration preserved |
| Debian lifecycle | scripts/test-package.sh as root in WSL | Install, same-version reinstall/upgrade-script path, service startup and removal passed; configuration preserved |
| Unit validation | systemd-analyze verify on installed system/user units | Passed in WSL |
| Example configuration | llc config validate examples/config.json | Passed on native Linux |
| Offline macro trace | llc macro simulate example UUID | Ctrl/A down at 0 ms; A/Ctrl up at 30 ms; physical_output=false |
| Read-only inventory | llc probe --json | Native host reports no target models; G0-unverified |
| Runtime dynamic libraries | ldd release/llc | libc and libgcc; no Qt or proprietary service dependency |

Core tests cover shared physical/software modifiers, cleanup order, while-held cancellation during a wait, once/toggle/retrigger, simultaneous-instance and event quotas, backwards-clock rejection, oversized repeats, macro bounds/balance, layers/banks/captured releases, profile-change suppression, source disconnect, latched bypass model, strict JSON, revision conflicts and corrupt-primary/backup preservation. Protocol/input tests use synthetic framing/sysfs/state; they are not hardware evidence.

The install lifecycle test was run on the selected WSL test host and left the backend removed. It preserved the service account/documentation as documented by source uninstall. No software was installed as a system service on the native NUC; its source and build output remain in the dedicated user build directory.

## Unqualified items

Positive authorization through a real graphical session, login activation/unlock/restart in GNOME, physical event capture with the actual devices, HID++ operations, live routing, emergency recovery, GUI, game behaviour, target-OS installation, latency and soak tests remain untested. Same-version package reinstall is not a schema-migration or cross-release upgrade qualification.

See ACCEPTANCE.md and KNOWN-LIMITATIONS.md. Gate G0 still requires observed control routes on the exact wired models. Keep original captures outside Git and add only reviewed sanitized fixtures.
