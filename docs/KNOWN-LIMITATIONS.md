# Known limitations and outstanding work

Repository setup is complete locally; application implementation has not started.

- No probe, configuration engine, HID++ transport, input routing, service, session agent, CLI or GUI exists.
- No workspace manifest, toolchain pin, dependency lockfile, CI or installable package exists yet.
- No native Linux environment, firmware, descriptors, regional keyboard layout or physical controls have been examined in this setup session.
- Gate G0 remains open: required control routes have no physical evidence.
- All application acceptance cases remain untested.
- Rust/Qt dependency versions and the application license remain undecided.

Next engineering task: initialize the pinned Rust workspace and implement the default read-only hardware probe described in specification section 3.2. Hardware qualification requires access to the actual wired G502 HERO and G815 on the native target.
