# Perf

## The change

None shipped.

## What was measured

Nothing in this iteration is compute-bound. The interesting cost is **simulation duration**: under the clamp the
opinion scenario cannot reach its threshold at any tick count, so a host running it would pay for 24 000 ticks and
still see spread at 0.40.

| ticks | spread | wall time |
|---|---|---|
| 12 000 | 0.3813 | ~9 s |
| 24 000 | 0.4036 | ~20 s |

Doubling ticks doubles the run and changes nothing — the clearest possible statement that the problem is not
duration.
