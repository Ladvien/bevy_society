# Eval - brake + Power(1.15) vs before, same seeds

Regenerate (release): `cargo run --release --example evidence -- --scenario village-evening --seed S --days 17 --out DIR` for S in 7, 42, 11; town: `--seed 7 --people 60 --days 30`. Before: `docs/loop/runs/2026-10-29-04/evidence/before/*/summary.json`. Host load 20-26, no timing claimed.

| run | insults b->a | accepted compliments b->a | rivals b->a | friends b->a | rumor_reach b->a |
|---|---|---|---|---|---|
| village s7 | 390->156 | 1116->1253 | 35->27 | 21->91 | 0.167->0.292 |
| village s42 | 369->165 | 1093->1345 | 28->27 | 22->90 | 0.417->0.417 |
| village s11 | 415->181 | 1092->1377 | 36->34 | 20->72 | 0.167->0.125 |
| town 60 s7 | 1875->828 | 5798->6676 | 170->54 | 29->209 | 0.117->0.050 |

Village compliments:insults 2.9:1 -> 8:1 (bar 5:1). `friendship_bound` (release): 110 friend ties of 3540 (3.1%, bar < 25%), 180 rivals (bar >= 1). Plots: `evidence/plots/before-after.png`, `series-v7.png`, `series-p115.png`; narrative `evidence/v7-narrative.txt`. Perf not measured (host load 20+); the brake adds one tie-list scan per person per exposure pass.

## Protected-test sweep (why it did not ship)

Brake (`friend_capacity` 10) plus the starter `snub` grudge curve `Power(e)`, protected tests (release):

| snub curve | `friendship_bound` (60-person test town) | `label_hysteresis` |
|---|---|---|
| Linear (shipped) | passes | passes |
| Power(1.1) | 47 friends, **1070 rivals** (30% of pairs) | passes |
| Power(1.15) | 110 friends (3.1%), 180 rivals, passes | **fails**: `rivals_with == rivals_without` is 19 vs 16 |
| Power(1.2), (1.25) | 776 friends (21.9%), **0 rivals**, fails `rivals >= 1` | not run |

The rival count is bistable in the exponent (1070, 180, 0), as the 2026-12-30 note found. At 1.15 the only red is `a_friendship_is_held_through_a_shallow_dip_and_lasts_longer`, which pins an exact equality of rival counts between two chaotic trajectories (band 0 and band 0.05); with the brake a held friendship occupies capacity, so the two runs diverge. Loosening it is not allowed in a loop pass. Patch: `friend-capacity-1.15.patch` (code, pack and changelog-free); its test: `friend_capacity_test.rs.txt`.
