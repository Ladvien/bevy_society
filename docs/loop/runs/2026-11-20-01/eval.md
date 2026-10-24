# Eval

## The timeout, fitted

| | 20 h | **4 h** |
|---|---|---|
| reach at 64 s/day | 74.7 m | **14.9 m** |
| village's longest walk | 2.2 h (8.2 m) | same |
| reported asleep | **56.8%** | **36.4%** |
| reported working | 14.8% | **21.5%** |
| needs mean | [0.268, 0.538, 0.217, 0.267] | **[0.709, 0.395, 0.402, 0.599]** |
| **total need mean** | **0.322** | **0.526** |

`evidence/{village-20h,village-4h}.csv`, `evidence/timeout-fitted.png`.

Holds at a second seed: seed 42 gives **36.2%** asleep and **0.511** total need mean.

**Rest falls** (0.538 → 0.395), which is correct: people no longer sleep for hours they have not earned.

## Two harness bugs

| | measured | fixed |
|---|---|---|
| `contact_radius` declared and never applied | `rumor-crosses-a-cluster` sets 4.0; the reader mapped three keys and not this one | `Spec::contact_radius` |
| `scenarios.yaml` parsed once per tick | **57 000 warnings** in a 67 200-tick run | `plant_at` hoisted out of the loop; **0** warnings |

## What that then showed

`rumor-crosses-a-cluster` reaches **0.333** at 12 h and **0.167** at 20 — both below the 40% its observable
asks for. The **0.583** recorded two iterations ago predates the `contact_radius` fix, so it was a
different run. The scenario is **`pending`**, with the design question in its note: two groups 40 m apart
need a **10.7 game-hour** crossing walk.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **36** binaries, 0 failures |
| `cargo test --no-default-features` | 36 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/` unchanged from `HEAD`.
