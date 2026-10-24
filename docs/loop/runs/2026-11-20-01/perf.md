# Perf

One harness fix is a perf fix: `scenarios.yaml` was parsed **once per tick** by `plant_at`, which is an
`include_str!` of a 130-line file scanned line by line, 67 200 times. Hoisting it out removes that entirely.

Not separately measured — the run is dominated by the simulation — but it also removed **57 000** `eprintln!`
calls, which are not free.

Library cost: none. `src/` is unchanged.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.573 |
| 1 000 | 15.67 |
| 5 000 | 300.8 |
