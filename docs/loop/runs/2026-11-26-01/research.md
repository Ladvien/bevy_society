# Research: the scale curve was not where the crate said it was

## What the crate claimed

`docs/guide/concepts.md`, `examples/benchmark.rs` and `scenarios.yaml` all said the same thing: *perception
is brute force over every pair, so the cost is quadratic*, and `finish`'s warning called 1 000 people
"fine". The design note offered a spatial index as the fix.

## What it measures

`examples/benchmark`, release, M1 Max, starter pack, 60 ticks after settling:

| persons | ms per tick | ms per million pairs |
|---|---|---|
| 100 | 0.99 | 199.9 |
| 1 000 | **14.55** | 29.1 |
| 2 000 | 53.96 | 27.0 |
| 5 000 | **254.5** | 20.4 |

Quadratic, and the per-pair cost is flat — so **pair count was the only variable**, which is what the docs
said. The inference looked safe.

## Where the time actually goes

Instrumented with direct timers, at 5 000 people:

| stage | all-pairs perception | grid perception |
|---|---|---|
| `perceive` | **36.0 ms** | **28.9 ms** |
| `decide_activities` | 82–108 ms | 82–108 ms |
| a whole fixed step | — | **~320 ms** |

Perception is **12–14%** of the tick. The all-pairs scan was never the curve. Substituting a grid for it is
a 3–7% saving — real, but it does not change what a host can afford.

## What does scale

`decide_activities` scores every `Person`-targeted activity against every neighbour. The benchmark puts
5 000 people on a 40 m disc; density is **0.995 per m²**, so an 8 m perception radius genuinely holds
**182** neighbours (measured; 200 by the density calculation). A tick then runs

```
5 000 people x 4 Person-targeted activities x 182 neighbours = 3.64M consideration evaluations
```

against 144 000 at 1 000 people — **25x**, close to the measured **17x** slowdown. The neighbours are really
there, so no index removes them. This is a decision-design cost, not an indexing one.

## The source for the index

**Ericson 2005, *Real-Time Collision Detection*** (Morgan Kaufmann, ISBN 1-55860-732-3), §7.1 *Uniform
Grids*: *"cell size is generally adjusted to be large enough (but not much larger) to fit the largest object
at any rotation. This way, the number of cells an object overlaps is guaranteed to be no more than four
cells (for a 2D grid)..."* and *"with cells and objects being almost the same size the maximum number of
objects in a cell is bounded by a small constant."*

For points the largest object has no extent, so the direct reading is **one cell per query radius**: a
neighbour within `perception_radius` is in the cell itself or one of the eight around it. §7.1 also warns
the failure mode is a grid "too coarse (with respect to object size)", where cells hold many objects and the
test degenerates to all-pairs — which is exactly what happens under a dense crowd, and is why the grid alone
does not rescue 5 000 people on a plaza.

## What believable output looks like

The same society, bit for bit. This iteration's claim is not behavioural, so the acceptance is **identity**:
67 199 of 67 199 series rows of `village-evening` unchanged, and a test that the grid's neighbour lists equal
the all-pairs lists exactly — same people, same ascending index order.
