# Eval

## Change

`chat`, `gossip` and `compliment` gain a `TieDelta::trust` on accept. Before this, **the word `trust` did not
occur in the starter pack at all** — `AcceptDef` centres trust on 0.5, `Tie::new` starts there, and the only
thing that moved it was `witness_reaction`'s `trust_gain · harm`, which is negative for every deed the pack
uses. So trust could only fall, and it suppressed every acceptance draw.

## Measured, controlled

The test builds the pack twice — as shipped, and with every `trust:` delta stripped — so the delta alone is the
only variable:

| | no trust delta | with |
|---|---|---|
| mean trust at an exchange | **0.261** | **0.345** |
| accept rate | **0.486** | **0.533** |

`village-evening`, seed 7:

| | before | after |
|---|---|---|
| mean pleasure | −0.0657 | **−0.0579** |
| positive person-ticks | 3.4% | **5.2%** |
| friends / rivals | 21 / 44 | **24 / 35** |
| mean needs | 0.607 | 0.607 |

## Tried and reverted, with the numbers

A `help` activity so the pack's unused `help` and `gift` deeds would run:

| gate | result |
|---|---|
| `Affinity > 0.55` | rivals **44 → 0**, friends 21 → 81 — no conflict at all |
| `Affinity > 0.70` | mood near neutral (38% positive ticks) but trust still declined |
| `Affinity > 0.80` | **1 accepted in 12 000 ticks** — inert |

No gate between those yields a believable village, and that is the useful result: **deed density is not the
lever**.

## Why trust still declines, stated plainly

A gain reaches two parties; a loss reaches **every witness** at full magnitude. One chat is `+0.10`; one insult
seen by three people is `-0.42`. The penalty multiplies with the crowd and the reward cannot. Recorded in
`needs-decision/a-deed-costs-what-the-crowd-saw.md` with three options and a recommendation.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 46 binaries |
| `cargo test --no-default-features` | 46 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.
