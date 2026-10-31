# 2026-10-30-28 — plan: `Input::Stock`, and households that provision themselves (opt-in overlay)

## The item

Pass 27's top backlog item, a bug recorded in the log: the 60-person town starves for good. Bugs come first and there is no
open issue (`gh issue list` returns nothing); the parked branches (passes 20-40) are older than `main` and ignored.

## Design

1. **`Input::Stock(good)`** (`src/defs.rs`, `src/behavior.rs`): the actor's own holding of a good, clamped to `[0, 1]` like
   every input. Compiles to `Signal::Stock(GoodId)`; `signal_value` reads `decision.stock`, the copy of the actor's `Stock`
   that `Surplus` already makes, so it costs one indexed read and no new copy. No `Tuning` field: the pack says in its curves
   what "low" means. It needs no target (not in `check_input`'s person-only list), and an undeclared good is refused at load
   like every other id.
2. **The overlay** `examples/assets/provisioning.society.ron`: `bake` and `eat` that read it.
   - `bake`: `Hour` `Hours(6, 20)`, `Skill("craft")` as before, `Stock("bread")` `Linear(-1, 1)` (empty shelf 1.0, a day's
     supply 0.0), `Need("food")` `Logistic(-10, 0.5)` (only a hungry household makes bread).
   - `eat`: as before plus `Stock("bread")` `Linear(4, 0.05)`: a meal in hand is 1.0, an empty shelf prices the meal at
     a fourteenth; **not a gate**, so `economy.rs`'s "nothing gates" holds.
3. **The starter pack is not changed.** `tests/starting_larder.rs::a_starting_larder_removes_the_second_days_famine`
   (pass 27) asserts `empty > 0.2`, "the transient this guards against has to be there to cure". The overlay cures the
   transient by itself (`empty` 0.057 on the overlay, measured), so that precondition is false on the overlay pack. It is the
   one block left (the loop's protected-check rule: never remove or loosen an assertion), so the overlay ships opt-in and
   the default flip is recorded in `docs/loop/needs-decision/households-provision-themselves.md` with the exact patch
   (`provisioning-default.patch`). The owner flips it by deleting that one assertion.
4. **Evidence harness**: `Ledger` writes `activities.txt` (starts, completions and interruptions per game-day and
   activity, plus the hour each was chosen at); `--provisioning` runs any scenario on the overlay.
5. **Docs**: `docs/guide/authoring.md` (the input and the recipe), `docs/research/economics.md` (the open loop),
   `CHANGELOG.md`, the README input list, the log.

## Not in this pass

Prepotent hunger (`rank`), a price-driven bake, any change to the clock the harness runs at. Written up in `research.md`.

## Tests

- `tests/stock_input.rs` (4): a person with bread eats and one without makes some; the input reads as one from one loaf up;
  a curve rescales the unit (and exact zero removes the option); an undeclared good is an error naming it.
- `tests/provisioning.rs` (2, separate threads): the evidence town (60 people, homes at the tables) on the clock-driven
  starter pack is short of bread on game-day 8 and on the overlay holds it, at the default 240-second clock.
- Not changed: every existing test. The default pack is byte-identical, which `eval.md` checks by comparing a run on this
  branch's harness against HEAD's.

## Acceptance

Town of 60, seed 7, harness layout: floor share and bread a head over thirty game-days at the 64-second day and over twelve at
the 240-second day, HEAD against the overlay; four village seeds for what else moved (friends, rivals, insults); the plot, the
narrative of one villager across a bake, and three reviews.
