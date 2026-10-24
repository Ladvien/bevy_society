# Research: walking lives in a different clock from everything else

## The finding

The log's #1 was "the travel curve's shape". Tried it: three curves on `eat` + `relax` at seed 7 —

| curve | travelling | needs |
|---|---|---|
| `Linear(-0.4, 1.0)` | 57.8% | 0.555 |
| `Linear(-0.25, 1.0)` | 59.0% | 0.546 |
| `Logistic(-6.0, 0.6)` | 53.1% | 0.533 |
| baseline (no `Distance`) | 58.9% | 0.545 |

**All within one point of baseline.** The curve is not the lever.

## The lever is the clock

`Tuning::walk_speed` is **metres per real second** (1.4), applied at the fixed step. Every
`ActivitySpec::duration_hours` is **game hours**. The bridge between them is `seconds_per_day`:

```
game-hours per metre = (1 / walk_speed) / seconds_per_day · 24
```

So a host's **pacing knob silently rescales every walk against every activity**. Measured on
`village-evening`, seed 7, the same 24 people and the same 16.4 game-days, varying only `seconds_per_day`:

| `seconds_per_day` | travelling | need mean | sleep |
|---|---|---|---|
| 64 | **58.9%** | 0.545 | 15.0% |
| 144 | 50.4% | 0.603 | 17.0% |
| 240 (crate default) | 46.8% | 0.621 | 16.5% |
| 480 | **43.9%** | 0.624 | 17.3% |

**Fifteen points of a villager's life**, from a setting documented as *"seconds of real time that make one
game-day"* — how fast you watch, not how much of it is spent walking.

This breaks the crate's own stated invariant: *"Every rate is per game-day and every dt is
`SocietyClock::dt_days`."* Walking is the exception, and nothing says so.

## What believable output looks like

A day where the typical walk is shorter than the typical activity. At the shipped default a 10 m walk is
**0.71 game-hours** against a **1.62 h** mean activity — 44% of an activity spent reaching it, which the
run shows as 46.8% travelling with all needs met (`food` 0.494). At 64 s/day the same walk is **2.68 h**,
1.7× the mean activity, and 58.9% of ticks are travel.

## Why this is not the fix

The coupling is `needs-decision/travel-time-in-two-clocks.md`, open, with a recommendation (move the timeout
to real time) that is a breaking change to the settings surface. **Not decided here.**

What is *not* a decision is the **silence**. The crate already ships two startup warnings on exactly this
principle — `warn_short_travel_reach` (*"the silence is not a decision the crate may defer"*) and
`warn_short_activities` — and neither covers the case where a day's walking outlasts its activities.

## Literature

No new model. The relationship is arithmetic, and the citation is the crate's own invariant. The utility
side (`Signal::Distance`, response curves) is `docs/research/needs-and-utility.md`; the clock is
`docs/loop/needs-decision/travel-time-in-two-clocks.md`.
