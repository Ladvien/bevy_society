# Eval

## Per person, at the scenario's own settings

64 s/day, 8 people, 80 000 ticks, seed 7. The lowest level each person reached on each need:

| need | lowest any person reached | cycles for all 8? | last iteration claimed |
|---|---|---|---|
| social | **0.39** | **no** | yes |
| rest | **0.50** | **no** | yes |
| food | 0.00 | **yes, once a day** | yes |
| fun | 0.12 | 6 of 8 | yes |

**Three of the four claims were wrong.** `social` and `rest` never come within 0.1 of the 0.35 threshold,
and `fun` fails for two of eight people.

## Why

The evidence series recorded a row at **tick 0**, before `size_person_state` fills `Needs`. Every need read
`0.000` in that one row, so "crosses below 0.35" was true of the series — on a state the society never has.

A mean over eight out-of-phase sawtooths cannot produce a 0.000 that none of them reached: the trough of a
mean is never below the lowest individual. So the artefact had to be the row, and it was. Exactly one sample
per need, at index 0, tick 0.

## The corrected series

With the row removed, the same run gives means whose minima are honest:

| need | min of the mean | crosses < 0.35 |
|---|---|---|
| social | 0.613 | no |
| rest | 0.535 | no |
| food | **0.218** | **yes** |
| fun | 0.556 | no |

`evidence/need-cycle-corrected.csv`, `evidence/tick-zero.png` — the left panel's "touch 0.00" is a single
vertical line at tick 0.

## Status

`need-cycle` goes back to **`pending`**, with the per-person figures in its note. Three needs staying above
0.5 is correct at these rates; the observable assumes starvation that does not happen. Whether the rates
should tighten or the observable should ask only that needs *recover* is a design question.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **34** binaries |
| `cargo test --no-default-features` | 34 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/` unchanged from `HEAD`.
