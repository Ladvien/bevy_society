# Perf

No library change and no measurable harness cost: one comparison per sampled tick.

`need-cycle` still runs 80 000 ticks (~40 s in release), which is what 19.8 game-days at 64 s/day costs.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
