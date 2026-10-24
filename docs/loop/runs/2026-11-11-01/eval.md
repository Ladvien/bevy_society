# Eval

## The defect

| | before | after |
|---|---|---|
| a person with `Destination(6.5, -5.0)` after 600 ticks | at `(0.0, 0.0)` | **6.72 m away, arrived** |
| `Place`/`Home` arrivals in a 24-person village | **26** of 228 starts | **212** |

## Completion rates, same seed, same layout, 4 200 ticks

| activity | target | completed before | after |
|---|---|---|---|
| chat | `Person` | 67/67 (100%) | 87/87 (100%) |
| gossip | `Person` | 86/86 (100%) | 89/89 (100%) |
| eat | `Place` | 14/62 (23%) | **63/74 (85%)** |
| relax | `Place` | 5/23 (22%) | **41/43 (95%)** |
| sleep | `Home` | 24/79 (30%) | **97/103 (94%)** |
| work | `Place` | 6/64 (9%) | **68/95 (72%)** |

## Needs, after tick 200 so the initial fall from 1.0 is excluded

| need | at zero, before | after | mean before | after |
|---|---|---|---|---|
| social | 39.3% | **2.0%** | 0.081 | 0.100 |
| rest | 43.6% | **0.8%** | 0.063 | **0.196** |
| food | 71.2% | **12.8%** | 0.019 | **0.075** |
| fun | 89.2% | **35.3%** | 0.008 | **0.031** |

`evidence/walk.png`, `evidence/{before,after}.csv`. The mean of the four needs stops being a flat line
at zero and starts oscillating — which is what `need-cycle` describes, and what a society with travel
should look like.

**`fun` remains the weakest at 35%.** That is the next iteration's item: `relax` is the only activity
that satisfies it and it is gated to 17:00–23:00.

## The timeout was not the cause

For the record, because the previous iteration's work makes it tempting to blame: 55 game-hours at
these settings is 587 ticks; at 0.0219 m/tick that is **12.8 m**, and the village's longest walk is
**8.2 m**. The warning added last iteration is correct in general and stays; this defect was separate.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **28** binaries (27 + `destination`) |
| `cargo test --no-default-features` | **28** binaries — and this is the configuration the bug lived in |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `walk_toward_destination` is reached from `BehaviorPlugin` and asserted by two tests.
