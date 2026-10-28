# Eval

`village-evening`, 17 game-days, `evidence/before/` = tree at HEAD `8649fce`, `evidence/after/` = this change. Regenerate:

```sh
for s in 7 42 11; do cargo run --release --example evidence -- --scenario village-evening --seed $s --days 17 --out docs/loop/runs/2026-10-27-04/evidence/after/s$s; done
cargo run --release --example evidence -- --scenario village-evening --seed 23 --people 16 --days 17 --out docs/loop/runs/2026-10-27-04/evidence/fresh_after
```
(`before` = the same with `git apply -R` of the `src/behavior.rs` hunk.) Raw CSV is ignored; `summary.json` per run is committed.

| seed | interrupted | interrupted `sleep` | completed | friends | rivals |
|---|---|---|---|---|---|
| 7 | 348 → **247** | 140 → **82** | 5719 → 5709 | 16 → 21 | 26 → 35 |
| 42 | 346 → **278** | 136 → **74** | 5657 → 5659 | 18 → 22 | 38 → 28 |
| 11 | 376 → **242** | 150 → **74** | 5717 → 5770 | 23 → 20 | 34 → 36 |
| 23 / 16 people | 164 → **78** | 61 → **20** | 3564 → 4020 | 13 → 11 | 18 → 23 |

Issue acceptance: interrupted endings do not rise against the 0.3.0 numbers (329, 369, 383): 247, 278, 242. Plot: `evidence/plots/interrupted.png`.
Model test: eight arrived bodies on a 2.0 m ring (disc count 0) admit a walker at 3.0 m (`1.5 + 0.7·sqrt(8) = 3.48`); fails on the old counting, passes now. Lone arrival unchanged (`an_empty_disc_is_unaffected`).
Perf: one sorted-Vec build per arrival pass; not measured separately (host load makes spreads of 37–57%).
