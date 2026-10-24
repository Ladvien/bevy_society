# Plan: four defects between the log's item and a rivalry

## Why this

The log's #1 was the clock decision, which this loop escalates rather than decides — it is written
up in `needs-decision/travel-time-in-two-clocks.md` with three options and a recommendation, and
taking it would be deciding it. #2 was rivalry, which is measurable with the harness, so it was
the pick.

## The chain, and why each step was taken in order

1. **Measure `snub`.** Zero person-ticks in the village against 2583 in a cluster. → the curve
   gates on the wrong thing.
2. **Replace the gate with a grudge.** `ctx.grudge` already exists, so no schema change is needed.
   → it scored 0, which is a bug, not the design.
3. **Fix the script binding.** `target` was −1 for every candidate, so no `ctx` call could read
   the person being considered. → snub fires, 125 deeds, still 0 rivals.
4. **Find where a deed touches the victim's liking.** It does not, and the SPEC says it does not.
   → the model gap.
5. **Fix witness approval's sign.** Witnesses were *rewarding* harmful acts to actors. → the
   negative tie finally appears.

## Design decisions

- `snub`'s consideration becomes `Script("ctx.grudge(actor, target)")` with a `Linear(2.5, 0.0)`
  curve. The 2.5 is a design default: Bondü & Richter 2016 give the mechanism (rejection and
  provocation sensitivity predict reactive aggression) but report correlations, not a rate.
- Witness approval becomes `norm · sign(harm)`. A witness who values kindness disapproves of an
  insult and approves of a gift. This reads the SPEC's *"opinion > 0 approves"* as approving the
  norm, which then judges the act.
- `patient_reaction` still does not move affinity. The SPEC specifies trust, the literature I
  could reach gives no rate for victim-to-aggressor liking, and inventing one would be the thing
  this crate does not do. It is recorded as a known gap instead.

## Touched

- `src/reputation.rs` (the approval sign),
- `src/behavior.rs` (per-candidate script binding),
- `assets/starter.society.ron` (`snub`'s consideration),
- `tests/behavior.rs` (two regression tests),
- `docs/research/rumor-and-reputation.md`, `CHANGELOG.md`, `docs/loop/log.md`.

## Not touched

`patient_reaction`'s affinity, for the reason above. `Signal::Field`. The clock decision.
