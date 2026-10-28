# Eval

`village-evening`, 17 game-days, `evidence/before/` = HEAD `bf7ddb1` pack (`Linear(1.0, 0.0)`), `evidence/after/` = `Power(1.15)`. Regenerate:

```sh
for s in 7 42 11; do cargo run --release --example evidence -- --scenario village-evening --seed $s --days 17 --out docs/loop/runs/2026-10-27-07/evidence/after/s$s; done
cargo run --release --example evidence -- --scenario village-evening --seed 23 --people 16 --days 17 --out docs/loop/runs/2026-10-27-07/evidence/fresh_after
```
(`before` = the same with `snub`'s curve restored.) Plot: `evidence/plots/friends_rivals.png` from each `series.csv`.

| seed | compliments : insults | friends | rivals | completed | interrupted |
|---|---|---|---|---|---|
| 7 | 1116:390 (2.9) → 1135:163 (**7.0**) | 21 → 50 | 35 → 8 | 5709 → 5239 | 247 → 255 |
| 42 | 1093:369 (3.0) → 1248:250 (**5.0**) | 22 → 45 | 28 → 12 | 5659 → 5670 | 278 → 209 |
| 11 | 1092:415 (2.6) → 1313:192 (**6.8**) | 20 → 52 | 36 → 11 | 5770 → 5734 | 242 → 243 |
| 23 / 16 people | 760:276 (2.8) → 690:163 (**4.2**) | 11 → 30 | 23 → 9 | 4020 → 3661 | 78 → 94 |

Sweep (seeds 7, 42; insults): 1.0 → 390/369, 1.1 → 397/371, 1.15 → 163/250, 1.3 → 56/0, 1.5 → 20/0, 2.0 → 3/0. Steep and threshold-like; 1.15 is the first value that reaches the paper's ratio while leaving rivalry (8–12 rivals) in place. Perf: no code changed.
