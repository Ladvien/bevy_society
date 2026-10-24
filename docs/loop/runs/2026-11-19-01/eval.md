# Eval

## The number that started it

| activity | share of person-ticks, `village-evening` at its own settings |
|---|---|
| **sleep** | **56.8%** |
| eat | 14.9% |
| work | 14.8% |
| relax, snub, compliment, gossip, play, chat, wander | 9.2% between them |

| | value |
|---|---|
| one 8-hour sleep a day | 33.3% |
| this run's 350 sleeps × 8 h over 24 people × 17 days | **28.6%** |
| **reported** | **56.8%** |

`evidence/reported-asleep.png` — the reported-asleep count peaks at **22–23 of 24** every night, against
the red line of 8 that one 8-hour sleep a day allows.

## The defect

A villager tracked tick by tick while `Doing.activity` said `sleep`:

```
   1: rest 0.087     7: rest 0.086
   2: rest 0.087     8: rest 0.086
  ...                12: rest 0.085
```

Decay alone is 0.7/day = **−0.00017/tick**; the observed change is **−0.00018/tick**. `sleep` pays 0.15/h
= **+0.00088/tick**. `rest` falls, so the payout has not begun: `arrived_day` is `None` and the person is
walking.

`travel_timeout_hours` is 20 at 64 s/day = **3 414 ticks**; an 8-hour sleep is **1 365**. A person walks
toward bed for two and a half times the length of the sleep, labelled asleep throughout.

## Four hypotheses disproved

| hypothesis | killed by |
|---|---|
| a `Place` activity overruns its duration | `elapsed_h = 8.004` on every sleep that ends |
| `satisfies` pays twice per tick | 2 h at 0.1/h paid **0.2010** — exact |
| `best_telling`'s tie-break starves a fresh rumor | three tests passed with **no code change** |
| a probe measuring overlapping sleeps | fixed with a queue; real range 8.00–28.00 |

The second is instructive: the first version of that test used 1.0/h, so a 2-hour activity paid 2.0 into a
ceiling of 1.0 and read **1.000** — which looked exactly like a half-rate payout. The need's clamp, not the
crate, was the cause.

## What ships

| item | state |
|---|---|
| `tests/drift.rs` | **2 pass** — the payout is exact, and displacement does not extend it |
| `tests/travel.rs` | **failing, ignored** — asserts the honest behaviour |
| `needs-decision/what-doing-reports-while-travelling.md` | three options, recommends a reserved `travelling` activity |

## Gates

| Command | Result |
|---|---|
| `cargo test` | **36** binaries, 0 failures |
| `cargo test --no-default-features` | 36 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/` unchanged from `HEAD`.
