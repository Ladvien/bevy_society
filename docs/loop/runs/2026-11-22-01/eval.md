# Eval

## The defect in isolation

Two taverns serving `food`, one **1 m** away and one **9 m**, 6 000 decisions:

| | near | far |
|---|---|---|
| before | 3 020 | **2 979** |
| after | **5 999** | **0** |

A coin flip became a choice.

## In the village

Same seed and spawn, 67 200 ticks:

| | random tie | **nearest tie** |
|---|---|---|
| travelling | 62.5% | **58.9%** |
| need mean | 0.526 | **0.545** |
| `sleep` | 14.7% | 15.0% |
| `work` | 9.1% | 10.1% |

`evidence/nearest.png`, `evidence/{village-random,village-nearest}.csv`. At seed 42: 58.8% travelling,
0.550 need mean — same direction.

**The village effect is small**: −3.6 points of travelling and +3.6% of need mean. Its four places sit
within about 13 m of each other, so near and far rarely differ. The plot shows the two travelling lines
overlapping heavily, which is the honest picture.

## What did not change

- `need-cycle`: travelling 0.1%, needs 0.762 — everyone starts at the place.
- `rumor-crosses-a-cluster`: travelling 0.3%, needs 0.648, reach 0.167 — unchanged.
- Determinism: 2/2.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **38** binaries, 0 failures |
| `cargo test --no-default-features` | 38 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `place_distance` is reached from the tie-break and asserted by the new test.
