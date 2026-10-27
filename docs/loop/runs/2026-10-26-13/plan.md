# Plan: dilute the affinity and respect a witnessed deed writes

Iteration `2026-10-26-13`. One change, in one function.

## Design

`Tuning::witness_dilution` already exists, is already cited (Sommerfeld 2007), and is already applied to the
trust a witness loses. It is **not** applied to the affinity and respect the same reaction writes — and
affinity is what `TieLabel::classify` reads to decide `Friend` against `Rival`. So a crowd still multiplies the
half of a deed's cost that shapes the network, and damps only the half that does not.

The change is to divide all three writes by the same `seen` factor the trust write already uses:

```rust
let seen = (witnesses.max(1) as f32).powf(settings.tuning.witness_dilution);
let gain = trust_gain(&settings.tuning, personality.conscientiousness) / seen;   // unchanged

let tie = ties.entry(actor);
tie.add_affinity(0.15 * approval / seen, &settings.tuning);        // was 0.15 * approval
tie.respect = (tie.respect + 0.1 * approval / seen).clamp(-1.0, 1.0);  // was 0.1 * approval
```

One line of arithmetic, no new field, no new authored data, no schema change, no new draw. `witnesses == 1`
divides by 1.0, so a deed seen by one bystander is **bit-identical** to today — which is also the cheapest
possible determinism check.

## What is deliberately not changed

- **The emotion.** `Admiration` / `Reproach` are appraised at `|approval|`, undiluted. An emotion is a person's
  own reaction to what they saw, not a social judgement about the actor; Sommerfeld's dampening is about the
  transmission of a report and how much it moves a third party's *behaviour toward* the person, which is the tie.
  Diluting an appraisal would also change mood, and `a-village-that-is-never-pleased.md` has been fighting a
  mood deficit; folding this change into it would make the next measurement unattributable.
- **`NormViolated`.** Its bar is `approval < -0.5`, which reads `approval`, not the diluted write. A witness that
  objects still objects — the existing comment's promise, now true of the writes too.
- **`Memory` importance.** `valence` and `importance` are what a witness remembers, at `|approval · harm|` and
  `|harm|`. Memory is how a persona explains itself to a host in the narrative; diluting it would make the
  narrative disagree with the numbers.
- **The trust write.** Already diluted; no change.

## Touched modules

| file | change |
|---|---|
| `src/reputation.rs` | two divides; the doc comment on `witness_reaction` and on `Tuning::witness_dilution` say affinity and respect are diluted too |
| `src/core.rs` | `witness_dilution`'s doc: the field now governs all three writes, with the measured numbers |
| `tests/witness_dilution.rs` | extend: affinity and respect are diluted where trust already was; one witness unchanged to the bit |
| `docs/research/rumor-and-reputation.md` | the formula line |
| `CHANGELOG.md` | one entry |
| `docs/loop/scenarios.yaml` | `village-evening`'s observable: the friend:rival balance the paper predicts |

## Tests

1. **`tests/witness_dilution.rs` — the existing test extended, not replaced.** `more_witnesses_means_less_trust_lost_by_each`
   already measures trust at 1 and 5 bystanders. It gains two sibling measurements on the same fixture:
   `more_witnesses_means_less_affinity_lost_by_each` and `more_witnesses_means_less_respect_lost_by_each`.
   Each fails against today's code (equal magnitudes) and passes after.
2. **`a_deed_seen_by_one_bystander_is_unchanged`** — the same fixture at `witnesses == 1`, asserting the
   affinity is exactly what the old formula gives (`0.15 · approval`). This is the test that catches a
   rewrite that changes the single-witness case, which no measurement would notice at 24 people.
3. **The direction is not diluted.** `a_witness_that_objects_still_objects` — at six witnesses the affinity is
   still negative and `NormViolated`-worthy approval is still below −0.5. Guards against "dilute until zero".
4. **A determinism run** at the village seed, hash equal across two runs (the existing mechanism; no new test).

Not added, per the brief: no test that pins the shipped number 0.15 or the exponent 0.5 as a value. Those are
already documented as design defaults, and a test on the constant would only restate the source line.

## Acceptance

- `cargo test` and `cargo test --no-default-features` green; `clippy --all-targets --all-features -D warnings`;
  `fmt --check`; `doc --no-deps --all-features -D warnings`.
- `village-evening`, seeds 7 / 42 / 11, 17.6 game-days: friend pairs rise and rival pairs fall from the
  measured baseline (18 / 42 at seed 7), and the ratio moves toward the paper's "good prevails by numbers"
  without either end emptying. Rivals must stay above zero.
- One bystander unchanged: reproduced by the test above, not by a scenario.
- `perf.md`: cost at 100 / 1 000 / 5 000 persons, before and after. The change is two divides inside a loop
  that already runs per witness, so no measurable cost is expected; the number is recorded, not claimed.

## Risks

- **Over-dilution in a genuinely crowded village.** Six witnesses at exponent 0.5 keeps 41% of the magnitude, so
  a deed still bites. At 30 witnesses it keeps 18% — plausible (nobody feels personally wronged by something
  thirty people saw) but it is the shape to watch in the zoom pass, and `village-evening` at 3× population is
  the run that would show it.
- **`village-grows` interaction.** That scenario already reports rivals falling as populations grow; this
  change pushes the same direction. Both numbers go in `eval.md` so the two are not confused.
- **A test that no longer reaches its premise.** `tests/reciprocity.rs` asserts both labels exist; if dilution
  removes rivals at its seed, that fixture needs the same arithmetic-in-comment treatment as last iteration —
  and the assertion must not be weakened to pass.
