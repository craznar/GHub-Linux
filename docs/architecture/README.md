# Architecture

The workspace follows specification section 4: llc-core (configuration, compiler, ownership/scheduler, persistence), llc-protocol (read-only USB/sysfs inventory and bounded HID++ framing), llc-input (physical safety state and explicit read-only evdev capture), llc-service (passive system D-Bus backend), llc-cli and llc-session-agent.

llc-gui and qml remain reserved paths. The specification requires actual hardware evidence before GUI implementation. It has not been replaced by simulated qualification.

See [ADR 0001](0001-engineering-boundary.md) for toolchain, privilege boundary, lifecycle and deferred routing decisions.
