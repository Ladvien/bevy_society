# Perf

No library change and no measurable harness cost: the instrumented counting was removed before commit.

`village-evening` runs 67 200 ticks at 64 s/day and takes about 50 s in release, unchanged.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
