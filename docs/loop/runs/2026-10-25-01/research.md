# Research: what a society needs to look like a society

This iteration began from the log's top item — the hero capture — and ended up asking a
more basic question: **what does a run have to contain for believable behaviour to be
observable at all?** The answer is measured, not assumed.

## The two things that were wrong, and the evidence

Nothing here needed a new model. Both were the crate and its scenarios disagreeing with
the clock.

### 1. A village cannot form ties in one evening

Same code, same seed, same village, only the run length differs:

| Scenario | People | Game-days | Friendships | Rivalries | Rumor reach |
|---|---|---|---|---|---|
| `village-evening`, 900 ticks | 24 | 4.14 | **0** | **0** | 0.167 |
| `village-evening`, 4200 ticks | 24 | 17.03 | **125** | 0 | 0.500 |
| `rumor-crosses-a-cluster`, 1200 ticks | 12 | 19.07 | 19 | **11** | 0.417 |

The middle row is the same `village.society.ron`, the same spawn layout and the same seed
as the first — only four times longer. Ties were never broken; they needed game-days.

The arithmetic says why. `TieLabel::classify` calls a tie a Friend at
`affinity ≥ 0.4 && familiarity ≥ 0.3`. Familiarity climbs fast
(`exposure_per_day = 2.0`), but affinity moves at `mere_exposure_per_day = 0.05 · (1 − |a|)`
— about 0.2 over four days — while `affinity_relax_per_day = 0.01` pulls the other way.
Measured at the end of the 4-day run, every tie sat at `affinity 0.07–0.13` with
`familiarity 0.82`: **uniformly neutral acquaintances**, which is exactly what those
numbers predict.

### 2. Walking and its timeout are in different clocks

`SocialMover::speed` is metres **per real second** and steering runs every fixed step
(`1/64 s`), so a mover advances `1.4 / 64 ≈ 0.022 m` per tick. `travel_timeout_hours` is
**game hours**.

```
a 6 m walk needs      274 ticks
a 20 m walk needs     914 ticks
the 2-hour timeout     21 ticks  (at seconds_per_day = 4.0)
```

At the capture's original `seconds_per_day = 64.0` the mismatch was worse still. Nobody
could reach anything: the first evidence run shows **37 interrupted activity endings
against 11 completed**, and all four needs pinned at 0.000 for hundreds of ticks because
`satisfies` only pays out while *arrived*.

The coupling is a design decision rather than a bug — both numbers are individually
defensible — so it is written up in
`docs/loop/needs-decision/travel-time-in-two-clocks.md` with three options and a
recommendation (make the timeout real-time in a later iteration, as a deliberate breaking
change).

This iteration fixes the *scenario* instead: a 2.5 m village grid and a 40-hour timeout,
which puts a neighbour walk at ~110 ticks of a 4200-tick run.

## What the corpus contributed

`distill_search` was run for the two model gaps the log listed (`Signal::Field`,
`ActivitySpec::barks`) and for the stigmergy and crowd-field literature behind them. The
strongest hit was Moussaïd, Helbing & Theraulaz 2011, *How simple rules determine pedestrian
behavior and crowd disasters* (PNAS, doc `10.1073_pnas.1016507108`, score 0.571) — a
heuristic model of crowd navigation by "distance of obstructions in candidate lines of
sight".

**It could not be used.** The PDF conversion stripped every equation: pages 4–5 render the
displayed maths as empty `$$` blocks, so the text describes the two heuristics in prose and
gives no formula, no parameter and no rate. Implementing a `Signal::Field` from a prose
description would have meant inventing the numbers, which is the one thing this crate does
not do.

That is why this iteration is about run length and clocks rather than about fields. The
field gap stays open and stays on the log's top five.

## Verdict

No new model. Two measured scenario/clock defects, one design decision escalated, and the
first evidence artefacts the loop has ever produced: narratives, per-tick series and
summaries under `docs/loop/runs/2026-10-25-01/evidence/`.
