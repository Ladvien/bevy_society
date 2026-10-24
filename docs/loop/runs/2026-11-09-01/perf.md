# Perf

The change is a computation in `finish`, which runs once when the plugin is built. It is three
floating-point expressions and a comparison.

No benchmark re-run: nothing in the tick path changed. `warn_short_travel_reach` reads
`SocietySettings` only, allocates nothing, and is called once.

Standing numbers, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.556 |
| 1 000 | 15.16 |
| 5 000 | 277.0 |

One measured effect worth recording, from `examples/evidence.rs`: raising the timeout from 40 to 55
game-hours changes the scenario's behaviour, not its cost — the same 4 200 ticks in the same 9.4 s.
