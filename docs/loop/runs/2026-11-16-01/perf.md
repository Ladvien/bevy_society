# Perf

No library change. `examples/evidence.rs` reads a spec file at startup and tracks a peak while it runs;
the second is one `max` per sampled tick.

One measured cost is worth recording because it is a consequence of running the spec: the village now runs
**67 200 ticks** instead of 4 200, because 4 200 ticks at 64 s/day is 1.03 game-days rather than 17. A
release run takes **48 s** instead of 5 s — the price of the scenario meaning what it says.

| | before | after |
|---|---|---|
| ticks for `village-evening` | 4 200 | **67 200** |
| game-days covered | 17.03 at 4 s/day | **17.03** at 64 s/day |
| release run | 5 s | 48 s |

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
