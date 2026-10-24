# Eval

## What was tried

`relax`'s `satisfies` 0.4 → 0.55 so one 1.5 h evening session pays 0.825 against `fun`'s 0.8/day decay
(0.97 uses a day to break even, down from 1.33).

| after tick 200, 24 people, 4 200 ticks | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **18.8%** |
| `fun` mean | 0.061 | 0.051 |
| `social` mean | 0.124 | 0.112 |
| `rest` mean | 0.221 | 0.225 |
| `food` mean | 0.101 | 0.085 |
| **mean of all four** | **0.127** | **0.118** |

`evidence/relax.png`, `evidence/{before,after}.csv`. `fun`'s zero fraction halves; the total gets
**worse**, because `relax` now wins the evening against `chat`, `gossip` and `eat`. Reverted.

## What shipped

| | |
|---|---|
| `warn_unbalanced_needs` | warns when a need's best source needs more than four uses a day |
| `SocietyDefs::need_decay` | a need's decay per day, or `None` |
| `SocietyDefs::best_source` | the activity supplying a need most per use, and how much |

The budget it prints, on the starter pack:

```
social  decay 1.00/day, best source chat   pays 0.36/use -> 2.78 uses/day
rest    decay 0.70/day, best source sleep  pays 1.20/use -> 0.58 uses/day
food    decay 0.90/day, best source eat    pays 0.90/use -> 1.00 uses/day
fun     decay 0.80/day, best source relax  pays 0.60/use -> 1.33 uses/day
```

None of the four trips the four-use threshold, so the starter pack does **not** warn — correctly: `fun`
starves for a reason the definitions alone cannot see (its one source costs a walk and an evening
window), and `MAX_USES_PER_DAY` is deliberately set where only an implausible definition trips it rather
than where a real one does.

## Three hypotheses tested and rejected

| hypothesis | measurement | verdict |
|---|---|---|
| `satisfies` does not pay | fun sampled at arrival and at end of one 1.5 h relax | **+0.6000**, exactly `0.4 × 1.5` — payout correct |
| the 17–23 gate is binding | gate removed | relaxes 79 → 167, `fun` mean 0.053 → 0.092 — a constraint, not the binding one |
| capacity | well 6 × four batches in a 6 h window | not binding |

The binding constraint is choice: `relax` competes with three cheap `Person`-targeted activities for the
same evening.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **30** binaries |
| `cargo test --no-default-features` | 30 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. The two accessors are public and used by the tests; the warning is reached from `finish`.
