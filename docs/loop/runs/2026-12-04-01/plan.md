# Plan: dilute a witnessed deed by the size of the crowd

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics (49.8% of person-ticks) — a host-facing component, so
escalated. Last iteration's decision file recommended witness dilution from a symmetry argument; this
iteration found that **the literature measures it**, which turns the recommendation into a fix. It is one
`Tuning` field and it keeps determinism: the witness count comes from `Perception`, which is already sorted.

## Design

`Tuning::witness_dilution` (default 0.5) is the exponent of the witness count in `witness_reaction`:

```rust
let seen = (witnesses.max(1) as f32).powf(settings.tuning.witness_dilution);
let gain = trust_gain(&settings.tuning, personality.conscientiousness) / seen;
```

The dilution applies to the deed's **magnitude**, never to its direction or to whether a witness is right to
disapprove — affinity, respect and the emotion keep their sign and their full weight.

`witness_reaction` gains a `witnesses: usize` parameter; `witness()` in `behavior.rs` already had the full
list, so the count is free.

## Measured

Per-witness trust lost from one witnessed insult, by crowd size:

| witnesses | before | **after** |
|---|---|---|
| 1 | 0.0800 | **0.0800** |
| 5 | **0.0800** | **0.0431** |

Before, identical at every crowd size — the aggregate grew linearly. After, `0.08 / sqrt(3)` for the three
bystanders, as intended.

Trust at an exchange, controlled comparison (same pack, deltas stripped):

| | no trust delta | with delta, before dilution | **with both** |
|---|---|---|---|
| mean trust | 0.299 | 0.435 | **0.435** |
| accept rate | 0.499 | 0.533 | **0.558** |

`village-evening` at three seeds:

| seed | mean pleasure before | **after** | rival pairs before → after |
|---|---|---|---|
| 7 | -0.0579 | **-0.0536** | 35 → 33 |
| 42 | **-0.1268** | **-0.0456** | 50 → **34** |
| 99 | **-0.1102** | **-0.0522** | 45 → 47 |

Large at seeds 42 and 99, small at 7 — recorded rather than averaged away. Rivals fall where the effect is
large, which is the mechanism showing through.

## Touched

- `src/core.rs` — `Tuning::witness_dilution`.
- `src/reputation.rs` — the divisor and the parameter.
- `src/behavior.rs` — passes the witness count.
- `tests/witness_dilution.rs` (new).
- `docs/loop/needs-decision/a-deed-costs-what-the-crowd-saw.md` — resolved.
- `CHANGELOG.md`, `docs/research/rumor-and-reputation.md`, `docs/loop/log.md`.

## New `Tuning` fields

| field | default | source |
|---|---|---|
| `witness_dilution` | 0.5 | Sommerfeld et al. 2007, DOI `10.1073/pnas.0704598104`; the exponent is a design default |

## Tests

| Test | What it proves |
|---|---|
| `tests/witness_dilution.rs::more_witnesses_means_less_trust_lost_by_each` | one witnessed insult costs each witness **0.0800** with one bystander and **0.0431** with five — **written red first at 0.0800 and 0.0800** |
