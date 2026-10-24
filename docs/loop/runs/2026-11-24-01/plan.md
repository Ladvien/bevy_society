# Plan: warn when a day's walking outlasts its activities

## Why this over the runner-up

Runner-up was the travel curve's shape (the log's #1). Lost on measurement: **three curve shapes land within
one point of baseline** (57.8 / 59.0 / 53.1 against 58.9). The clock coupling is **15 points** and silent.
Fixing the curve would have been tuning a 1-point lever while a 15-point one went unmentioned.

## Design

`warn_walk_outlasts_activity(defs, settings)`, called from `BehaviorPlugin::finish` beside the two existing
checks. It computes:

```
walk_hours = MIN_WALK_METRES / 1.4 / seconds_per_day · 24
```

and warns when that exceeds the **mean** `duration_hours`.

**Mean, not median**, and the choice is measured. The starter pack's median is 0.5 h, which a 240-second
day's 0.71 h walk already exceeds — a warning that fires on the crate's own healthy default is worse than
none. Over the mean it separates exactly the two cases that matter:

| day length | 10 m walk | mean activity | verdict | measured travelling |
|---|---|---|---|---|
| 64 s | **2.68 h** | 1.62 h | **warns** | 58.9% |
| 240 s | 0.71 h | 1.62 h | quiet | 46.8% |

It names the setting, the arithmetic, and the open decision, and offers three ways out (longer activities,
higher `seconds_per_day`, closer places) rather than telling the host its layout is wrong — `finish` cannot
see the layout.

`1.4 m/s` is named in the function for the same reason `warn_short_travel_reach` names it: the check speaks
for the default a host gets and cannot see a host's own `SocialMover::speed`.

## Touched

- `src/behavior.rs` — the warning and its call site.
- `tests/warnings.rs` — assertion from the capturing app, plus the silence-at-default case.
- `examples/evidence.rs` — a `--seconds-per-day` override, so this comparison is reproducible.
- `CHANGELOG.md`, `docs/loop/log.md`, `docs/loop/needs-decision/travel-time-in-two-clocks.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `every_startup_warning_is_asserted_from_one_app` | the warning **fires** at 4 s/day, names `seconds_per_day`, and points at the decision |
| `the_starter_pack_warns_about_nothing` | it is **silent** at the 240 s default (0.71 h ≤ 1.62 h) and **fires** at the village's 64 s (2.68 h > 1.62 h) |
