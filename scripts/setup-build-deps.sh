#!/usr/bin/env bash
set -euo pipefail
if [[ $EUID == 0 ]]; then echo 'Run this script as your ordinary user; it uses sudo for system packages only.' >&2; exit 2; fi
command -v apt-get >/dev/null || { echo 'This helper supports Ubuntu/Mint/Debian using apt-get.' >&2; exit 2; }
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl ca-certificates dbus dbus-x11 systemd udev kmod python3 shellcheck
if ! command -v rustup >/dev/null; then
    temp=$(mktemp)
    trap 'rm -f -- "$temp"' EXIT
    curl --proto '=https' --tlsv1.2 -fsS https://sh.rustup.rs -o "$temp"
    sh "$temp" -y --profile minimal --default-toolchain 1.90.0 --component rustfmt --component clippy
else
    rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
fi
printf 'Build prerequisites installed. Run: source "%s/.cargo/env"\n' "$HOME"
echo 'Then run: ./scripts/build.sh'
