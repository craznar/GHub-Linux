# Macros

Implemented in llc-core: typed press/release, tap/chord tap, wait, scroll and nested finite repeat; once, Repeat N, while-held and toggle modes; ignore/restart retrigger; shared ownership, deterministic scheduler ordering and cancellation cleanup. Verification uses simulated events only.

JSON operation tags are down, up, tap, wait, scroll and repeat. Output distinguishes Linux key and button codes. Tap duration is explicit; the example stores 30 ms. See examples/config.json and schemas/config.schema.json.

Limits are centralized in llc-core/src/lib.rs: 4096 stored operations, depth 4, repeat 1–10000, delay/hold 1–600000 ms, 100000 expanded steps, 60-minute predicted duration, 32 instances, 2000 generated events/second, and iteration interval at least 10 ms. Finite executions must balance owned presses/releases. Cleanup releases are exempt from throttling.

The scheduler accepts elapsed monotonic milliseconds from its caller and never sleeps in input handling. It releases only its own output ownership, in reverse acquisition order. Tapping an output held by another owner preserves that hold and may produce no additional application keypress.

```sh
llc macro simulate examples/config.json 00000000-0000-4000-8000-000000000002 --json
```

Simulation runs one finite iteration against a virtual clock and prints the trace. It opens no input/output devices. Global playback, recording, GUI editing and physical Wayland output remain unavailable.
