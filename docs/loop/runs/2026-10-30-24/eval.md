# Eval
Regenerate (release): `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` for S in 7, 42, 11, 99; town: `--seed S --people 60 --days 30` for S in 7, 99. Before = `runs/2026-10-29-04/evidence/before` (main, same harness). Host load high; no timings.

| run | insults b->a | accepted compliments b->a | rivals b->a | friends b->a |
|---|---|---|---|---|
| village s7 | 390->202 | 1116->1334 | 35->39 | 21->66 |
| village s42 | 369->159 | 1093->1473 | 28->25 | 22->90 |
| village s11 | 415->186 | 1092->1325 | 36->37 | 20->72 |
| town60 s7 | 1875->781 | 5798->6956 | 170->49 | 29->249 |
| village s99 (fresh) | - | 1467 | 30 | 80 (insults 167) |
| town60 s99 (fresh) | - | 6856 | 30 | 255 (insults 729) |

Exponent sweep, town seed 7 (snub `Power(e)`): 1.1 -> 52 friends / 140 rivals / 1558 insults; 1.15 -> 249 / 49 / 781; 1.2 -> 651 / 0 / 7; 1.3 -> 576 / 5 / 67. Steep: recorded as a caveat.

Gates (release unless noted): full `cargo test --all-features` green incl. unmodified `label_hysteresis` (9/9, was red with the label-counting brake), `friendship_bound` (1/1), `friend_capacity` (2/2); clippy -D warnings, fmt, doc, `--no-default-features` build. Plot: `evidence/plots/before-after.png`.
