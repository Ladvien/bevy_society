# Plan

- `core.rs`: `Away` (`#[require(Dormant)]`), `AwaySince(f64)`, `Tuning::{missing_cap, missing_days, reunion_fraction}`.
- `absence.rs` (new, registered by `MindPlugin`): `PersonReturned`, `ReturnQueue`, observers (record the start, queue the return, `Deceased` ends the absence), `miss_the_away` (Mind set, after `update_minds`), `announce_returns` (Clock set, before `apply_appraisals`), shared `closeness` (grief now uses it too).
- `mind.rs`: `Emotions::raise_to` (holds a level; `appraise` adds).
- `save.rs`: `VERSION` 3, `PersonSave::away_since_bits`, `upgrade` 2 to 3.
- Tests `tests/absence.rs` (criteria 1 to 6 of the issue plus the zero-length and party edge cases); harness `examples/absence.rs`, `tools/plot_absence.py`.
- Decisions, assumed: see `needs-decision/an-absence-is-felt-not-caused.md`.
