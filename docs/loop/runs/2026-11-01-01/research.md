# Research: `Field` and `Gradient` are accepted and inert

The log's #3 was "`Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting". This iteration
checked the premise before chasing the paper, and the premise was wrong in an important way: the
re-conversion is not what blocks them. **The field grids work.** Nothing reads them.

## What is true, measured

With the `fields` feature on (the default):

| Claim | Measured |
|---|---|
| The grids are built and deposited | yes — after two ticks `SocialFields` exists |
| `field_value(Crowd, …)` at a crowded cell | **1.0**, with sample == peak (0.069) |
| `gradient_target(Crowd, &[], pos, true)` | `Some(Vec2(4.0, -4.2e-7))` on a crowd |
| A `Gradient` activity over 60 ticks | **0 person-ticks** — never chosen |
| A `Field(Crowd)` activity over 60 ticks | 0 person-ticks |

So the dependency, the deposits, the evaporation, the peaks and both query functions are all
correct, and `signal_value`'s `Field` arm plus `targets_for`'s `Gradient` arm are the only things
missing. That is a wiring gap, not a missing paper.

## What I did, and where it stopped

Wiring is mechanical: both functions are public, and the decision already holds positions. I threaded
`Option<Res<SocialFields>>` through `decide_activities` → `score_option` → `signal_value`, and
through `targets_for` with the actor's faction memberships, and wrote two tests. **Both failed** and I
could not find why:

- A trace inside the `Field` arm showed it running, with the actor's real position.
- `decision.position` comes from `Perception.pos`, verified correct.
- `field_value` at that same cell, at that same tick, returns 1.0.
- The activity still scored 0, and the person idled on every tick of a 60-tick run.

One ordering fact is established and real: `SocialFields` is inserted by a deferred command in
`Clock` and `deposit_fields` runs in `Relate`, which is **after** `Decide`. A decision therefore reads
the previous tick's grid, and on the first tick reads no grid at all. That is defensible — a person
chooses from what they can see — but it does not explain the tick-20 case, where the grid is populated
and the score is still 0.

Rather than push a half-working change, the wiring is reverted and the measurements are recorded in
`needs-decision/field-inputs-are-unscored.md` with three options.

## Why the Moussaïd re-conversion is not needed for this

The log listed the re-conversion as a prerequisite because `Signal::Field` needs a model. It does not:
the crate already implements the field inputs (`FieldInput = Crowd | Commotion | OwnFaction |
OtherFactions | OwnShare`, with `OwnShare` citing Schelling 1971), and `docs/research/` documents
them. Moussaïd 2011 was wanted for a *crowd-steering* heuristic, which is a different feature — the
`Gradient` target walks a steeper-is-better field, which needs no crowd model at all.

## What shipped

A startup warning naming every definition that relies on either, so no host is misled while the
wiring is unfinished. Same shape as the `Script`-consideration warning two iterations ago, and pinned
by `tests/fields_unscored.rs`.

## Verdict

A negative result with a real deliverable: the log's item was mis-attributed to a missing paper, the
actual gap is a wiring bug I could not close this iteration, and the crate now says so loudly instead
of silently accepting a dead definition.
