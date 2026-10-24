# Plan: give the social need a rate, and find out which low needs are actually wrong

## Why this

The log's top-but-one item was "mean need level", recorded last iteration from arithmetic
(`rest` decays 0.7/day, sleep restores at most 1.2/night) rather than from measurement. The
harness now exists, so the hypothesis could be tested per need instead of reasoned about.

Measured first, before touching anything: all four needs sat below 0.35 for 94–96% of ticks.
A cycling need spends its time spread across its range, so all-four-low is structural.

## Design

One change, in the authored data rather than the engine:

- `chat` gains `satisfies: [("social", 1.2)]` — the same rate `eat` pays food.
- `gossip` gains `satisfies: [("social", 1.0)]` — slightly less, because a telling is shorter
  and it also moves reputation.

Rationale: `social` decays at 1.0/day and was the only need whose entire supply was one-shot
interaction effects (+0.2 per accepted chat), while the two other satisfied needs both paid
per game-hour. Time spent talking counted for nothing.

## What was deliberately not changed

`food` and `fun`. Their means are low and that is correct: `eat`'s consideration is
`Need(food) Logistic(-10, 0.35)`, so a villager eats when hungry. Measured, the curve scores
0.0015 at a full need and 0.50 at 0.35. Raising `food`'s rate would be tuning a correct
logistic to flatter a summary statistic, which is the thing this loop exists not to do.

## Also in this iteration

The evidence harness starts villagers within `interaction_radius` of their faction's place.
At this day length a walk is 160 ticks and a 0.3 game-hour chat is 3, so arrival never
preceded the end of an activity. That is the clock mismatch escalated last iteration; the
scenario works around it, and the decision stays open.

## Touched

- `assets/starter.society.ron` (two `satisfies` lines),
- `examples/evidence.rs`, `examples/common/scene.rs` (spawn positions),
- `CHANGELOG.md`, `docs/loop/log.md`, and this run's directory.

No `src/` behaviour. No new `Tuning` fields.
