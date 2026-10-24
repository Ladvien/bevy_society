# Eval

## The budget, before and after

| need | before | after |
|---|---|---|
| social | 2.78 uses/day of `chat` | 2.78 (unchanged, and never short) |
| rest | 0.58 of `sleep` | 0.58 |
| food | 1.00 of `eat` | 1.00 |
| **fun** | **1.33 of `relax`** | **0.94 of `play`** |

And `fun` now has two sources: `play` at 0.85 per use and `relax` at 0.60.

## Behaviour, same seed and layout, after tick 200

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **1.3%** |
| `fun` mean | 0.061 | **0.112** |
| `social` mean | 0.124 | 0.107 |
| `rest` mean | 0.221 | 0.212 |
| `food` mean | 0.101 | 0.080 |
| **mean of all four** | **0.127** | **0.128** |

`evidence/play.png`, `evidence/{before,after}.csv`. The plot and the numbers agree this time: `fun` holds
a band above the old line and never flatlines, and the mean of all needs sits higher — unlike last
iteration's rate change, whose plot looked better while its total was worse.

**The three other needs pay a little** — `play` competes with `chat` and `eat` for the same hours. Total
is flat (0.127 → 0.128) and the need that was at zero for a third of every run is now at zero for 1%.

## A test that fired correctly

`tests/need_budget.rs::the_budget_exposes_the_one_need_that_starves` asserted `fun` needs **> 1** use a day
and carried the message *"if this ever becomes <= 1, one evening session covers a day and the imbalance is
gone"*. It fired. Rewritten as `every_need_is_now_covered_by_one_use_a_day`, asserting the opposite for
`rest`, `food` and `fun`, with `social` documented as the exception.

## Narrative

`/tmp/v13/narrative.txt` shows `play` as a real evening activity: `d17 21h33: started play`,
`d17 22h07: finished play (completed)`.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **30** binaries |
| `cargo test --no-default-features` | 30 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `sources_for` is public and used by two tests; `play` is reached by the decision system like every
other activity.
