# Perf (ms per tick, median / min of 5 reps, 200 ticks, `examples/benchmark`, baseline `169f806`)

The host load average was 20+ and spreads were 35 to 99 %, so only the minimum is a fair comparison.

| people | before median / min | after median / min |
|---|---|---|
| 100 | 0.909 / 0.629 | 0.896 / 0.689 |
| 1000 | 54.75 / 38.95 | 84.72 / 37.99 |
| 5000 | 1196.95 / 934.8 | not measured: the run was killed at the 3000 s ceiling under load |

No claimed change. The new per-tick cost with nobody away is one pass over the index that finds no `AwaySince` (`miss_the_away` returns before touching any tie) and an early return in `announce_returns`.
