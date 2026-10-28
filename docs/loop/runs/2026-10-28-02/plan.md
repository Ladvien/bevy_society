# Plan
- `Tuning::failed_place_memory_hours: Option<f32>` (None).
- Crate-private `FailedPlaces(Vec<(Entity, until_day)>)` component, written in `end_activities` for an unarrived, uncompleted, non-dormant `Place` walk; read in `decide_activities` as a `retain` after `targets_for`, so RNG draws are identical with or without memory.
- Tests: `tests/failed_place.rs`.
