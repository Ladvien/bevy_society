# Research: `Doing` should say what a person is doing now

## Why this

Three of the loop's five top items are escalation-blocked (`affinity`'s four readers, the clamp trade, the clock
decision). The one unblocked host-facing defect is that **`Doing` lies to every consumer**: `activity` names the
activity that was *chosen* from the moment it is chosen, while the activity's needs and duration start counting
only when `arrived_day` is set.

Measured on the shipped village at 64 s/day, tallying `Doing.activity` raw over-reports sleep by more than three
points of every person-tick — the gap is people walking to bed. A host HUD reading it shows a village asleep
while a third of them are in transit.

No paper governs this. It is a contract question, and the crate's own evidence harness had already answered it
privately:

```rust
let name = if activity.arrived_day.is_some() { base } else { format!("travelling:{base}") };
```

## The design question, and why this answer

The decision file (`what-doing-reports-while-travelling.md`) offers three options and recommends option 2 — a
reserved `travelling` activity plus a new target field. That is the better *schema* and still needs a host's
approval, because it changes a value a host switches on and adds a field to a component hosts construct.

Option 1 said "document that a reader must check `arrived_day`". That leaves every consumer to remember, which is
why it was rejected. But there is a form of option 1 that does not: **put the check on the component.**

```rust
impl Doing {
    pub fn is_travelling(&self) -> bool { self.arrived_day.is_none() }
    pub fn label(&self, defs: &SocietyDefs) -> String { /* travelling:<activity> | <activity> */ }
}
```

- `activity` keeps its existing meaning, so **no existing consumer changes behaviour** — the byte-identical
  harness output proves it.
- A host that wants the truth calls `label`, which cannot be got wrong by forgetting a second field.
- The crate owns the convention two independent readers had already invented.

## What remains open

Option 2 is not foreclosed. A host that switches on the activity *name* still sees `sleep` for a walker. The
difference is that the crate now offers a correct alternative right beside the misleading one, and the decision
file says so.

## Sources

None — this is a data-contract question, not a model. The measurement is `docs/loop/runs/2026-11-19-01/` plus the
`village-evening` tally in this run's evidence.
