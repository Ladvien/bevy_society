# Plan: fit the travel timeout to the layout

## Why this over the runner-up

Runner-up was the `Doing`-while-travelling decision from last iteration. Lost on one sentence: **that
decision is the operator's, but the scale of the problem is a settings value nobody chose deliberately** —
20 game-hours is 74.7 m of reach in an 8.2 m village, and it makes a stuck walker reported asleep for two
and a half times the sleep.

## Design

`docs/loop/scenarios.yaml`:
- `village-evening` and `deed-draws-a-response`: `travel_timeout_hours` **20 → 4**. The village's longest
  walk is 2.2 game-hours and 4 buys 14.9 m.
- `rumor-crosses-a-cluster`: **20 → 12**, because its two groups are 40 m apart and a crossing walk is
  10.7 game-hours.

`examples/evidence.rs`:
- `Spec` gains `contact_radius`, read and applied. It was declared and silently ignored.
- `plant_at` is hoisted out of the tick loop. It parses `scenarios.yaml`, so calling it per tick parsed
  the file 67 200 times and emitted **57 000 warnings**.

## What it does

| | 20 h | **4 h** |
|---|---|---|
| reported asleep | 56.8% | **36.4%** |
| reported working | 14.8% | **21.5%** |
| total need mean | 0.322 | **0.526** |

Holds at seed 42 (36.2%, 0.511). `evidence/timeout-fitted.png`.

## What it exposed

`rumor-crosses-a-cluster` reaches **0.333** at 12 h and **0.167** at 20 — both below its 40% observable.
The **0.583** recorded earlier was before `contact_radius` was applied, so it was a different run. The
scenario goes back to **`pending`** with the measurement and the design question written into its note:
two groups 40 m apart need a 10.7 game-hour walk.

## Touched

- `docs/loop/scenarios.yaml` — three timeouts, the cluster's status and note.
- `examples/evidence.rs` — `Spec::contact_radius`, the hoisted `plant_at`.
- `tests/scenario_spec.rs` — asserts the cluster states a `contact_radius`.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `the_village_scenarios_use_a_day_a_plausible_activity_can_live_in` | the village scenarios state a timeout; the cluster also states a `contact_radius` |
| the other three in that file | unchanged |

`tests/travel.rs` stays ignored and failing: the labelling defect is untouched by this iteration, and the
decision file still stands.
