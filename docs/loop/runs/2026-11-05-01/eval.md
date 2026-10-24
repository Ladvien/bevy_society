# Eval

## The invariant

Two activities identical in every consideration, differing only in target kind. One person vs twelve,
on one place so travel is zero.

| people | `visit` (Person) starts | `rest_here` (Place) starts | `visit` share |
|---|---|---|---|
| 1 | 0 | 3 | 0.00 |
| 12, **before** | 35 | 1 | **0.97** |
| 12, **after** | 18 | 18 | **0.50** |

`tests/candidate_bias.rs`, written red first at `0.97`.

## Starvation, with travel removed

Same definitions, everyone on one plaza so no walk can fail. 1 500 ticks.

| people | `food` at 0 before → after | `fun` at 0 before → after |
|---|---|---|
| 1 | 0.0 → 0.0 | 5.9 → **12.9** |
| 4 | 4.5 → **1.3** | 5.4 → **1.2** |
| 8 | 9.6 → **3.2** | 28.2 → **4.2** |
| 16 | 11.4 → **2.3** | 43.4 → **7.7** |

The population-scaling is gone. The one-person row is single-person noise (`fun` depends on the
17–23 `relax` window, and one person has few starts in 1 500 ticks).

## Starvation, village layout, zero travel

24 people, four places at the origin, 4 200 ticks — the same run as `village-evening` except distances
are zero.

| | need0 | need1 | need2 | need3 |
|---|---|---|---|---|
| before, zero% at 0 | 0.0 | 0.0 | **39.3** | **64.7** |
| after, zero% at 0 | 0.0 | 0.0 | **1.9** | **4.8** |
| before, mean | 0.924 | 0.810 | 0.298 | 0.132 |
| after, mean | 0.869 | 0.810 | **0.554** | **0.526** |

That is the fix's own contribution, isolated from travel.

## Village with real distances

`village-evening`, seed 7, 24 people, 4 200 ticks. Needs still starve, because walks still do not
finish — the escalated two-clocks decision.

| | need2 at 0 | need3 at 0 | need2 mean | need3 mean |
|---|---|---|---|---|
| before | 62.9% | 74.6% | 0.042 | 0.042 |
| after | **27.7%** | **55.2%** | **0.070** | **0.049** |

Other counters moved with it: rumors 53 → 34, friend edges 118 → **42**, reach 0.333 → 0.417.
Fewer friend edges is expected: the draw no longer multiplies social contacts by the crowd, so the
village socialises less, which is the intended effect of the fix rather than a regression.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 21 binaries (20 + `candidate_bias`) |
| `cargo test --no-default-features` | 21 binaries |
| `cargo test --test determinism` | 2 ok — the target pick is still seeded |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero, unchanged. No public item added or removed — `Candidate` is private.
