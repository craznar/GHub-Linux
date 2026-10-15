# Acceptance register

Recorded 15 October 2026. No application or hardware tests have run. Setup host: Windows. Native Ubuntu release, kernel, GNOME, Qt, firmware and keyboard layout: not inspected.

Hardware gate G0: OPEN, with no physical observations. Every required input control still needs an evidenced route. See specification sections 3.2 and 17 for the normative criteria.

| Case | Criterion | Result | Evidence |
|---|---|---|---|
| A-01 | Fresh install: both devices type/point normally, no mode change or management until enabled. | Untested | None |
| A-02 | Every G502 programmable control uniquely triggers its assigned action; original action is suppressed; mechanical wheel switch excluded with evidence. | Untested | None |
| A-03 | G1–G5 in all three banks execute distinct actions; actual F1–F5 retain independent behaviour. | Untested | None |
| A-04 | Ordinary G815 keys remap correctly, including left/right modifiers, keypad, media and lock LED pass-through. | Untested | None |
| A-05 | Mouse button mapped to held Ctrl permits Ctrl+physical-key use and releases Ctrl exactly when expected. Repeat for Shift, Alt and Meta. | Untested | None |
| A-06 | Physical G815 Ctrl held during a macro remains held after macro completion/cancellation; test overlapping macro owners. | Untested | None |
| A-07 | Once, repeat N, while-held and toggle modes match section 8, including cancellation during waits and chord holds. | Untested | None |
| A-08 | G815 layer activator changes G502 assignments; release/disconnect restores base; held actions release under their captured mappings. | Untested | None |
| A-09 | Profile switching while keys/macros are held produces no stuck output or newly synthesized macro trigger. | Untested | None |
| A-10 | Close/crash GUI: mappings continue; active recording ends without leaving captured input suppressed. | Untested | None |
| A-11 | Emergency physical chord works despite reassigned modifiers/Backspace, active loops and recording; bypass remains latched. | Untested | None |
| A-12 | Unplug either device during held output: its actions stop; the other device remains usable; reconnect does not restart macros. | Untested | None |
| A-13 | At least 20 reconnects per device and 20 suspend/resume cycles preserve mappings, ordinary input and recovery rules. | Untested | None |
| A-14 | Lock/unlock, logout/login and a second-user session transition revoke/restore ownership correctly; no macros at login/lock screen. | Untested | None |
| A-15 | Panic, kill and hang each backend component; measure ordinary input restoration and separately document diverted-control recovery. | Untested | None |
| A-16 | Verified DPI stages/shift/report-rate/settings read back correctly; rejected writes never show success. | Untested | None |
| A-17 | Verified lighting controls work; disconnected/unsupported/write-failed states remain distinct; input continues during failure. | Untested | None |
| A-18 | Save/export/import on a clean user account preserves all software assignments/macros/banks/layers, excluding identity by default. | Untested | None |
| A-19 | Corrupt primary/backup, disk full and interrupted save do not erase last-good active state or silently replace source files. | Untested | None |
| A-20 | Competing manager yields a clear conflict or documented supported configuration; no automatic disabling of third-party services. | Untested | None |
| A-21 | Normal scrolling, horizontal tilt, high-resolution data and mouse motion remain correct without duplication under load. | Untested | None |
| A-22 | Native Wayland and at least one XWayland application receive equivalent physical-key/mouse output. | Untested | None |
| A-23 | With owner-provided installations, test WoW and Diablo IV through their actual Wine/Proton setup: modifier hold, G-key action and a short macro in an appropriate input context. No downloads/accounts assumed. If unavailable, mark game qualification untested, never passed. | Untested | None |
| A-24 | Qt UI works with keyboard-only navigation and 100/150/200% scaling; dangerous click reassignment timed revert succeeds. | Untested | None |
| A-25 | Package upgrade/uninstall preserves configuration and restores usable ordinary input; automatic activation follows the user's setting. | Untested | None |
| A-26 | Performance and soak thresholds in section 14 met with reports. | Untested | None |
