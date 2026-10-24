# Plan: let a successful exchange raise trust

## Why this over the runner-up

Runner-up was `Doing`'s travelling semantics — a host-facing component, escalated. This iteration's item was
the pack's trust: `AcceptDef` centres trust on 0.5 and **nothing in the pack could raise it**, so the term
subtracted on every acceptance draw and 47% of interactions were refused. That is an omission, not a tuning
disagreement, and `AGENTS.md` sanctions pack changes as data.

## Design

`chat`, `gossip` and `compliment` gain `TieDelta::trust` on accept (`0.05`, `0.03`, `0.02`). The magnitudes are
design defaults, chosen against the witnessed-deed cost: a `steal` is `-0.14` per witness, so a conversation
at `0.05` means a bad act costs about three good ones rather than seven at `0.02`.

## Measured, as a controlled comparison

The test builds the same pack twice — once as shipped, once with every `trust:` delta stripped out — so the
delta alone is the variable:

| | no trust delta | with trust delta |
|---|---|---|
| mean trust at an exchange | **0.261** | **0.345** |
| accept rate | **0.486** | **0.533** |

`village-evening`, seed 7:

| | before | after |
|---|---|---|
| mean pleasure | -0.0657 | **-0.0579** |
| positive person-ticks | 3.4% | **5.2%** |
| friends / rivals | 21 / 44 | **24 / 35** |
| mean needs | 0.607 | 0.607 |

## What was tried and reverted

A `help` activity, so the pack's own `help` and `gift` deeds would be used. Two settings, both rejected on
measurement:

| `help` gate (`Affinity` logistic midpoint) | result |
|---|---|
| 0.55 | rivals **44 → 0**, friends 21 → 81 — a village with no conflict at all |
| 0.80 | **1 accepted in 12 000 ticks** — inert |

and 0.70 in between gave near-neutral mood but was still not enough to reverse the trust decline. There is no
gate that produces a believable village, which is the finding: **deed density is not the lever.**

## The finding that goes to a decision

A trust gain reaches two parties; a trust loss reaches every witness at full magnitude. One chat is `+0.10`;
one insult seen by three people is `-0.42`. The penalty is multiplied by the crowd and the reward is not.
Recorded in `needs-decision/a-deed-costs-what-the-crowd-saw.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `tests/trust_rises.rs::a_successful_exchange_raises_trust_against_a_pack_that_has_no_such_delta` | the delta alone raises mean trust **0.261 → 0.345** and accept rate **0.486 → 0.533**, with the stripped pack as the control |
