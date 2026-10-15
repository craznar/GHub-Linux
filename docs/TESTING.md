# Testing

No application tests have run. Repository checks cover file layout and documentation links only.

Follow specification section 17. Add deterministic core tests, protocol fixtures, integration failure/recovery tests and native Linux hardware qualification with the corresponding implementation.

Record environment, command, revision, expected behaviour and observed result for each test. Label evidence as automated, simulated or physical. Keep untested, blocked, failed and passed outcomes distinct. Mocks cannot pass hardware gate G0.

Put sanitized fixtures under `tests/fixtures/`; keep unsanitized local captures in the ignored `local-captures/` directory.
