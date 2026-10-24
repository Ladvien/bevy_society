# Perf

No library change. `examples/evidence.rs` reads one more field from the spec.

The measured cost is a consequence of honouring the spec: scenarios now run their declared game-days at
their declared day length, which is many more ticks than the old four-second day needed.

| scenario | ticks before | ticks now |
|---|---|---|
| `need-cycle` | 1 500 | **80 000** |
| `rumor-crosses-a-cluster` | 1 200 | **19 200** |
| `village-evening` | 4 200 | **67 200** |
| `deed-draws-a-response` | 400 | **6 400** |

A full sweep of the four scenarios takes about 2 minutes in release, against about 20 seconds before. That
is the price of the scenarios meaning what they say, and it is paid once per iteration rather than per tick.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
