# Plan: name a timeout that cannot reach the places

## Why this over the runner-up

Runner-up was the `Place`/`Person` completion asymmetry (78 of 108 `eat`s interrupted against 0 of 152
socials). Lost on one sentence: **the asymmetry's first cause is the timeout, and the crate can already
prove the timeout is too short** — so the fix that helps every host, not just this project's village, is
to say so. The asymmetry is now the top item with numbers attached.

## Design

`BehaviorPlugin::finish` gains `warn_short_travel_reach`, beside the two warnings already there. It
computes:

$$\text{metres a timeout buys} = \text{speed} \times \text{tick\_secs} \times \frac{\text{hours}/24}{\text{tick\_secs}/\text{seconds\_per\_day}}$$

and warns when that is under `MIN_WALK_METRES = 10.0`.

Two choices worth stating:

- **10 m, not 0.** A warning that fires on every marginal layout is one hosts learn to ignore. 10 m is
  shorter than any village that is a village, so it fires only when the configuration is certainly wrong.
- **It speaks for the default speed.** A host that sets its own `SocialMover::speed` is beyond what
  `finish` can see — `finish` has the settings and the definitions, not the entities. The doc comment
  says so rather than implying the check is complete.

## The harness

`examples/evidence.rs` set `travel_timeout_hours = 40.0` with a comment reasoning about "the 6 m walk this
layout asks for (274 ticks)". The layout's diagonal is **13 m**, measured at 527 ticks. Corrected to
**55 game-hours** (587 ticks), with the measured numbers in the comment.

## Touched

- `src/behavior.rs` — `warn_short_travel_reach`, `MIN_WALK_METRES`.
- `tests/unreachable.rs` (new) — two tests.
- `tests/warnings.rs` — the single capturing app now asserts the new warning too.
- `examples/evidence.rs` — 40 → 55 hours, with the measurement in the comment.
- `docs/loop/scenarios.yaml` — `need-cycle`'s blocker note corrected and the wrong one superseded.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. The default `travel_timeout_hours` is unchanged at 2.0, which at the default `seconds_per_day`
of 240 buys 28 m and is correct.

## Tests

| Test | What it proves |
|---|---|
| `a_place_beyond_reach_is_named_at_startup` | a 40-hour timeout at a four-second day warns, and the warning names `travel_timeout_hours` |
| `a_place_within_reach_is_not_warned_about` | a 240-second day does not warn |
| `every_startup_warning_is_asserted_from_one_app` | the new warning joins the other two in the one capturing app |

Written **red first**: the warning did not exist.
