# Eval

## Change

`Tuning::positivity_offset` (default 0.1) — the intercept of the positive affect channel, added to the
resting point that `Mood::step` returns pleasure to.

It was missing. The crate had the paper's **gain** asymmetry (negativity bias) and not its **intercept**
(positivity offset), and that absence is why pleasure's ceiling was exactly zero.

## Before / after

`village-evening`, seed 7, 24 people, 17 game-days:

| offset | mean pleasure | **max pleasure** | stdev | needs |
|---|---|---|---|---|
| 0.0 (old) | −0.154 | **+0.0000** | 0.035 | 0.596 |
| **0.1 (new)** | −0.122 | **+0.0155** | 0.032 | 0.603 |
| 0.3 | −0.058 | **+0.0476** | 0.036 | 0.603 |

The maximum crosses zero for the first time. Monotonic in the offset. Needs unchanged.

## A single-person check

A person alone in a field with no interactions:

| offset | idle pleasure after 2 000 ticks |
|---|---|
| 0.0 | **+0.0000** |
| 0.1 | **+0.0122** |

Not the full 0.1, and correctly so: the person still has a little emotion, and the steady state is
`(pull·I·c + back·rest) / (pull·I + back)`.

## What this does not fix

The mean is still negative. The offset tilts the resting point; a saturated negative centre dominates it.
At `I = 0.5`, `c = −1.0` pleasure sits near −0.6 whatever the offset. The mean needs the **appraisal
balance** — 1 302 distress appraisals against 427 joy — which is `a-village-that-is-never-pleased.md`'s
option 1 and remains a decision.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **45** binaries |
| `cargo test --no-default-features` | 45 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Tests updated rather than deleted

Three existing tests asserted "mood returns to the baseline". That is now false by design, so each was
rewritten to the new contract **and kept** — including a new one asserting that a zero offset restores the
old behaviour exactly, so the change is a setting rather than a silent one-way door.

## Dead-code inventory

Zero.
