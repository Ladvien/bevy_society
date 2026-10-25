# Perf

## The change

One extra drain of `TieLabelChanged` per tick in `examples/evidence.rs`. `TieLabelChanged` fires only on label
transitions — 460 in a 67 200-tick village run — so this is a message read that finds nothing on almost every tick.

## Cost

Not measurable. The harness is a measurement tool, not the shipped path: nothing in `src/` changed.

## Scale

Unchanged, and the tool that measures it is now trustworthy: `examples/benchmark --reps N` reports
min/median/max/spread, and the guide quotes the **fastest window** (0.74 / 28.4 / 100.6 ms at 100 / 1 000 / 2 000
people) because that is the figure that reproduces to under 1%.
