# Known limitations

This engineering build is not the completed application in SPECIFICATION.md.

Implemented: strict configuration/schema, macro compiler and cancellable simulated scheduler, shared ownership, layer/bank/profile transition core, atomic revisioned persistence, read-only USB/interface inventory, explicit bounded event capture, offline CLI/profile tools, passive credential-checked D-Bus backend, session agent and build/install/package scripts.

Outstanding:

- G0: no actual G502 HERO/G815 descriptors, control maps, diverted-mode behaviour or recovery observations exist. Inventory/capture code alone cannot pass it.
- No HID++ transport/feature discovery, diversion, firmware readback or native-default reproduction. Framing/response matching is tested separately.
- No live evdev grab/forward/uinput runtime. Physical bypass, lock/session input transitions, watchdog recovery and unplug recovery remain unqualified.
- DPI/report-rate/lighting operations and hardware-setting configuration entities are unimplemented.
- Qt GUI, macro editor/recorder and timed primary-click revert are deferred by the specification's pre-GUI gate.
- No live macro output. Simulated tests do not establish device/game support.
- Schema migrations, single-profile bundles, complete IPC signals/progress/recording and competing-manager detection remain unfinished.
- Ubuntu 26.04 GNOME/Wayland, physical A-01–A-26, latency and soak qualification remain untested.
- Application license remains undecided. Builds/packages collect third-party license texts.

Next owner action when ready: attach the actual devices and follow HARDWARE-TESTING.md, starting with the read-only inventory. Review the results before implementing/enabling temporary mode writes or active routing.
