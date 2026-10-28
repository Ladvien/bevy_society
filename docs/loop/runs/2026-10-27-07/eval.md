# Eval

`village-evening`, 17 game-days, tree at HEAD `c7d7630` (before) vs `Power(1.2)` (after). Regenerate each cell:

```sh
for s in 7 42 11; do cargo run --release --example evidence -- --scenario village-evening --seed $s --days 17 --out docs/loop/runs/2026-10-27-07/evidence/after/s$s; done
cargo run --release --example evidence -- --scenario village-evening --seed 23 --people 16 --days 17 --out docs/loop/runs/2026-10-27-07/evidence/fresh_after
```
Committed: each `summary.json`. Raw `series.csv` is ignored.

| seed | compliments : insults | friends | rivals |
|---|---|---|---|
| 7 | 1116:390 → **1489:73** | 21 → 80 | 35 → 7 |
| 42 | 1093:369 → **1867:0** | 22 → 106 | 28 → 0 |
| 11 | 1092:415 → **1543:41** | 20 → 86 | 36 → 5 |
| 23 / 16 people | 760:276 → **1090:3** | 11 → 63 | 23 → 0 |

Exponent sweep (same seeds, compliments : insults): 1.4 → 1516:51, 1992:0, 1679:34, 893:48; 2.0 → 1836:3, 1903:0, 1995:0, 1015:2; 3.0 → 0 insults everywhere.
Perf: data-only change, no code path touched; not measured (load average ~18).
Limits: no plot or narrative rendered (no matplotlib on this host); the evidence is the counts above. Seed 42 shows no insult in 17 days.

## Blocked: the 60-person bar (issue #10)

`cargo test --release --test friendship_bound` (60 people, seed 7, 30 game-days) is green at HEAD and red with `Power(1.2)` (rivals 0, assertion `rivals >= 1`) and with `Power(1.4)`: 3216 of 3540 ordered pairs are friend ties (90.8 %, bar < 25 %), 0 rival ties. The snub rate that keeps the 24-person village near 5:1 removes the only brake on friendship in a town of 60. Reverted; the starter pack is unchanged.
