# Operations

This passive engineering backend cannot grab devices, inject input, change HID modes or flash firmware. Management enable fails explicitly while G0 is unverified. There is no usable remapping GUI yet.

## Install and startup

Build as an ordinary user, then:

```sh
sudo ./scripts/install.sh --install-deps
```

The optional flag installs D-Bus, systemd, udev and kmod utilities through apt. Installation registers the dedicated linux-logitech-control account, CLI in /usr/bin, backend/agent in /usr/libexec, system/user units, D-Bus policy/introspection, schemas, man page, documentation and dependency notices. The system service starts passively at boot. --no-start enables it without starting immediately.

Use `llc status --json`, `systemctl status linux-logitech-control.service` and `journalctl -u linux-logitech-control.service`. The service has a private device namespace and no input-device privileges. Its unreadable-node status is not proof of a host permission failure; run the explicit CLI probe in the host namespace.

The optional ordinary-user agent has no window:

```sh
systemctl --user daemon-reload
systemctl --user enable --now llc-session-agent.service
```

Do not run these user commands with sudo. The desktop must manage graphical-session.target; qualify that integration on Ubuntu GNOME. `llc login enable|disable` separately saves the opt-in management setting. It cannot bypass G0.

## Authorization and recovery

Mutations require bus-provided credentials plus an active, unlocked, local graphical user on seat0 from logind. Remote SSH, inactive sessions, greeters and other seats are refused. Runtime ownership belongs to the session, not a particular CLI process. Session loss clears it; the agent retries after unlock/backend restart without restarting macros/recording.

Authorized `llc bypass` latches bypass; `llc resume` clears it. Physical emergency interception currently exists only as a tested state primitive, not a working device route. Login-screen input is unchanged.

If primary configuration is corrupt, validate config.json.bak and explicitly restore a copy after preserving the corrupt source. No automatic repair overwrites it. Export and diagnostics refuse existing output files. No competing manager is killed or reconfigured. Mode restoration, active-input fault recovery and manager-conflict detection await qualified adapters.

## Remove

For a source installation, disable the agent as the desktop user: `systemctl --user disable --now llc-session-agent.service`. Then run `sudo ./scripts/uninstall.sh`. Executables and service definitions are removed; user profiles/backups, documentation and service account remain.

For a Debian installation use `sudo apt remove linux-logitech-control`; home configuration is preserved even on purge. Run `systemctl --user daemon-reload` after removing user units.
