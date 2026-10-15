# systemd integration

linux-logitech-control.service is the dedicated unprivileged passive backend. It has a private device namespace, no capabilities, read-only system and no home access. llc-session-agent.service is optional and bound to graphical-session.target. Installer enables the system backend; user agent and login management remain separately opt-in. See docs/OPERATIONS.md.
