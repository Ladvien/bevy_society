# Plan: one candidate per activity, not per target

## Why this over the runner-up

Runner-up was the two-clocks decision, which the log has escalated twice. Lost on one sentence: **a
decision needs an operator, and this bug does not.** The choice bias is unambiguous — Dill's own text
says the draw is over options, and an option is an action — it is measurable in isolation, and it is
the reason a person surrounded by friends is *less* likely to eat. Fixing it also strips the confound
out of the two-clocks decision: after this, a starving village is evidence about travel and nothing
else.

## Design

Scoring stays per target, because a consideration may legitimately read the target
(`Distance`, `Affinity`). What changes is what the *draw* sees:

- For each activity, score every target as before, keep the **best** score and every target that
  tied for it.
- One `Candidate` per activity, carrying `targets: Vec<DoingTarget>` (the ties) instead of one
  `target`.
- The weight-based draw runs over activities, exactly as Dill describes.
- The winning activity's target is then picked uniformly from its ties, from the seeded RNG — so the
  same person and seed still reach the same place, and `tests/determinism.rs` still passes.

`Candidate` loses `Copy` (it holds a `Vec` now); the two places that relied on it clone instead.

**Why best-not-average:** an activity should be as attractive as its best available target. Averaging
would make a person in a crowd *prefer* the activity less the more neighbours it has, which is the
same bug with the opposite sign.

**Why ties rather than the argmax:** if `chat` scores the same with three neighbours, the target
should still be a choice, or everyone in a pair would always talk to the first person in the index.
The tie set is exactly the set of equally-good targets.

## Touched

- `src/behavior.rs` — `Candidate.targets`, the per-activity best/ties reduction, the target pick.
- `tests/candidate_bias.rs` (new) — the invariant, written red first.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. No rate changed; this is selection, not tuning. That matters: it means the fix cannot be
masked or double-counted by any authored number.

## Tests

| Test | What it proves |
|---|---|
| `a_crowd_does_not_multiply_a_person_activity_s_odds` | two activities that differ *only* in target kind keep a comparable share as the crowd grows |

The test's two activities are identical in score by construction (`visit` targets a `Person`,
`rest_here` targets the only `Place`), so any divergence is the target count and nothing else.
Written **red first**: at twelve people the `Person` activity took **0.97** of all starts; after the
fix, **0.50**.
