#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
stage=$(mktemp -d)
chmod 755 "$stage"
trap 'rm -rf -- "$stage"' EXIT
version=0.1.0
arch=$(dpkg --print-architecture)
for binary in llc llc-service llc-session-agent; do [[ -x $target/release/$binary ]] || { echo 'Run scripts/build.sh first.' >&2; exit 2; }; done
[[ -d $target/third-party ]] || { echo 'Dependency notices are missing; run scripts/build.sh first.' >&2; exit 2; }
install -Dm755 "$target/release/llc" "$stage/usr/bin/llc"
install -Dm755 "$target/release/llc-service" "$stage/usr/libexec/llc-service"
install -Dm755 "$target/release/llc-session-agent" "$stage/usr/libexec/llc-session-agent"
install -Dm644 "$root/packaging/systemd/linux-logitech-control.service" "$stage/usr/lib/systemd/system/linux-logitech-control.service"
install -Dm644 "$root/packaging/systemd/llc-session-agent.service" "$stage/usr/lib/systemd/user/llc-session-agent.service"
install -Dm644 "$root/packaging/sysusers/linux-logitech-control.conf" "$stage/usr/lib/sysusers.d/linux-logitech-control.conf"
install -Dm644 "$root/packaging/dbus/au.craznar.LinuxLogitechControl1.conf" "$stage/etc/dbus-1/system.d/au.craznar.LinuxLogitechControl1.conf"
install -Dm644 "$root/packaging/dbus/au.craznar.LinuxLogitechControl1.xml" "$stage/usr/share/dbus-1/interfaces/au.craznar.LinuxLogitechControl1.xml"
install -Dm644 "$root/schemas/config.schema.json" "$stage/usr/share/linux-logitech-control/config.schema.json"
install -Dm644 "$root/packaging/man/llc.1" "$stage/usr/share/man/man1/llc.1"
install -d "$stage/usr/share/doc/linux-logitech-control" "$stage/DEBIAN" "$root/dist"
cp -R "$root/docs/." "$stage/usr/share/doc/linux-logitech-control/"
cp -R "$target/third-party" "$stage/usr/share/doc/linux-logitech-control/"
install -m644 "$root/README.md" "$root/SPECIFICATION.md" "$stage/usr/share/doc/linux-logitech-control/"
sed -e "s/@VERSION@/$version/" -e "s/@ARCH@/$arch/" "$root/packaging/debian/control" > "$stage/DEBIAN/control"
for script in postinst prerm postrm; do install -m755 "$root/packaging/debian/$script" "$stage/DEBIAN/$script"; done
find "$stage/usr/share" -type d -exec chmod 755 {} +
find "$stage/usr/share" -type f -exec chmod 644 {} +
dpkg-deb --root-owner-group --build "$stage" "$root/dist/linux-logitech-control_${version}_${arch}.deb"
echo 'Engineering package only. Application licensing remains undecided; no production hardware qualification is claimed.'
