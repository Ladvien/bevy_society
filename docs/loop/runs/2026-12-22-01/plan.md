# Plan: break the sleep jam

## Why this over the runner-up

The previous iteration left 142 of 150 sleep failures surviving a budget that covers the whole village, and named
it the top item. It is my own loose end, it is 37.5% of all remaining failures, and the diagnosis was already half
done.

## What was tried, in order

1. **Scatter destinations (RNG).** Hypothesis: a crowd cannot share one goal point, so ORCA holds the back of the
   queue outside the radius. Result: 456 interrupted. *Not comparable* — it consumes the ChaCha8 stream, so the
   whole run diverges. Retested deterministically.
2. **Scatter destinations (deterministic, golden angle by index).** Comparable, no RNG draw. Result: **worse**,
   484 interrupted, sleep 174. Hypothesis dead.
3. **Steering `stop_radius` 0.5·r → r.** Hypothesis: walkers press toward 0.75 m, jam outside 1.5 m; letting them
   stop at the arrival radius should fire arrival. Result: **byte-identical** output. Dead, and informative.
4. **`interaction_radius` 1.5 → 2.2.** Result: 378 interrupted, but rivals **38 → 90**.
5. **`interaction_radius` 1.5 → 2.6.** Result: 364 interrupted, sleep 136, rivals **82**.
6. **`Home` honours `Place::capacity`** (the inconsistency with `Place(tag)`). Result: **worse**, 440 interrupted,
   rivals 69.
7. **Instrumented velocity** to check the premise: all 150 have a `SocialMover`, and 84 of 150 still have velocity
   above 0.01 m/s. So they are pressing, not idle.

## Why nothing shipped

Seven levers, none reduces the jam, three damage another model. The loop's rule is five failed rounds then log the
blocker and revert; this is round seven. Reverted to a clean tree.

## What the evidence points at

`stop_radius` producing a **byte-identical** simulation is the key negative result. A change to how far walkers push
toward their goal that changes nothing means the wish is not what is being limited — either it is already zero for
these movers, or ORCA's pairwise constraints refuse all motion regardless. That is upstream of every constant this
iteration tried, and it is what the next attempt should instrument: the solver's actual inputs (`pref_vel`,
neighbour count, member radius) for the stuck walkers, rather than another constant.

## Touched

- `docs/loop/runs/2026-12-22-01/` — this record and `evidence/levers.png`.
- **No crate changes.** `git diff` on `src/` is empty.
