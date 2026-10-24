# Plan: name the refusal constant, and let the definition own it

## Why this over the runner-up

The runner-up is the whole mood finding — and it is a **decision** (`a-village-that-is-never-pleased.md`),
because three defensible answers disagree about what the model is, and one of them is a pack retune that a
host's own pack may not want.

What is actionable, and unambiguously wrong: the crate broke its own rule. *"Every rate is a named field on
`Tuning`"* — and the refusal distress was a literal `0.3`, the **only** unnamed emotion intensity in the
crate. A deed's emotion comes from the deed file's `harm`; `groups.rs` names its contest constants.

## Design

1. `Tuning::rejection_distress` replaces the literal, default `0.3`, documented as a design default with
   the OCC mechanism cited.
2. **A definition's own `on_reject` emotion replaces it.** Emitting both stacked `0.3` on top of a pack's
   `0.1` and measured *worse* mood than either alone — two paths for one event, which the crate's rules
   forbid. The crate's number is now the fallback for a pack that says nothing.
3. The starter pack gains `on_reject: (actor_emotion: Some((Distress, 0.1)))` on `chat` and `gossip`, so a
   refused greeting is mildly deflating rather than a small tragedy — and so the pack owns the number.

## Measured

Eight people in contact, 4 000 ticks, identical accept/refuse counts (51/56):

| configuration | mean pleasure |
|---|---|
| literal 0.3, no `on_reject` in the pack | **-0.202** |
| `on_reject 0.1` added, both emitted | -0.207 |
| `on_reject 0.1` + crate fallback replaced | **-0.165** |

The middle row is the reason for rule 2: two paths made it worse.

In the full village the effect is small (-0.150 to -0.154), which is itself the finding recorded in the
decision file: refusals are not the village's dominant negative source there, and the 3:1 ratio has
another one.

## Touched

- `src/core.rs` — `Tuning::rejection_distress`.
- `src/behavior.rs` — reads it; replaced by a definition's `on_reject`.
- `assets/starter.society.ron` — `on_reject` on `chat` and `gossip`.
- `tests/rejection.rs` (new).
- `docs/loop/needs-decision/a-village-that-is-never-pleased.md` (new).
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

| field | default | source |
|---|---|---|
| `rejection_distress` | 0.3 | design default; OCC mechanism (Gratch & Marsella 2004) |

## Tests

| Test | What it proves |
|---|---|
| `tests/rejection.rs::a_refusal_distress_of_zero_leaves_mood_better_than_the_default` | the constant reaches `Mood::step`: 0.0 gives **-0.105** against 0.3's **-0.202**, at identical accept/refuse counts |
