#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if [[ $EUID != 0 ]]; then echo 'Run: sudo ./scripts/install.sh [--install-deps] [--no-start]' >&2; exit 2; fi
install_deps=false
start=true
for argument in "$@"; do
    case "$argument" in --install-deps) install_deps=true;; --no-start) start=false;; *) echo "Unknown argument: $argument" >&2; exit 2;; esac
done
target=${CARGO_TARGET_DIR:-$root/target}
for binary in llc llc-service llc-session-agent; do
    [[ -x $target/release/$binary ]] || { echo "Missing $binary; build as your ordinary user first." >&2; exit 2; }
done
[[ -d $target/third-party ]] || { echo 'Dependency notices are missing; run scripts/build.sh before installing.' >&2; exit 2; }
if $install_deps; then
    command -v apt-get >/dev/null || { echo '--install-deps requires apt-get.' >&2; exit 2; }
    apt-get update
    DEBIAN_FRONTEND=noninteractive apt-get install -y dbus systemd udev kmod
fi
for utility in install systemctl systemd-sysusers udevadm modprobe; do
    command -v "$utility" >/dev/null || { echo "Missing utility $utility; rerun with --install-deps." >&2; exit 2; }
done
[[ -d /run/systemd/system ]] || { echo 'A running systemd host is required. Use package.sh for a staged package.' >&2; exit 2; }
systemctl stop linux-logitech-control.service 2>/dev/null || true
install -Dm644 "$root/packaging/sysusers/linux-logitech-control.conf" /usr/lib/sysusers.d/linux-logitech-control.conf
systemd-sysusers /usr/lib/sysusers.d/linux-logitech-control.conf
install -Dm755 "$target/release/llc" /usr/bin/llc
install -Dm755 "$target/release/llc-service" /usr/libexec/llc-service
install -Dm755 "$target/release/llc-session-agent" /usr/libexec/llc-session-agent
install -Dm644 "$root/packaging/systemd/linux-logitech-control.service" /usr/lib/systemd/system/linux-logitech-control.service
install -Dm644 "$root/packaging/systemd/llc-session-agent.service" /usr/lib/systemd/user/llc-session-agent.service
install -Dm644 "$root/packaging/dbus/au.craznar.LinuxLogitechControl1.conf" /etc/dbus-1/system.d/au.craznar.LinuxLogitechControl1.conf
install -Dm644 "$root/packaging/dbus/au.craznar.LinuxLogitechControl1.xml" /usr/share/dbus-1/interfaces/au.craznar.LinuxLogitechControl1.xml
install -Dm644 "$root/schemas/config.schema.json" /usr/share/linux-logitech-control/config.schema.json
install -Dm644 "$root/packaging/man/llc.1" /usr/share/man/man1/llc.1
install -d /usr/share/doc/linux-logitech-control
cp -R "$root/docs/." /usr/share/doc/linux-logitech-control/
cp -R "$target/third-party" /usr/share/doc/linux-logitech-control/
install -m644 "$root/README.md" "$root/SPECIFICATION.md" /usr/share/doc/linux-logitech-control/
find /usr/share/doc/linux-logitech-control -type d -exec chmod 755 {} +
find /usr/share/doc/linux-logitech-control -type f -exec chmod 644 {} +
systemctl daemon-reload
systemctl reload dbus
systemctl enable linux-logitech-control.service
if $start; then systemctl start linux-logitech-control.service; fi
echo 'Installed the passive engineering backend, CLI, user agent, documentation and schemas.'
echo 'No input permissions, device modes or firmware were changed. Live remapping is unavailable until G0 qualification.'
echo 'As your desktop user, run: systemctl --user daemon-reload'
echo 'Optional session agent: systemctl --user enable --now llc-session-agent.service'
echo 'Login management is separately opt-in using: llc login enable'
