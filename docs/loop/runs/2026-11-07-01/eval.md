# Eval

## Scenario status

| Scenario | before | after |
|---|---|---|
| `need-cycle` | pending | **pending**, with a measured `blocked:` note |
| `rumor-crosses-a-cluster` | measured | measured |
| `opinion-converges-but-not-entirely` | **pending** | **measured** (`tests/opinion_convergence.rs`) |
| `deed-draws-a-response` | **pending** | **measured** (`tests/deed_response.rs`) |
| `village-evening` | measured | measured |
| `determinism` | **pending** | **measured** (`tests/determinism.rs`, already green) |
| `scale-1000` | measured | measured |

Four pending → one, and that one now says why.

## The summary fix

| scenario | `rumor_known_by` before | after | reach |
|---|---|---|---|
| `need-cycle` | 0 (with 119 rumors) | 0 (plants nothing) | 0.000 |
| `village-evening` | 0 | **1** | 0.042 |
| `rumor-crosses-a-cluster` | 0 | **4** | 0.333 |

The old column counted knowers of `RumorId(0)`, which for a scenario that plants a later rumor is
nobody — so it printed `0` regardless.

## The deed path, end to end

| | before | after |
|---|---|---|
| planted `PlantRumor` with 5 witnesses | 0 violations, `known_by = 1` | unchanged — **this is not what `PlantRumor` does** |
| accepted interaction carrying `deed: Some("steal")`, 4 witnesses | — | **1 rumor, `known_by` 4, 4 violations** |

Both paths measured; the second is the one the crate defines, and the scenario now tests it.

## Opinion convergence

| ticks | std dev | verdict |
|---|---|---|
| 2 000 | 0.4637 | |
| 6 000 | 0.3699 | scenario's original setting — **one percent short of a third** |
| 9 000 | 0.3447 | past a third |
| 12 000 | **0.2398** | what the scenario needs |

In the village the variance plateaus at 0.033 and stops falling — narrow but not collapsed, which is
the same claim at a different scale.

## The need-cycle blocker

| | |
|---|---|
| completed 4-hour sleeps in 20 000 ticks | **29** |
| ticks in 4 game-hours | **683** |
| `rest` range | **[1.000, 1.000]** |

The sleeps are back to back, so the need is met continuously and never decays. Cause: the starter
pack's `Linear(-1, 1.0)` scores a full 1.0 whenever the need is below 1.0, so the top-up activity is the
best option the instant the last one ends.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **25** binaries (22 + 3) |
| `cargo test --no-default-features` | 25 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero, unchanged. No library surface changed.
