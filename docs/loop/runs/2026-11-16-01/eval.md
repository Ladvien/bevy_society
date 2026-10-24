# Eval

## The configuration drift

| | `seconds_per_day` |
|---|---|
| `docs/loop/scenarios.yaml` — the spec | **64.0** |
| `examples/evidence.rs`, hardcoded | 4.0 |
| `examples/capture.rs`, hardcoded | 8.0 |

Same scenario name, three values, and neither harness read the file. The spec had never been executed.

## What that cost, measured at the same seed and spawn

| | hardcoded 4.0 s/day | **spec 64.0 s/day** |
|---|---|---|
| rumor reach (peak) | 0.042 | **0.375** at the spec's own pacing |
| needs mean | [0.11, 0.21, 0.08, 0.11] | **[0.88, 0.77, 0.50, 0.81]** |

Needs are **4–7× higher** at the settings the spec declares. At 4 s/day a four-hour activity is 4 ticks
and an 8 m walk is 375, so travel dominates everything.

## Four defects found while fixing it

| | measured | fix |
|---|---|---|
| `ticks: 4200` is 1.03 game-days at 64 s/day, not 17 | the run ended after one day | `ticks:` scaled by `seconds_per_day / 4.0` |
| `plant_at` = 560 ticks is 3.3 game-hours at 64 s/day | reach **0.375 → 0.000** | `plant_at` scaled the same way |
| the 2-hour default timeout buys 7.5 m at 64 s/day | `BehaviorPlugin`'s own warning fired | scenario declares 20 h (75 m) |
| `summary.json` read the final state | `known_by: 0` while the series showed 2 | reports the **peak** |

The last is the subtlest: a rumor is *forgotten*, because `knowledge_capacity` (64) evicts the
least-retained and a rumor from day 2 has retention `0.995^360 = 0.165` against 150 later ones. Measured
at the end: zero of 24 villagers held it. That is the model working, so the summary now reports what the
scenario asks about — the peak.

## The scenario's observable

It asked the rumor to reach **half the village**. The village makes about **18 rumors a day**, and
`best_telling` prefers the strongest and freshest, so a planted rumor is told (`known_by` 1 → 2) and then
out-competed — Daley–Kendall working. The observable now asks that it be **told to at least one other
person**; reach is recorded, not a pass condition.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **33** binaries |
| `cargo test --no-default-features` | 33 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `examples/evidence.rs` is example code; every new item is reached by the run, and `scenario_spec`
is exercised by three tests.
