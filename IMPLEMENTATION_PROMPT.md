# Codex implementation request

Implement the application defined in [SPECIFICATION.md](SPECIFICATION.md). Treat that document as the functional, architectural and acceptance baseline. Use Rust for the backend/core and Qt 6/QML for the GUI. The primary platform is Ubuntu 26.04 LTS, x86-64, GNOME/Wayland. Initial hardware is the wired Logitech G502 HERO and G815.

Read repository instructions first. Inspect the available build environment, Linux access and attached hardware. Follow the specification's phased implementation, starting with the hardware probe and input/recovery foundation. Record architecture decisions and actual test evidence in the prescribed documentation.

Proceed autonomously with ordinary implementation choices. Ask only for missing information that materially blocks dependent work. If Linux or physical devices are unavailable, implement and test independent components and document exactly which hardware gates remain blocked. Never fabricate device captures, capabilities, successful tests or acceptance results.

Keep ordinary typing and mouse operation usable throughout development. Do not enable intrusive device probing, persistent device writes, broad input permissions or competing-service modifications automatically. Follow the probe, authorization and recovery rules in the specification.

Implement real behaviour, not placeholder GUI actions. Keep the repository buildable at phase boundaries, run appropriate checks and maintain requirement traceability. Distinguish simulated results from actual device qualification. Do not mark the project finished until its definition of done is satisfied; if blocked, state the completed work, precise outstanding requirement and the single next action needed from the owner.

Do not expand into additional device models, firmware updates, onboard macro storage, cloud features or automatic application profiles before the required first release is complete.
