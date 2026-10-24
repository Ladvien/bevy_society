# Eval

## The seed was unread too

`seed:` in `docs/loop/scenarios.yaml` was decoration: every scenario ran at the crate default
`0x5EED_5EED`. Scenarios declaring 7, 11, 3, 5 and 1 were all the same run. `Spec` now reads it, with
`--seed` still overriding.

Verified deterministic: two consecutive `need-cycle` runs end at identical values, `[0.957, 0.608, 0.669,
0.867]`.

## Every scenario at its own settings

| scenario | need means | all four cycle? | notable |
|---|---|---|---|
| `need-cycle` | [0.86, 0.81, 0.58, 0.79] | **yes** | `pending` → **`measured`** |
| `village-evening` | [0.29, 0.53, 0.24, 0.26] | **yes** | — |
| `rumor-crosses-a-cluster` | [0.64, 0.15, 0.40, 0.61] | **yes** | reach **0.583** vs its 40% observable |
| `deed-draws-a-response` | [0.86, 0.81, 0.63, 0.81] | **yes** | — |

`evidence/{need-cycle,village}-spec.csv`.

## The stale conclusion this retires

The log's #3 was *"`food` at 15% of ticks at zero"*. At the spec's settings `food` (need2) has a mean of
**0.58** in `need-cycle` and **0.24** in the village, with **one trough a day**. That number was an artefact
of the 4 s/day configuration, and it is retired rather than fixed.

## The plot nearly fooled me

`evidence/spec-needs.png` looks like violent thrashing across 19.5 days. Counting local minima below 0.4:

| need | troughs below 0.4 over 19.5 days |
|---|---|
| social | **0** |
| rest | **0** |
| food | **20** — 1.0/day |
| fun | **0** |

`evidence/need-cycle-zoom.png` over three game-days shows the clean picture: every need decays steadily and
jumps when its activity runs, `rest` holds at 1.0 through sleep, `food` sawtooths 0.25 → 0.80 once a day.

**A full-range plot can look like a defect that is not there**, and the trough count is what tells them
apart.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **34** binaries |
| `cargo test --no-default-features` | 34 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/` is unchanged from `HEAD`.
