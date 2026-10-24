# Perf

Not measured this iteration: no model changed. The `evidence` example runs 4200 ticks with
24 people in about nine seconds of wall clock including process start, which is dominated by
Bevy's startup rather than by per-tick cost.

`scale-1000` in `scenarios.yaml` remains `pending` and is the fifth candidate in `review.md`.
