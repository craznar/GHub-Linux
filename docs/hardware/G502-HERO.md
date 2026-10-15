# G502 HERO hardware evidence

**Status: not probed; capabilities unverified.**

No physical device was examined during repository setup. The starting USB identifier in specification section 3.1 is upstream reference evidence only.

| Evidence | Result |
|---|---|
| Device identity, firmware and interface inventory | Not observed |
| Descriptors and evdev/hidraw node association | Not captured |
| Control press/release and suppression routes | Not mapped |
| Host/diversion modes and restore behaviour | Not tested |
| DPI, polling and lighting read/write/restore | Not tested |
| Disconnect, service failure and suspend/resume | Not tested |

Follow specification section 3.2. Record exact hardware, firmware, environment, commands, observations and untested cases here. Store sanitized captures under `tests/fixtures/`. Default probing must be read-only; no persistent writes during the initial probe phase.
