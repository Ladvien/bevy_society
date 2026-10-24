# Eval

## The split the harness was not making

Same run, `village-evening` at its own settings, 67 200 ticks:

| label | share of person-ticks |
|---|---|
| `travelling:sleep` | **21.8%** |
| `sleep` | **14.7%** |
| `travelling:eat` | **12.6%** |
| `travelling:work` | **12.4%** |
| `work` | 9.1% |
| `travelling:snub` | 7.4% |
| `travelling:compliment` | 5.2% |
| `travelling:relax` | 3.1% |
| `gossip` | 3.0% |
| `eat` | 2.5% |

| | |
|---|---|
| **travelling, total** | **62.5%** |
| doing an activity | 37.5% |
| `sleep` alone | **14.7%**, against the 33.3% one 8-hour sleep a day allows |

`evidence/travelling.png`. Before this change the harness reported 36.4% "asleep" — of which **21.8 points
were walking.**

## The arithmetic behind it

At 64 s/day a crossing walk of 8.2 m is **375 ticks**. Eight starter activities are shorter than that:

| activity | ticks | scores `Distance`? |
|---|---|---|
| `snub` | **17** | **no** |
| `compliment` | **34** | **no** |
| `chat` | 51 | yes |
| `gossip` | 51 | yes |
| `play` | 85 | yes |
| `eat` | **128** | **no** |
| `relax` | **256** | **no** |
| `wander` | 85 | **no** |

## The candidate fix, measured

Adding `Distance` to `eat`, `relax` and `compliment`:

| | before | after |
|---|---|---|
| travelling | **62.5%** | **50.5%** |
| total need mean | 0.526 | 0.513 |

Not shipped: it edits the definitions file a host ships. Filed as
`needs-decision/travel-cost-is-not-scored.md`, recommending a crate-level consideration so the term exists
once and is opt-in.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **37** binaries, 0 failures |
| `cargo test --no-default-features` | 37 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/` unchanged from `HEAD`.
