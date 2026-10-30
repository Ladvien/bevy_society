# Plan

`Tuning::retaliation_refractory_hours: Option<f32>` (default `None`). `exchange` records `RecentHarm(actor -> (target, until))` when the interaction's deed has `harm < 0`, accepted or refused, after `requires` and before the acceptance draw (no RNG draw moves). `decide_activities` drops such targets from a harmful activity's person targets after `targets_for` (RNG draws unchanged). Host `Interact` is not held back. Tests: `tests/retaliation.rs`. Harness flag `--retaliation-refractory H`.
