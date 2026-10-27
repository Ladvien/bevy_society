# Plan: a victim is moved by what was done to them

Iteration `2026-10-27-03`. Research in `research.md`; the pick and its measurement are there.

## The change

`src/reputation.rs`, `patient_reaction`:

1. **The victim's affinity moves.** Today it moves `trust` and nothing else. It gains
   `tie.add_affinity(witness_affinity · severity, tuning)` where

   ```text
   severity = harm                             if harm >= 0   (a benefit)
            = harm · negativity_ratio          if harm <  0   (an injury)
   ```

   - **`harm`, not `norm · sign(harm)`.** A witness's reaction is a *judgement* of a norm and reads its
     `Opinions`; the patient's is a *reciprocation* of what was done to it. The two are the two halves of
     indirect reciprocity and the paper's claim is about reciprocity — "negative, displeasurable behaviours
     were reciprocated to a significant degree". Reading the patient's opinion would also make the write
     **silently zero for any person with no opinion on the topic** (`Opinions::value` returns 0.0 when
     unsized), which is the class of bug this iteration exists to remove.
   - `negativity_ratio` (5.0) is Baumeister et al. 2001 / Gottman's "bad events are on average five times as
     powerful as good ones", already cited by `snub` and already used by `Tuning::rival_band`.

2. **`Tuning::witness_affinity` (0.15) and `Tuning::negativity_ratio` (5.0) are new named fields.** The `0.15`
   is a literal inside `witness_reaction` today — a rate with no field, which breaks the crate's own rule. Hoisting
   it means the victim's write and the witness's write read one number.

3. **The signature grows** so `patient_reaction` can read `Tuning`. One caller
   (`src/behavior.rs:2948`), which already has it in scope. The patient's `Opinions` are **not** read, per
   the reasoning above, so nothing else is threaded through.

No new system, no new message, no schema change. The `MemoryKind::Suffered` half — the memory the `ReportDeed`
path never writes — is **escalated**, not decided: it changes a public enum a host matches on.

## Touched modules

| file | what |
|---|---|
| `src/core.rs` | `Tuning::witness_affinity`, `Tuning::negativity_ratio`, defaults, docs with DOIs |
| `src/reputation.rs` | `patient_reaction` gains the affinity write; `witness_reaction` stops using a literal |
| `src/behavior.rs` | the one call site passes opinions, topic and tuning |
| `examples/evidence.rs` | a `--negativity-ratio` sweep flag, and a `victim_likes_insulter` counter, so the defect and its repair are both readable from `summary.json` |
| `docs/research/rumor-and-reputation.md` | the equation and the default, with the source |
| `docs/guide/` | nothing: no host-visible surface moves except two `Tuning` fields |
| `CHANGELOG.md` | the entry, with before/after |

## Tests

Unit tests in `src/reputation.rs` (`patient_tests`; `patient_reaction` is `pub(crate)`, so they live beside it):

- **the victim's affinity falls for a harmful deed and rises for a helpful one** — both directions, because a
  write that only ever subtracts is a different, wrong model;
- **a harmful deed moves the victim more than a bystander** (the ratio's whole point), measured as a ratio
  against `negativity_ratio`, not against a magic number;
- **`negativity_ratio = 0.0` makes the victim's write symmetric** with a bystander's, so the field is read;
- **the victim's write respects `affinity_ceiling`** — a third theft does not take a tie past the clamp;
- **a benefit raises the victim's affinity by exactly `witness_affinity · harm`**, unscaled, so the ratio
  applies to injuries and nothing else.

`tests/tuning_covers_rates.rs` gains the two new fields to the surface check it already makes.

## Evidence

Before and after at **seeds 7, 11, 42**, same pack, 67 200 ticks, plus fresh eyes at **seed 23 / 16 people**.
The observable: `victim negative affinity after an accepted insult`, counted in `summary.json`, and the village's
friend/rival balance, which must not lurch.

A sweep of `negativity_ratio` over `0.0 / 1.0 / 5.0` at one seed, because 5.0 is a cited number and the *shape*
must be monotone rather than a lucky point.

## Risks

- **The affinity ceiling is 0.35 and a harmful write is `-0.75 · |harm|`.** At `harm -0.4` that is `-0.30` — one
  insult takes a neutral tie almost to the floor. That is the intended reading of "bad is stronger", and it is
  why the sweep exists: if the village's friend count collapses at 5.0, the number is wrong for this crate's
  scale and the record says so.
- **Determinism.** The write is `PersonIndex.sorted`-independent (it touches one person's own `Ties`) and draws
  no randomness, so the determinism test must stay bit-identical in structure. `tests/determinism.rs` will
  change *value* (a different society) and must not change *verdict*.
- **`witness_affinity` is a new field on a non-`#[non_exhaustive]` `Tuning`**, so it is a breaking change for a
  host that constructs `Tuning` in full. CHANGELOG's Breaking section.
