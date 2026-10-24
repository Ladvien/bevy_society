# Plan: recover the clamp's cost, find what it really costs

## Why this over the runner-up

Item 1 is the clamp decision; item 2 is recovering its cost, which is what would let the decision be made. That is
the actionable one.

## What was done

1. **Derived the mood loss exactly.** `compliment` gates on `Affinity Logistic(8.0, 0.7)`, which scores **0.057**
   at the clamp ceiling against **0.673** unclamped — a 12× drop. The cost was a curve/gate mismatch, not a
   missing rate.
2. **Repaired it** by moving the gate to `Logistic(16.0, 0.25)`; mood recovers to **-0.045**, better than the
   -0.054 baseline.
3. **Found the bar had to move too** — `friend_affinity` 0.4 is unreachable under a 0.35 ceiling, which is why
   the first repairs gave **0 friends**. At 0.30 with a trust second input: **16 friends / 10 rivals / -0.045**.
4. **Ran the whole suite** and found the real cost.

## Measured

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | -0.054 |
| clamp, gate unchanged | 0 | 10 | -0.078 |
| clamp + gate 0.25 | 0 | 10 | **-0.045** |
| clamp + gate 0.25 + bar 0.30 + trust 0.5 | **16** | **10** | **-0.045** |

And the failure the suite found:

| | before | with the clamp |
|---|---|---|
| opinion spread, 12 000 ticks | **53% fall** | **25% fall** |

`opinion-converges-but-not-entirely` asserts at least a third.

## Why it did not ship

The clamp is good on the axes it was aimed at — rivals a third, mood better than baseline — and it **re-times
the opinion model**, because Deffuant influence is weighted by the same `affinity` the clamp bounds. That is a
coupling across three models and a scenario's tick count, not a number to tune in passing.

## Touched

- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` — the mechanism, the repair table, the
  opinion failure, and three options.
- `CHANGELOG.md`, `docs/loop/log.md`.
- **Reverted**: `src/behavior.rs`, `src/core.rs`, `src/relationships.rs`, `assets/starter.society.ron`, and the
  two test call sites.

## New `Tuning` fields

None shipped.
