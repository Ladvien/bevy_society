# Perf

## The change

None shipped.

## What it would have cost

`clamp(-K, K)` is the same single `clamp` call with different bounds — no compute change. The gate change is a
different pair of constants in the definitions file.

The real cost is not compute: it is that opinion convergence needs ~2× the ticks under the clamp, which is 2×
the simulation time for any host running that scenario.
