# Manual hardware evidence

G0 is unverified. Attach the wired G502 HERO and G815 on native Linux, then start with this single read-only inventory action:

```sh
llc probe --json > inventory.json
```

Review the report before further investigation. It includes USB topology, interface nodes, descriptors when readable and errors, omitting serial values. USB bcdDevice is not presented as verified firmware. Root is normally unnecessary for sysfs inventory; descriptor access varies by host.

Later, explicitly capture one model's normal physical events:

```sh
sudo llc probe events g815 15 g815-events.json
```

Capture is bounded to 30 seconds and 20000 events. It records key/button/wheel activity and interface indices, which still reaches applications. Use a blank test area and press only the controls under investigation. There are no grabs or device writes. Output files are private and never overwritten. Opened evdev and USB-parent identities are rechecked. Missing or ambiguous models fail. This event recorder is not a macro recorder.

Keep original captures outside Git under ignored local-captures/. Review/sanitize before adding fixtures. Record exact hardware, firmware, layout, kernel/desktop, commands, physical labels, press/release observations and untested cases in the hardware reports.

Normal-mode capture establishes only part of G0. Feature discovery, unique G-key delivery/suppression, default reproduction, temporary settings with readback/restore, reconnect/failure/suspend and the rest of specification section 3.2 require an evidence-based protocol probe. --allow-transient-changes currently fails explicitly: those operations are not implemented. No persistent writes are allowed in this phase.
