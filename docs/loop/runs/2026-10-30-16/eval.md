# Eval - brake + Power(1.15) vs before, same seeds

Regenerate (release): `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` for S in 7, 42, 11; town: `--seed 7 --people 60 --days 30`. Before: `docs/loop/runs/2026-10-29-04/evidence/before/*/summary.json`. Host load 20-26, no timing claimed.

| run | insults b->a | accepted compliments b->a | rivals b->a | friends b->a | rumor_reach b->a |
|---|---|---|---|---|---|
| village s7 | 390->156 | 1116->1253 | 35->27 | 21->91 | 0.167->0.292 |
| village s42 | 369->165 | 1093->1345 | 28->27 | 22->90 | 0.417->0.417 |
| village s11 | 415->181 | 1092->1377 | 36->34 | 20->72 | 0.167->0.125 |
| town 60 s7 | 1875->828 | 5798->6676 | 170->54 | 29->209 | 0.117->0.050 |

Village compliments:insults 2.9:1 -> 8:1 (bar 5:1). `friendship_bound` (release): 110 friend ties of 3540 (3.1%, bar < 25%), 180 rivals (bar >= 1). Plots: `evidence/plots/before-after.png`, `series-v7.png`, `series-p115.png`; narrative `evidence/v7-narrative.txt`. Perf not measured (host load 20+); the brake adds one tie-list scan per person per exposure pass.
