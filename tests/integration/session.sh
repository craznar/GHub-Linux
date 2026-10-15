#!/usr/bin/env bash
set -euo pipefail
: "${LLC_TEST_BIN:?}"
: "${DBUS_SESSION_BUS_ADDRESS:?}"
export DBUS_SYSTEM_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS
temp=$(mktemp -d)
export XDG_CONFIG_HOME=$temp/config
service_pid=''
cleanup() { if [[ -n $service_pid ]]; then kill "$service_pid" 2>/dev/null || true; wait "$service_pid" 2>/dev/null || true; fi; rm -rf -- "$temp"; }
trap cleanup EXIT
"$LLC_TEST_BIN/llc-service" > "$temp/service.log" 2>&1 &
service_pid=$!
ready=false
for _ in {1..40}; do
    if "$LLC_TEST_BIN/llc" status --json > "$temp/status.json"; then ready=true; break; fi
    sleep 0.05
done
$ready || { cat "$temp/service.log"; exit 1; }
if command -v gdbus >/dev/null; then
    gdbus introspect --address "$DBUS_SESSION_BUS_ADDRESS" --dest au.craznar.LinuxLogitechControl1 --object-path /au/craznar/LinuxLogitechControl1 --xml > "$temp/interface.xml"
    if [[ -n ${LLC_EXPORT_XML:-} ]]; then cp "$temp/interface.xml" "$LLC_EXPORT_XML"; fi
fi
python3 - "$temp/status.json" <<'PY'
import json,sys
v=json.load(open(sys.argv[1]))
assert v['schema_version']==1
assert v['management_enabled'] is False
assert v['active_revision'] is None
assert v['hardware_gate']=='G0-unverified'
PY
"$LLC_TEST_BIN/llc" config init
path=$("$LLC_TEST_BIN/llc" config path)
"$LLC_TEST_BIN/llc" config validate "$path" --json
"$LLC_TEST_BIN/llc" profile create Integration
"$LLC_TEST_BIN/llc" config export "$temp/export.json"
"$LLC_TEST_BIN/llc" config import "$temp/export.json"
"$LLC_TEST_BIN/llc" probe --json > "$temp/probe.json"
set +e
"$LLC_TEST_BIN/llc" bypass --json > "$temp/denied.json"
denied=$?
"$LLC_TEST_BIN/llc" bank select 4 --json > "$temp/invalid.json"
invalid=$?
"$LLC_TEST_BIN/llc" config export "$temp/export.json" >/dev/null 2>&1
overwrite=$?
set -e
[[ $denied == 3 ]] || { cat "$temp/denied.json"; echo "Expected denial (3), got $denied"; exit 1; }
[[ $invalid == 2 && $overwrite == 6 ]]
"$LLC_TEST_BIN/llc" diagnostics export "$temp/diagnostics.json"
python3 - "$temp" <<'PY'
import json, pathlib, sys
p=pathlib.Path(sys.argv[1])
for name in ['export','probe','denied','invalid','diagnostics']:
    v=json.loads((p/(name+'.json')).read_text())
    assert v['schema_version']==1
assert json.loads((p/'probe.json').read_text())['read_only']
PY
kill "$service_pid"
wait "$service_pid" 2>/dev/null || true
service_pid=''
set +e
"$LLC_TEST_BIN/llc" status --json > "$temp/unavailable.json"
unavailable=$?
set -e
[[ $unavailable == 4 ]]
echo 'Isolated D-Bus/CLI integration passed: passive startup, validation, persistence, denied nonlocal control and service loss.'
