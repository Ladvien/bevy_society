# Review

## Pass 1 — Behavior: does the feature now do what its documentation says?

**GREAT.** The README lists social fields; the `fields` feature is on by default; the definitions file
accepts `Field` and `Gradient`. All three now hold together.

| | before | after |
|---|---|---|
| field read by the decision | constant `0.0` | **0.120**, the chooser's real cell |
| gradient target | none | `Some(Vec2(-2.0, 0.0))` |
| a `Field`-only activity, 2 400 ticks | **0** | **6** |

The test is the minimal case the decision file asked for: one chooser, one activity, one consideration.

## Pass 2 — Zoom: read the evidence closely

**GREAT**, and the plot is the honest shape. `evidence/field-wiring.png`:

- Left: the decision's reading of the field was a **flat zero** while the grid itself had always held a real
  value. That is the whole defect in one pair of bars.
- Right: dead, then live.

Zoomed beyond the plot: the activity is chosen **6** times in 2 400 ticks, not 400 — because `idle` holds
for 640 ticks and the chooser must first fail, wait out the fallback, and only then re-decide. Six is the
right order of magnitude for 3.75 game-hours, and I checked that arithmetic rather than accepting the number.

No saturation, no clipping, no drift: the field value is a real cell reading, not a constant pushed to 0 or 1.

## Pass 3 — Fresh eyes: a different seed and a different feature set

**GREAT.** Two configurations rather than two seeds, because the change is feature-gated:

- **`fields` on (default):** the activity is live; the startup warning is **silent**, which
  `tests/warnings.rs` now asserts.
- **`fields` off:** the definitions file **rejects** a `Field` input or `Gradient` target at parse time, so a
  host cannot write one that silently does nothing. That is a stronger contract than the warning, and the
  warning remains for a build that somehow has one.

And with the starter pack, both builds are bit-identical to before: 67 199 of 67 199 rows.

The narrative to a host:

> Social fields work now. An activity that reads `Field(Crowd)` is scored on the crowd at the person's own
> cell, and one that targets `Gradient(Crowd, true)` walks toward where the crowd is thicker. Before, both
> were accepted by the definitions file and could never be chosen — the crate knew, warned at startup, and
> left you to write the feature yourself.

## What I got wrong

Twice I concluded the feature was broken when the *test* was: a crowd in one cell has no slope, and 400 ticks
is less than one 1-hour fallback. The decision file had already recorded two failed attempts; the reason they
failed was in the tests. Recorded because "the feature is dead" and "my probe is too short" look identical
from a red assertion.

## Verdict

Three GREAT. Shipped.
