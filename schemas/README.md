# Schemas

config.schema.json is generated from the Rust configuration types using llc config schema; build.sh detects drift. Rust validation additionally enforces semantic limits, references and balances. cli-output.schema.json defines the common versioned JSON envelope; individual payloads are documented by llc --help and the corresponding Rust types.
