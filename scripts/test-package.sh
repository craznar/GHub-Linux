#!/usr/bin/env bash
# Run only on an explicitly selected disposable/systemd test host.
set -euo pipefail
if [[ $EUID != 0 ]]; then echo 'Run as root on a systemd test host.' >&2; exit 2; fi
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if dpkg-query -W -f='${Status}' linux-logitech-control 2>/dev/null | grep -q 'install ok installed'; then
    echo 'An installation already exists; refusing to replace it during a lifecycle test.' >&2; exit 2
fi
[[ ! -e /usr/bin/llc ]] || { echo 'An existing source installation was found; refusing to replace it.' >&2; exit 2; }
temp=$(mktemp -d)
cleanup() { rm -rf -- "$temp"; }
trap cleanup EXIT
export XDG_CONFIG_HOME="$temp/config"
bash "$root/scripts/install.sh"
systemctl is-active --quiet linux-logitech-control.service
llc status --json > "$temp/source-status.json"
llc config init
config=$(llc config path)
before=$(sha256sum "$config")
bash "$root/scripts/uninstall.sh"
[[ $(sha256sum "$config") == "$before" ]]
package="$root/dist/linux-logitech-control_0.1.0_amd64.deb"
dpkg -i "$package"
systemctl is-active --quiet linux-logitech-control.service
llc status --json > "$temp/package-status.json"
# Reinstall exercises the package upgrade maintainer-script path.
dpkg -i "$package"
systemctl is-active --quiet linux-logitech-control.service
systemd-analyze verify /usr/lib/systemd/system/linux-logitech-control.service /usr/lib/systemd/user/llc-session-agent.service
dpkg --remove linux-logitech-control
[[ $(sha256sum "$config") == "$before" ]]
[[ ! -e /usr/bin/llc ]]
if systemctl is-active --quiet linux-logitech-control.service; then echo 'Backend still active after removal.' >&2; exit 1; fi
python3 - "$temp/source-status.json" "$temp/package-status.json" <<'PY'
import json,sys
for path in sys.argv[1:]:
    v=json.load(open(path))
    assert v['management_enabled'] is False
    assert v['state']=='ready-pass-through'
PY
echo 'Source install/start/remove and Debian install/reinstall/remove passed; configuration preserved.'
