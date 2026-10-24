# Eval

## Change

1. `Tuning::rejection_distress` names the refusal constant — it was a literal `0.3`, the crate's only
   unnamed emotion intensity, in a crate whose rule is that every rate is a named `Tuning` field.
2. A definition's own `on_reject` emotion **replaces** it. Emitting both measured *worse* than either alone.
3. The starter pack gives `chat` and `gossip` an `on_reject` of `Distress 0.1`.

## Before / after

| configuration | mean pleasure, 8 people, 4 000 ticks |
|---|---|
| literal 0.3, pack has no `on_reject` | **-0.202** |
| pack `on_reject 0.1`, **both** emitted | -0.207 |
| pack `on_reject 0.1`, crate fallback replaced | **-0.165** |

Identical accept/refuse counts in all three (51 accepted, 56 refused), so the comparison is clean.

`village-evening` full village: -0.150 → -0.154. Small — and that is the finding, not a disappointment:
refusals are not the village's dominant negative source.

## The finding that goes to a decision

**Nobody in the village is ever pleased.** Max pleasure over every person and every tick is exactly
**+0.000**, stdev 0.031, for 17 game-days. `Mood::step` pulls toward the emotion centre at 4:1 and toward
the personality baseline at 1:1, and `pad::baseline` is **zero** for a neutral personality from
`Personality::sample` (traits centre on 0.5). So pleasure's ceiling is zero, and the emotion centre is
negative nearly always: 1 302 distress appraisals against 427 joy over 20 000 ticks, and 15 people ending
with live `Anger` against **0** with `Joy`.

Three defensible fixes, disagreeing about what the model is — pack retune, a "life is going well" term, or
a different pull:return ratio. Recorded in `docs/loop/needs-decision/a-village-that-is-never-pleased.md`.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **44** binaries |
| `cargo test --no-default-features` | 44 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.

## What I measured wrong first

Counted **emotion entries observed per tick** and reported "52 038 distress appraisals" — an emotion that
persists for 6 000 ticks was counted 6 000 times. `Emotions::appraise` merges same-kind-same-target and
caps, so the real number was **112**. The corrected count is 12x smaller and changes the story from
"saturated" to "a 3:1 ratio that the mood update cannot escape".
