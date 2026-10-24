# Perf

No library change, so the standing benchmark is unchanged by construction. The measurement this
iteration took is the *shape* of the cost over a run rather than its level, and it is the substance:

| tick | rumors in book | ms/tick, no `RumorReach` trigger | ms/tick, with one |
|---|---|---|---|
| 1 000 | 765 | 5.76 | 5.95 |
| 2 000 | 1 454 | 5.64 | 4.99 |
| 3 000 | 2 412 | 3.51 | 3.07 |
| 4 000 | 3 374 | 2.97 | 2.79 |
| 5 000 | 4 299 | 4.80 | 3.93 |

200 people, four places, `seconds_per_day = 4.0`. Each figure is 100 ticks of wall clock.

**The book does not make ticks slower.** The cost drifts down as the village settles into longer
activities, and a `RumorReach` trigger does not change it — 4 299 pointer-chases are nothing beside 200
people scored against every neighbour within 8 m.

Memory is the real cost: ~48 bytes a rumor, so 105 KiB at 3 000 ticks and 228 KiB at 6 000 for 200
people, extrapolating to roughly 5 MiB for 5 000 over a comparable run.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
