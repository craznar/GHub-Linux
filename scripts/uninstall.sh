#!/usr/bin/env bash
set -euo pipefail
if [[ $EUID != 0 ]]; then echo 'Run: sudo ./scripts/uninstall.sh' >&2; exit 2; fi
systemctl disable --now linux-logitech-control.service 2>/dev/null || true
# Fixed installed paths only; user profiles and backups are preserved.
rm -f -- /usr/bin/llc /usr/libexec/llc-service /usr/libexec/llc-session-agent \
    /usr/lib/systemd/system/linux-logitech-control.service /usr/lib/systemd/user/llc-session-agent.service \
    /etc/dbus-1/system.d/au.craznar.LinuxLogitechControl1.conf \
    /usr/share/dbus-1/interfaces/au.craznar.LinuxLogitechControl1.xml \
    /usr/lib/sysusers.d/linux-logitech-control.conf /usr/share/man/man1/llc.1
systemctl daemon-reload
systemctl reload dbus
echo 'Executables and service definitions removed. User configuration, installed documentation and the service account were preserved.'
echo 'In each desktop session, run: systemctl --user disable --now llc-session-agent.service'
