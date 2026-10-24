# Eval

## The log's #1: two attempts, both measured, both reverted

Goal: make `chat` reachable in `rumor-crosses-a-cluster`, where it ran 128 person-ticks against
gossip's 3093.

| attempt | chat person-ticks | social mean | outcome |
|---|---|---|---|
| baseline | 128 | 0.895 | — |
| gossip `satisfies` 1.0 → 0.3 per hour | **106** | 0.895 | worse, reverted |
| gossip interaction effect 0.15 → 0.05 | **153** | 0.903 | +20%, still starved, reverted |

The arithmetic ruling out a rate fix: a gossip fires ~17 times per person per game-day, so any
per-instance supply `X ≥ 0.06` covers the 1.0/day decay and holds the need above `chat`'s 0.45
threshold. A supply small enough to balance the decay makes gossiping worthless.

Evidence: `evidence/cluster-gossip-satisfies-0.3/`, `evidence/cluster-interaction-0.05/`.

Escalated rather than decided: `chat` and `gossip` serve one need with no differentiation, which is
a definitions-schema question.

## The change shipped: a startup warning for degenerate durations

| | value |
|---|---|
| tick length at `seconds_per_day = 4.0` | 0.094 game-hours |
| `snub` duration | 0.1 h = **1.1 ticks** |
| measured consequence | 38 snubs per person per game-day; re-decides every tick |
| before this change | accepted silently |
| after | `WARN ... 1 activities are shorter than two society ticks at seconds_per_day 4 (one tick is 0.094 game-hours) ... snub (0.1h = 1.1 ticks)` |

Verified by running the example and reading the log line, not by asserting the invariant alone.

`SocietyDefs::activity_durations()` is now public, so a host can run the same check itself.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **17** binaries ok (was 16; `tests/degenerate.rs` added) |
| `cargo test --no-default-features` | 17 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Scenario outcomes

Unchanged from the previous iteration, because nothing in this iteration alters behaviour: the two
attempts that would have were reverted. `village-evening` 110 friends / 0 rivals / reach 0.375;
`rumor-crosses-a-cluster` 1 / 12 / 0.333; `need-cycle` 2 / 8 / 0.0.
