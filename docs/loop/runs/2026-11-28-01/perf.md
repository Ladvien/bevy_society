# Perf

## Crate

`Obstacle` adds one query pass over the solids and one `Vec` filter per mover per fixed step, plus a second
pass over the movers to resolve contact. The demo has four obstacles; the filter is over four entries.

Not benchmarked in isolation: `examples/benchmark` spawns no `Obstacle`, so the added work is one empty
query. The crate's scale numbers are unchanged.

## Clips

| | frames | size |
|---|---|---|
| `village.gif` | 240 | 4.6 MiB |
| `mood.gif` | 240 | 4.6 MiB |
| `opinions.gif` | 240 | 4.1 MiB |
| `needs.gif` | 240 | 5.0 MiB |
| `ties.gif` | 240 | 4.4 MiB |
| `factions.gif` | 240 | 3.9 MiB |

Each is a 21-second `capture` run plus a 4-second Pillow encode.
