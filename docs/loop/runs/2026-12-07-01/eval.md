# Eval

## Change

`tests/cluster_reach.rs` measures the cluster's observable (second group of six, not all twelve) and pins its
current failure; `scenarios.yaml` records the measurement and the cause and names its test; a decision file
records the three options.

No crate code changed.

## Before / after

| | before | after |
|---|---|---|
| the observable checked | by eye, never | **by test**, at two clocks |
| second group reached | 0 of 6 (unverified) | **0 of 6 (asserted)** |
| `test:` line | **absent** | `tests/cluster_reach.rs` |
| the cause | unknown | **arrival timing**, established by six layered measurements |

## The cause, in one line

An exchange fires on **arrival**, once. Two people placed together give **39 interactions**; two people who
arrive hours apart share 400 ticks of standing together and share no arrival, so nothing fires. A 20 m walk is
5.4 game-hours against a 0.75-hour activity.

## What I got wrong

Three attempts, all measured and discarded:

1. **A crossroads carrying `gathering`** — reach 0.167 → 0.167. `Place` targets pick the *nearest* match and
   each group's own hearth is 2 m away against 20.
2. **A market with a tag the hearths lack** — chosen for 36 489 ticks, arrived 6 720, still 0 of 6.
3. **An explicit cross-group `Interact`** — 0 of 6, and my probe addressed the rumor's own subject.

Each narrowed the cause; none of them is the fix, and none shipped.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **50** binaries |
| `cargo test --no-default-features` | 50 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.
