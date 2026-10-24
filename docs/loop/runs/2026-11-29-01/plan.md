# Plan: wire the field inputs

## Why this over the runner-up

Runner-up was `Doing`'s travelling semantics (`needs-decision/what-doing-reports-while-travelling.md`).
That one changes a component a host reads, so it is properly escalated. The field inputs are a **feature
that does not work** — with the `fields` feature on by default and the README advertising it — and wiring
them changes no schema and no message.

Measured before choosing: **49.8%** of person-ticks in `village-evening` are an activity the person has not
started, which is the travelling defect's scale, but it is a decision. Field definitions being silently dead
is a bug.

## Design

- `Decision` gains `own_factions: Vec<FactionId>` — the actor's memberships at the tick start. The `Field`
  inputs are faction-relative (`OwnFaction`, `OtherFactions`), so they cannot be read without it.
- `Signal::Field(input)` reads `SocialFields::field_value(input, &own_factions, cell)` at the actor's own
  cell.
- `ActivityTarget::Gradient(input, toward)` reads `gradient_target(...)`, which returns `None` on a flat
  field — correctly, because there is no direction to walk.
- One parameter had to go to stay inside Bevy's sixteen-parameter limit: `decide_activities` no longer takes
  a separate `Query<(Entity, &Transform)>`. The places query already carries their `Transform`, and a
  person's position comes from `Perception`.
- With the feature off, `FieldGrids` is a `NoFields` stand-in answering both calls with zero and `None`, so
  the decision keeps one shape in either build.

## The warning changes meaning

It used to say "the decision does not score these". It now says "the `fields` feature is off, so no grid
exists" — and it only compiles into the `not(feature = "fields")` build. With the feature on, a `Field`
definition is an ordinary definition and the crate stays quiet.

## Touched

- `src/behavior.rs` — `Decision::own_factions`, the two arms, the parameter rework, the warning.
- `tests/field_scores.rs` (new) — written red first.
- `tests/warnings.rs` — the field warning is now absent with the feature on.
- `CHANGELOG.md`, `docs/loop/needs-decision/field-inputs-are-unscored.md`, `docs/loop/log.md`.

## New `Tuning` fields

None.

## Tests

| Test | What it proves |
|---|---|
| `tests/field_scores.rs::a_field_consideration_scores_above_zero` | a populated field reads **0.120** at the chooser, a gradient exists, and the activity is chosen **6** times in 2 400 ticks |
