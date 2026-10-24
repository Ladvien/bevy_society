# Eval

## Change

`Tuning::witness_dilution` — a witnessed deed's trust cost is divided by `witnesses ^ dilution`, default 0.5
(square root). Before, every witness paid the full `trust_gain · harm`, so a deed's total cost grew linearly
with the crowd while a trust *gain* could only reach two people.

## Before / after

Per-witness trust lost from one witnessed insult:

| witnesses | before | **after** |
|---|---|---|
| 1 | 0.0800 | **0.0800** |
| 5 | **0.0800** | **0.0431** |

Trust at an exchange, controlled:

| | no delta | delta, no dilution | **both** |
|---|---|---|---|
| mean trust | 0.299 | 0.435 | **0.435** |
| accept rate | 0.499 | 0.533 | **0.558** |

`village-evening`, three seeds:

| seed | pleasure before | **after** | rivals before → after |
|---|---|---|---|
| 7 | −0.0579 | **−0.0536** | 35 → 33 |
| 42 | **−0.1268** | **−0.0456** | 50 → **34** |
| 99 | **−0.1102** | **−0.0522** | 45 → 47 |

## Two things I will not smooth over

1. **Seed 7 barely moved** (−0.058 → −0.054) and its rival count only 35 → 33. Seeds 42 and 99 moved a lot.
   The three-seed spread is the result; an average would hide that the effect is layout- and seed-dependent.
2. **Seed 99's rivals rose** (45 → 47) while its mood improved. Dilution lets trust recover, and a recovering
   village also tolerates the *insult* activity more often, which can manufacture new rivalry. Recorded rather
   than explained away.

## Why this is a fix and not a preference

The decision file recommended dilution from symmetry. The literature **measures** it: Sommerfeld et al. 2007
(DOI `10.1073/pnas.0704598104`) found gossip transmission has "a dampening effect ... the more extreme a gossip
was, the less impact it had on the reader", and cites Hess & Hagen on "an increase in **believability** with an
increasing number of sources". Believed more, moving each recipient less.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **48** binaries |
| `cargo test --no-default-features` | 48 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.
