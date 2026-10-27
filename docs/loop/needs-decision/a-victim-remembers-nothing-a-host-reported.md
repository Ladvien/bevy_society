# Does a victim get a memory of what was done to them?

> **Escalated 2026-10-27-03.** A structural change to a public enum, so it is recorded rather than decided.

## The state

Two paths write a deed's reaction, and they disagree about what the victim keeps.

| path | who is written | what the victim keeps |
|---|---|---|
| an accepted `Interact` with `deed: Some(..)` | witnesses (perception), the patient, the actor | an `Interaction` memory at `-(0.3 + \|harm\|)`, which **does** feed `Memories::grudge` |
| a host-written `ReportDeed` | **only the named witnesses** | **nothing at all** |

Measured on the second path (`tests/` scratch probe, one bystander named as witness, deed `insult` at harm `-0.4`):

```
BEFORE patient->actor aff 0.0000 trust 0.5000 | witness->actor aff 0.0000
AFTER  patient->actor aff 0.0000 trust 0.4200 | witness->actor aff 0.0000
AFTER  patient mems []  grudge 0.0000
AFTER  witness mems ["Witnessed { deed: DeedId(0), actor: 65v0 } v=-0.000 i=0.400"]
```

The victim's trust moves, its affinity did not (that half is fixed: `Tuning::negativity_ratio`), and **it has
no memory of the deed**, so `Memories::grudge` reads `0.0000` forever after.

## Why it matters

`snub`'s only consideration is `Script("ctx.grudge(actor, target)")`. A grudge is `max(0, −valence)` over the
memories about a person. So after a `ReportDeed` — the surface a host uses for every event the society has no
eyes for: a brawl, a theft found at dawn, a townsperson found afflicted — **the victim can never come to resent
the person who did it**, and the retaliation the whole reputation chain exists to model is unreachable.

The exchange path has the memory by accident, in `resolve_interactions`, because an exchange already records one
on both sides.

## Options

### 1. Record `MemoryKind::Witnessed` at the victim too

- **For:** the enum already exists, the save format already carries it, and no host code changes. It is one line
  in `patient_reaction` plus a `deed` argument.
- **Against:** it is **not what happened**. `Witnessed` means "a deed seen by the person" (its own doc says so),
  and a victim did not see it, they were on the receiving end. A host matching
  `MemoryKind::Witnessed { .. }` — `tests/report_deed.rs` counts exactly that — would start seeing victims in a
  set documented as bystanders.

### 2. A new `MemoryKind::Suffered { deed, actor }`

- **For:** the truth. A victim's memory of an insult is a different fact from a bystander's, and the crate's two
  other kinds already distinguish those roles (`Interaction { initiated }`). A host can react to being wronged
  differently from seeing wrong done.
- **Against:** `MemoryKind` is a **public enum with no `#[non_exhaustive]`**, so a host that matches it exhaustively
  gets a compile error — a breaking change — and `SocietySave::VERSION` must go to 3, refusing every existing
  save. `tests/leaf.rs` and `docs/guide/persistence.md` both have to be revisited.

### 3. Leave it, and say so

- **For:** no schema movement, and a host which wants a victim's grudge can `fx.memory` it from a script.
- **Against:** it is a silent asymmetry between two paths that the crate presents as the same thing. The
  `ReportDeed` doc says "this is the same path [as an accepted exchange] with the witness list supplied instead",
  and it is not.

## Recommendation

**Option 2, in an iteration that has the room to do the save migration properly.** It is the only one that is
both true and usable — option 1 puts victims into a set documented as bystanders, and option 3 leaves a doc
claiming two paths are one. The next iteration should ship it together with `SocietySave::VERSION` 3, the
migration note, and a `tests/report_deed.rs` assertion that the patient holds a `Suffered` memory and no
`Witnessed` one.

**Not a defect this iteration introduces:** the missing memory predates it and the affinity half is fixed
separately. `docs/loop/runs/2026-10-27-03/research.md`.
