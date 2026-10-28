# Eval: is the friends/rivals swing under stall detection a real effect?

Regenerate (release `examples/evidence`, per seed `s` in 1 2 3 4 5 6 8 9 10 12; stall rows add `--stall-timeout 1`):
`evidence --scenario village-evening --seed $s --days 10 --out evidence/$name-$s [--stall-timeout 1]`

Village, 24 people, 10 game-days. Per-seed `friends rivals interrupted` from each `summary.json`.

| seed | default f / r / int | stall 1 h f / r / int |
|---|---|---|
| 1 | 18 / 19 / 140 | 17 / 29 / 382 |
| 2 | 25 / 23 / 116 | 27 / 22 / 347 |
| 3 | 18 / 29 / 158 | 16 / 23 / 321 |
| 4 | 17 / 26 / 127 | 16 / 35 / 345 |
| 5 | 23 / 21 / 124 | 13 / 25 / 396 |
| 6 | 21 / 27 / 116 | 14 / 30 / 395 |
| 8 | 19 / 33 / 143 | 23 / 19 / 310 |
| 9 | 15 / 27 / 146 | 24 / 19 / 358 |
| 10 | 20 / 29 / 126 | 23 / 26 / 337 |
| 12 | 15 / 27 / 157 | 11 / 44 / 458 |
| **mean** | 19.1 / 26.1 / 135 | 18.4 / 27.2 / 365 |
| **sd** | 3.1 / 3.9 | 5.1 / 7.3 |

**Reading.** Over ten new seeds stall detection leaves the mean friends (19.1 to 18.4) and rivals (26.1 to 27.2) where they were; the swing seen in 2026-10-28-04 (rivals 24 to 40 at seed 7) is seed-to-seed variance, which stall detection roughly doubles in spread (sd of rivals 3.9 to 7.3, one seed at 44). What it does change is interrupted endings, 135 to 365 per run (about 2.7x): it cuts walks the default lets run to the travel timeout. So the swing is not a bias in who becomes friend or rival; it is a wider scatter. No crate code or default changed; the village stays `None`/`None`.
