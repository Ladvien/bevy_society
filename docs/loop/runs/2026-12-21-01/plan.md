# Plan: name the layout a timeout cannot reach

## Why this over the runner-up

The top item was a `tests/`-reachable harness builder — a refactor that would help *future* iterations. But the
breakdown showed `sleep` is now **37.5% of all failures** and every one of them is a walk that ran out of
`travel_timeout_hours`. The crate already computes reachable distance; it just cannot see the layout that would
make the comparison meaningful. Closing that is a few dozen lines and it makes the crate able to answer a
question a host actually asks.

## Change

`warn_unreachable_places`, a `Startup` system in `BehaviorPlugin`, which:

- computes reachable metres from `tick_secs`, `seconds_per_day` and `travel_timeout_hours`
  (`speed · seconds_per_day · timeout_hours / 24`);
- measures every pairwise distance between `Place` entities (they all carry `Transform`);
- warns with the **farthest pair by name**, the distance, and the `travel_timeout_hours` that would cover it.

Skipped when `Locomotion::Host` (the host walks people itself) or when there is no timeout — the same guards
`warn_short_travel_reach` already uses, for the same reasons.

## Touched

- `src/behavior.rs` — the system and its registration in `finish()`.
- `tests/unreachable_places.rs` — one test, both halves (see below).
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## Measured

| | |
|---|---|
| warning on `village-evening` | `'market' -> 'workshop'` at **16.4 m**, needs **4.4 h**, has 4 |
| interruptions, timeout 4 h → 5 h | 400 → **312** |
| `sleep` failures, 4 h → 5 h | 150 → **142** |

**The honest limit:** eight of the 150 sleep failures are this problem. The other 142 persist at an 18.6 m budget
and are a separate defect, recorded and not claimed.

## Verification

| check | result |
|---|---|
| warning fires / stays quiet | both halves asserted in one test |
| bite check (system disabled) | the test **fails**, and prints the old warning as the only output |
| `cargo test` both feature sets | **60** binaries |

## Why one test, not two

`tracing` permits a single global subscriber per process. A second test in the same binary runs with no sink and
reads an empty string — it would pass or fail for reasons unrelated to the code. Both cases therefore live in one
test, in order, with that stated in the file.
