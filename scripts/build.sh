#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"
profile=release
if [[ ${1:-} == --debug ]]; then profile=debug; shift; fi
if (($#)); then echo 'Usage: scripts/build.sh [--debug]' >&2; exit 2; fi
command -v cargo >/dev/null || { echo 'Rust is missing. Run scripts/setup-build-deps.sh as your ordinary user.' >&2; exit 2; }
command -v cc >/dev/null || { echo 'A C toolchain is missing. Run scripts/setup-build-deps.sh.' >&2; exit 2; }
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
if [[ $profile == release ]]; then cargo build --workspace --release --locked; else cargo build --workspace --locked; fi
target=${CARGO_TARGET_DIR:-$root/target}
"$target/$profile/llc" config schema > "$target/config.schema.json"
cmp schemas/config.schema.json "$target/config.schema.json"
if command -v shellcheck >/dev/null; then shellcheck scripts/*.sh tests/integration/session.sh; fi
python3 scripts/third-party-notices.py "$target/third-party"
printf '\nBuild complete: %s/%s\nEngineering build: hardware management remains gated by G0.\n' "$target" "$profile"
