# Eval: does `failed_place_memory_hours` pay in the village?

Regenerate (release build of `examples/evidence`; `$cfg` is empty, `--stall-timeout 1`, or `--stall-timeout 1 --failed-place-memory 3`):
`evidence --scenario village-evening --seed $s --days 10 --out evidence/$name-$s $cfg`

Village, 24 people, 10 game-days. `completed/interrupted` = `ActivityEnded` counts for everybody.

| seed | config | completed | interrupted | friends | rivals |
|---|---|---|---|---|---|
| 42 | default (no stall detection) | 3280 | 172 | 20 | 20 |
| 42 | stall 1 h | 3496 | 311 | 16 | 27 |
| 42 | stall 1 h + memory 3 h | 3861 | 236 | 16 | 28 |
| 7 | default | 3275 | 141 | 23 | 24 |
| 7 | stall 1 h | 3838 | 332 | 15 | 40 |
| 7 | stall 1 h + memory 3 h | 3706 | 246 | 19 | 34 |
| 11 | default | 3345 | 139 | 20 | 25 |
| 11 | stall 1 h | 3476 | 325 | 24 | 24 |
| 11 | stall 1 h + memory 3 h | 3857 | 290 | 17 | 29 |

**Reading.** Memory alone (stall rows vs stall+memory rows) cuts interruptions 24 %, 26 %, 11 % and raises completions in two of three seeds. But stall detection itself is what moves the society: interruptions double (stall cuts activities the default lets run to the travel timeout) and rivals/friends swing by seed (rivals 24 to 40 at seed 7). The village default stays `None`/`None`: turning stall detection on is a behaviour change that needs plots and three reviews, and the friends/rivals swing is unexplained. No crate behaviour changed this pass; only `examples/evidence` gained `--stall-timeout` and `--failed-place-memory`. Raw per-run `summary.json` files are in `evidence/*/`.
