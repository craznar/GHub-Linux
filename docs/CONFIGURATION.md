# Configuration

Version 1 is strict UTF-8 JSON, maximum 4 MiB. Unknown fields and future versions are rejected. See the [generated schema](../schemas/config.schema.json) and [example](../examples/config.json). Rust validation additionally enforces references, ranges, balance and execution limits.

`llc config init` creates a passive default with one UUID profile, no bindings/macros and login activation off. Storage is `$XDG_CONFIG_HOME/linux-logitech-control/config.json` or `$HOME/.config/linux-logitech-control/config.json`; `llc config path` prints it. XDG_CONFIG_HOME must be absolute; a relative value uses the fallback.

Profiles contain UUID/name, default bank and semantic bindings. Bindings select control, base/secondary layer, G-key bank, impulse semantics and typed action. Missing secondary assignments inherit base; missing base assignments pass through. Physical key codes use Linux evdev numbering. Device serial/USB-port selectors are optional and removed by default export; selectors are not connected to live routing yet.

Use `llc profile create/rename/duplicate/delete`, `llc profiles list`, validation and import/export. Saving uses a private lock, expected revision, same-directory temporary file, fsync and atomic rename. Files are created with mode 0600 and new directories with 0700. A validated previous revision is retained as config.json.bak. Corrupt primary data is never silently overwritten through backup recovery or saving.

Import saves without changing runtime state. Activation saves the selection, stages validated bytes over D-Bus and requests management. That final request currently fails with exit 5 because G0 is unverified. Staged and active revisions are distinct.

Not implemented: hardware-setting entities, schema migrations, a single-profile bundle format, live device matching, GUI editor and macro recorder. Current export contains the full validated configuration with device identity removed. Configurations cannot execute processes or scripts.
