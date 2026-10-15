#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
profile=${1:-debug}
export LLC_TEST_BIN="$target/$profile"
command -v dbus-run-session >/dev/null || { echo 'Install dbus-x11 for the isolated IPC test.' >&2; exit 2; }
dbus-run-session -- bash "$root/tests/integration/session.sh"
