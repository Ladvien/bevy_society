# Performance: a label is a latitude

The change is one comparison inside `Tie::refresh_label`, which already ran once per tie per tick. It
adds no allocation, no draw and no iteration; `classify_from` computes the promotion classification it
already needed and then, only at the low end, compares familiarity against a second bar.

Measured with `examples/benchmark --reps 3` on the M1 Max, before and after, release build.

| persons | before, fastest window (ms/tick) | after, fastest window (ms/tick) | spread after |
|---|---|---|---|
| 100 | 1.215 | **1.125** | 14.9% |
| 1 000 | 30.593 | **31.954** | 37.9% |
| 5 000 | — | **527.471** | 14.6% |

**The change is inside the noise.** At 100 people the after figure is 7% *lower* than the before figure
and at 1 000 it is 4% higher, which is the shape of a shared machine rather than of 109 lines of
comparison: the 1 000-person run's own spread was 37.9%, the "before" spread 42.3%. The honest statement
is that a per-tie comparison against a constant costs nothing measurable beside the neighbour scan this
tick already does.

The 5 000-person figure is new and is **not** comparable to `docs/guide/concepts.md`'s table, which stops
at 2 000 because the earlier tool could not be trusted above it. It is recorded here because the number
is now available, not as a claim about the guide's curve: **527 ms per society tick** at 5 000 people,
`ms_per_million_pairs` 47.4 against 79.4 at 1 000, i.e. the same sub-linear shape the guide describes.

Nothing was slowed by this change, and no scenario it did not target was touched: the `village-evening`
runs it was measured with take the same wall-clock as before (67 200 ticks in about 3 minutes).
