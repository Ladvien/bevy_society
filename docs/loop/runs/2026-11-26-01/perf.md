# Perf

Hardware: M1 Max, release, `examples/benchmark`, 60 ticks after settling, starter pack.

## Before / after

| persons | ms per tick (all pairs) | ms per tick (grid) | change |
|---|---|---|---|
| 100 | 0.990 | **0.593** | **-40%** |
| 1 000 | 14.551 | **13.502** | **-7%** |
| 2 000 | 53.963 | **49.908** | **-8%** |
| 5 000 | 254.543 | **245.129** | **-4%** |

Amortised per 60 fps frame at one society tick per fifteen frames: 1 000 people **0.90 ms (5%)**, 5 000
**15.3 ms (92%)**.

## Where a 5 000-person tick goes (measured, direct timers)

| stage | ms |
|---|---|
| `perceive`, all pairs | **36.0** |
| `perceive`, grid | **28.9** |
| `decide_activities` | 82–108 |
| whole fixed step | ~320 |

## The curve is scoring, not search

`5 000 people x 4 Person-targeted activities x 182 neighbours = 3.64M` consideration evaluations per tick,
against 144 000 at 1 000 — 25x, matching the measured 17x slowdown closely enough to call it the cause. The
182 neighbours are genuinely there (0.995 people per m² on a 40 m disc), so an index cannot remove them.

## Not slowed

`village-evening` is bit-identical (67 199 of 67 199 rows), and the 24-person village is in the range where
the grid is cheapest — one cell per 8 m, four places, no dense crowd.
