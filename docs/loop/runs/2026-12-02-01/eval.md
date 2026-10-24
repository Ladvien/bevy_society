# Eval

## Change

`chat` and `gossip` pay mutual `Liking` on accept (`0.2` and `0.15`). They previously paid needs to both
people and **no emotion to either**, so 43% of accepted interactions — the most common 43% — left no mark on
anyone's mood.

## Before / after

Appraisals, 20 000 ticks, 24 people:

| | before | after |
|---|---|---|
| positive | **443** | **1 491** |
| negative | 1 587 | 1 587 |
| ratio | **3.58 : 1** | **1.06 : 1** |

Mood, three seeds:

| seed | before | **after** |
|---|---|---|
| 7 | −0.1224 | **−0.0657** |
| 42 | −0.1268 | **−0.0652** |
| 99 | −0.1102 | **−0.0696** |

Needs 0.603 → 0.607 — unchanged, so this is a mood correction and not a simulation rewrite.

## The structural fact underneath

| | count |
|---|---|
| refusals | 1 216 |
| distress appraisals | **1 216** |
| acceptances | 1 338 |
| positive appraisals | **443** |

Every refusal is exactly one appraisal and acceptances were not. That is the entire ratio, and it is now
pinned by an `assert_eq!`.

## Steady state

At the end of a 20 000-tick run, per person:

| | before | after |
|---|---|---|
| Anger | 14 | 14 |
| Distress | 4 | 25 |
| **Joy** | **0** | **22** |
| **Liking** | **0** | **14** |
| max pleasure | −0.047 | **+0.237** |

Positive affect was **absent** from the steady state and is now present.

## Honest limit

Mean pleasure is still negative (−0.066), and distress still outnumbers joy, because refusals are frequent.
The gain asymmetry — `Anger 0.5` against `Joy 0.3` — is what Cacioppo et al. describe as the negativity bias
and is deliberately unchanged.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **46** binaries |
| `cargo test --no-default-features` | 46 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.
