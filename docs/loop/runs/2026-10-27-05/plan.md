# Plan
- `Tuning::stall_timeout_hours: Option<f32>` (None), `stall_epsilon: f32` (0.25 m); both design defaults.
- `TravelProgress` component (crate-private, not saved): written by `arrive_and_satisfy` while travelling and only when a timeout is set; read in `end_activities`, which ends the activity exactly as the travel timeout does. Removed on arrival, ending and re-decision.
- No change to `Doing` (public, built by hosts and tests), to messages or to the save format.
- Tests: `tests/stall.rs` (motionless walker ends after the window; slow closing walker never cut; None leaves a motionless walk alone).
