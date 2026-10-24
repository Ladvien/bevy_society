# Plan: make the harness read the spec

## Why this over the runner-up

Runner-up was `food` at 15% of ticks at zero — a behaviour item. Lost on one sentence: **every behaviour
measurement this loop has made was taken under a configuration the spec does not describe**, so fixing a
need before fixing the harness would be fixing a symptom of the harness.

## Design

`examples/evidence.rs` gains `scenario_spec(name)`, a small hand parser for the one shape the file uses:

```text
  - name: village-evening
    ...
    settings: { seconds_per_day: 64.0, start_hour: 15.0, travel_timeout_hours: 20.0 }
    ticks: 4200
```

It reads `seconds_per_day`, `start_hour`, `travel_timeout_hours` and `ticks`, warns about any other key it
sees in `settings:` (so a new key is visible rather than silently dropped), and falls back to the crate
defaults for a scenario the file does not name — a missing spec should reproduce the crate's behaviour,
not invent one.

Hand-parsed rather than adding a YAML dependency: the file uses exactly one form for the field, and a
dependency for four floats is not worth its build cost.

Three conversions, each forced by a measurement:

- **`ticks:` is scaled by `seconds_per_day / 4.0`**, because the file's tick counts were authored against
  the old four-second day. Without this the village ran **1.03 game-days** instead of 17.
- **`plant_at` scales the same way.** 560 ticks at 64 s/day is 3.3 game-hours; measured, planting that
  early took reach from 0.375 to **0.000** because nobody had gossiped yet.
- **The village scenarios declare a fitted `travel_timeout_hours`.** At 64 s/day the 2-hour default buys
  7.5 m against an 8.2 m village, and `BehaviorPlugin`'s own warning said so.

`tick_secs` stays pinned at `1/64` for every scenario, so a scenario's tick count is its number of updates
whatever its day length. The two are independent, and conflating them is how the drift went unnoticed.

## The summary reads the peak

A planted rumor spreads and is then **forgotten** — `knowledge_capacity` (64) evicts the least-retained,
and a rumor heard at day 2 of a 17-day run has retention `0.995^360 = 0.165` against 150 later ones.
`summary.json` now reports the highest reach the series saw, not the final state.

## Touched

- `examples/evidence.rs` — `Spec`, `scenario_spec`, scaled `settings_for` and `ticks_for` and `plant_at`,
  `Row::known_by`, peak tracking, `summary` takes the peak.
- `docs/loop/scenarios.yaml` — the village scenarios gain `travel_timeout_hours: 20.0`; `village-evening`'s
  observable corrected.
- `tests/scenario_spec.rs` (new) — three tests pinning the contract.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `every_scenario_declares_a_day_length` | every scenario in the file states `seconds_per_day` |
| `the_village_scenarios_use_a_day_a_plausible_activity_can_live_in` | the village scenarios run 64 s/day and declare a fitted timeout |
| `the_starter_pack_still_compiles_with_the_village_additions` | a spec change that broke the merge fails here, not only when someone runs an example |
