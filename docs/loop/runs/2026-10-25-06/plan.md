# Plan: a profession is a set of tasks, and a skill is what you bring to them

## Pick, and why

**Pick:** the smallest slice of "professions and economics" that is measurable and that the literature
actually supports — **a `Skill` input and an optional `profession:` on an activity**, so that who does what
is decided by comparative advantage rather than by tag availability.

**Why this over the whole economy.** The research settled that a task is not a skill (Acemoglu & Autor
2010). A money-and-prices economy is a second model on top of that, and its rates are unsourced. This slice
answers the question the request is really asking — *do people with different work behave differently?* —
and it is the thing a prices layer would need underneath it anyway.

**Runner-up:** an Edgeworth barter market between pairs. It fits the crate (`Interact` is a pairwise
transaction) but it needs a goods list, an inventory component, a price index and three unsourced rates
before it can do anything at all. It is the *next* iteration, and the decision note
(`needs-decision/professions-and-economics.md`) now carries the design for it.

## Design

### `Input::Skill(String)` — the new consideration input

A skill is a **per-person scalar in `[0, 1]`**, named by the definitions file, exactly like a need:

```ron
skills: [(id: "milling"), (id: "weaving"), (id: "smithing")],
```

A person's value comes from its **personality**, not a new component: a skill's level is
`skill_of(traits, id)` — deterministic, and different for every person, so comparative advantage exists
without storing anything. That is the trick that keeps this additive: no new `Component`, no save-format
change, no new state to keep deterministic.

The mapping: **conscientiousness sets the craft skills, openness sets the creative ones, extraversion the
social ones** — three named groups, each a `Tuning` weight, each documented as a design default. A skill
declares its group in the defs file:

```ron
skills: [(id: "milling", group: Craft), (id: "weaving", group: Craft)],
```

### `profession:` on an activity — the task side

An activity may name the skill it is a task for:

```ron
(id: "mill", target: Place("work"), profession: Some("milling"), duration_hours: 4.0, ...)
```

The activity's considerations can then read `Skill("milling")`, which is how a good miller chooses to mill.
**Nothing is gated**: an unskilled person can still take the task, just at a lower score. That is
comparative advantage, not assignment, and it is what the paper describes.

### The profession tag on a person

`Profession(String)` — a `Component` naming the skill a person is *known* for. Optional: a person without
one behaves exactly as today. Its only effect in this slice is a bark and a `ctx.profession` for scripts,
which keeps it honest until the barter layer needs it.

### The economy is deferred, with its design recorded

`needs-decision/professions-and-economics.md` gains the Edgeworth-barter design and the two measurements
that must come first. Nothing about money ships.

### Touched

| file | change |
|---|---|
| `src/defs.rs` | `SkillDef { id, group }`, `SkillGroup { Craft, Creative, Social }`, `skills: Vec<SkillDef>`, `ActivityDef::profession`, `Input::Skill` |
| `src/core.rs` | `profession_of(traits, group)`, and three `Tuning` weights |
| `src/behavior.rs` | `signal_value` handles `Input::Skill` |
| `src/script.rs` / `snapshot.rs` | `ctx.profession(a)` |
| `assets/starter.society.ron` | three skills and a `mill` task, so the pack demonstrates it |
| `examples/assets/village.society.ron` | the weavers get a task |
| `tests/profession.rs` | **new** |
| `docs/research/economics.md` | **new** paper note |

### Tests

1. **Same trait, same skill**: two people with identical personalities have identical skills, and two with
   different conscientiousness do not — determinism by construction.
2. **The skill changes the choice**: two people, same needs, one high conscientiousness. With a `mill` task
   gated on `Skill("milling")`, the conscientious one mills more often. Measured over a run, not a unit test
   of `signal_value`.
3. **Nothing is gated**: a low-skill person still completes the task when it is the only option.
4. **`village-evening` does not regress**: friends, rivals, rumor reach and interruptions unchanged within
   noise, because no existing activity gains a `profession`.
5. Determinism: same seed, same activity sequence.

## Measurement plan

- `village-evening`, seeds 7 / 42 / 11, 16.4 game-days. Report friends, rivals, rumor reach, interrupted
  endings, and the **activity mix by profession** — the number that has to move.
- A dedicated scenario: 12 villagers, three professions, one shared workplace; report the share of each
  task done by the person with the highest skill in it. A believable run concentrates: the best miller does
  most of the milling.
- Plots: (a) task share by skill rank; (b) activity mix before/after; (c) the existing village's four
  numbers, unchanged.

## Not doing

- **Money, prices, inventories.** Recorded with a design and a measurement plan instead.
- **A skill that levels up through practice.** It needs a `Component` and a save field; and the literature
  cited here is about *assignment*, not learning.
- **Gating** a task on a skill threshold. That would be assignment rather than comparative advantage, and
  the paper is explicit that the allocation is endogenous.
