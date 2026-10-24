# Perf

No library change. The harness adds one `Option` check and, when travelling, a `format!` per person per
sample — a few thousand strings per run, invisible against 67 200 ticks of simulation.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
