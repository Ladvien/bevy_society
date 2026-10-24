# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what it claims?

Claim: a definition relying on a `Field` consideration or a `Gradient` target now produces a startup
warning naming it, where before it was accepted in silence.

**Evidence.** `tests/fields_unscored.rs` asserts the four facts the warning keys on: the definition
compiles, `activity_signals` reports the `Field` consideration, `has_gradient_activity` reports the
`Gradient` target, and the starter pack reports neither.

**Critique.** Same gap as the duration warning two iterations ago: the tests pin the *facts*, not the
*log line*, because `headless` has no `LogPlugin` and the crate has no log-capture harness. Deleting
the `warn!` call would leave all four green. That is now the second time this gap has bitten, and it
belongs on the log as its own item rather than being noted again.

**Verdict: GREAT** for the accessors and the facts, with the assertion gap named and escalated to the
top five.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique.** The warning must be silent for a host on the starter pack — otherwise every app logs it
on frame one and it becomes noise. Asserted directly (`the_starter_pack_uses_no_field_inputs`).

`Signal` became public to make this checkable. That is new public API in a crate that is
`publish = false`, and it is justified: a host that wants to lint its own definitions needs it. The
alternative — a `bool` accessor that hides *which* signal — would tell a host that something is wrong
without telling them what.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: does the record survive someone else reading it?

**Critique.** This iteration's real value is the negative result, so it has to be legible to whoever
picks it up. `needs-decision/field-inputs-are-unscored.md` records:

- what is measurably true (four facts, each with its value);
- what was attempted and reverted;
- the one ordering fact established (`Decide` precedes `Relate`, so decisions read the previous tick's
  grid);
- **what remains unexplained** — the tick-20 case, where the grid is populated and the score is still
  0;
- three concrete options with a recommendation.

That last bullet is the one that matters. A record that said "wiring is complex" would be useless; one
that says "these four things are true, this one is not, here is where I stopped" gives the next
attempt a starting point.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — facts pinned; the missing log assertion named and escalated (second occurrence) |
| Zoom | GREAT — silent on the starter pack, and `Signal`'s public exposure justified |
| Fresh eyes | GREAT — the negative result is recorded with enough detail to resume from |

## Next iteration's candidates

1. **`Field` / `Gradient` wiring** — the actual bug, with four measured facts and the unexplained
   tick-20 case recorded. Start from a single-person case with one cell of value 1.0, where nothing
   else can hide.
2. **The cluster's 1:24 starvation** — a second need; a schema change.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
4. **A log-capture harness** — now blocking two warnings from being properly tested. Promoted.
5. **A spatial index for perception** past ~2 000 people, with the benchmark numbers behind it.
