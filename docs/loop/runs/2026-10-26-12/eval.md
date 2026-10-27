# Evaluation: a meal is what was on the shelf

## What was built

1. **The payout.** `arrive_and_satisfy` (`src/behavior.rs`) scales an activity's `satisfies` by
   `min(1, held / asked)` over every good it declares in `consumes`, `1.0` and no `Stock` read for an
   activity that declares none. No `Tuning` field: the fraction is authored data.
2. **The pack that could not feed itself.** The producer `bake` moved from `examples/assets/village.society.ron`
   into `assets/starter.society.ron`, so `DefsFile::starter()` alone — a host's actual starting point — has an
   activity that makes the good its `eat` consumes.
3. **The meal and the loaf, calibrated.** `eat` consumes **0.25** of a loaf (was 1.0) and `bake` produces
   **4.0** (was 3.0).
4. **`warn_unmade_goods`** (`src/behavior.rs`): a startup warning naming any good that some activity consumes
   and none produces — the defect this iteration shipped in, made unsilent.

## The defect, quantified before the fix

The price pass ended by measuring a village holding **zero bread for 24.0%** of its ticks, in one run of 3.39
game-days, with nothing reading it. Counted from that same series (seed 23, 24 people, 17.62 game-days):
**4.80 loaves baked a game-day against 21.06 completed meals**, so ~77% of meals were eaten out of an empty
store and every one paid its full `satisfies: [("food", 1.2)]`. `produce_and_consume` took what `Stock` had
(nothing) and told no one.

## Before → after, same seed, same pack

Two populations, each measured with the pre-change binary built from `55475ea` in a worktree at
`~/.cache/bs-before` and the shipped binary. `village-evening`, 17.62 game-days.

### The authored village — seed 23, 24 people (the price pass's seated run)

| | before | after |
|---|---|---|
| mean `food` | 0.450 | **0.339** |
| `food` minimum | 0.240 | **0.015** |
| ticks below 0.1 `food` | 0.0% | **10.2%** |
| mean bread per person | 0.216 | **0.809** |
| **ticks with no bread anywhere** | **23.2%** | **3.8%** |
| price | 1.575 | **0.841** |
| friends / rivals | 26 / 5 | **26 / 2** |
| shares | 29 | **110** |
| rumor reach | 0.42 | **0.46** |
| needs (social, rest, food, fun) | 0.766, 0.724, 0.450, 0.632 | 0.760, 0.682, 0.339, 0.612 |

### Fresh eyes — seed 42, 16 people (different seed *and* population)

| | before | after |
|---|---|---|
| mean `food` | 0.416 | **0.330** |
| ticks below 0.1 `food` | 0.0% | 6.2% |
| mean bread per person | 0.416 | **0.817** |
| ticks with no bread anywhere | 9.3% | **4.1%** |
| price | 1.253 | 0.895 |
| friends / rivals | 16 / 1 | **20 / 0** |

**The famine is gone and `food` is lower, which is the same fact read twice.** The village used to be fed by
arithmetic: 77% of meals were paid for from a shelf that was empty. It is now fed by 4.0 loaves a bake, 0.25
a meal, and it holds bread on 96% of ticks instead of 77%. `food` falls because the meals it cannot supply
are no longer counted.

**Nothing clips and nothing drifts.** `evidence/plots/food.png` (seed 23) and `food-fresh.png` (seed 42): the
before line (red) keeps a floor near 0.24 that is the fiction; the after line (blue) reaches the real floor
and recovers on the next bake. The price never leaves its band and is never at the floor — the 0.5 values it
does reach are `v(1)`, the **full-store** value, on the ticks the store is genuinely a head or more.

## The calibration, and the two numbers rejected on the way

Three quantities interact — meals a day, loaves a bake, loaves a meal — and each candidate was measured at
**both** populations before the next was chosen.

| loaves a bake | meals a meal | `food` @24 / @16 | price | full-store ticks | friends @24 | reach |
|---|---|---|---|---|---|---|
| 3 (shipped before) | 1.00 | 0.450 / 0.416 (paid in full) | 1.575 / 1.253 | 0% / 3.1% | 26 | 0.42 |
| 3 | 0.25 | 0.356 / 0.271 | 0.738 | 50.1% | 18 | 0.12 |
| 6 | 0.25 | 0.336 / 0.371 | 0.710 / 0.744 | 62.1% / 73.0% | 21 | 0.25 |
| **4** | **0.25** | **0.339 / 0.330** | **0.841 / 0.895** | **40.0% / 30.9%** | **26** | **0.46** |

1. **Three loaves is starvation.** At 16 people it supplies 8.7 loaves against 14.8 needed; `food` 0.271 with
   26.5% of ticks under 0.1. This is the pack's own supply, and it is short once hunger is real.
2. **Six loaves oversupplies.** The store holds 1.67 loaves a person and the price sits at the full-store value
   on 62% of ticks — a glut, and the social fabric moves with it: friends 26 → 21 and rumor reach 0.42 → 0.25,
   because sharing swallows the day (29 → 140 shares). A village where a third of the day is trade is not the
   authored village.
3. **Four is the calibration that holds both.** It is also the one the pack's own arithmetic predicts: meals
   a day × a quarter loaf ≈ `people / 3.6 / 4`, a shade under one loaf a head, which is exactly what
   `Tuning::price_full_store` calls a full store. The two defaults agree instead of fighting.

**A finding that changed the design: the supply does not scale with the population.** Bakes a game-day were
**3.3 at 16 people and 3.9 at 24** — the dawn window (`Hours(6, 7)`, one hour) is what limits it, not the
number of mouths. Raising loaves a bake from 3 to 6 barely moved bakes a day (3.28 → 3.91). So the **loaf**
carries the whole calibration and the hour cannot: a host scaling its population must scale
`produces`, and that is now written in the pack beside the number.

## What the warning is for

`tests/unmade_goods.rs` pins the check against a written-out malformed pack rather than against the shipped
data: one test, because `tracing` allows a single global subscriber per process, so a second would read an
empty sink. It asserts the good is named, and that a pack whose own activity produces what it consumes is not
warned about.

**This is the one thing that would have caught the defect in the first place**, and it is why the check ships
with the fix rather than after it.

## Tests

| file | tests | what it pins |
|---|---|---|
| `tests/economy.rs` | 9 (5 new) | a meal from an empty shelf feeds nobody and still runs; a fed meal pays in full and is debited; half a loaf feeds half (`0.5 ± 0.06`); an activity with no `consumes` is unaffected; plus the five existing price and band tests |
| `tests/unmade_goods.rs` | 1 (new) | a good nothing produces is named at startup; a self-feeding pack is silent |

The empty-shelf test is the defect as a test. `half_a_loaf_feeds_half` is what pins the **linear** shape
rather than a threshold — a crate with no source for "a meal needs a whole loaf or it does not count" must not
invent one.

**One existing test failed and was a fixture whose premise was the old lie.** `tests/reciprocity.rs`'s
`the_village_keeps_both_labels` asserts that a dyad step leaves both `Friend` and `Rival` standing, and it ran
for **4 200 ticks** — a little over one game-day on a village that starts at 15:00 and bakes only at first
light. Under the old payout that window was fed anyway, so rivalries formed over meals nobody ate. Measured
at seed 7 now: rivals are **0 at 4 200 ticks**, **0 at 8 400** (one bake, `food` 0.455), and **8 at 12 600**
(`food` 0.586, 0.906 loaves a head). The test now runs 12 600 ticks with that arithmetic in its doc comment.
It tests the dyad step's claim — mutual and one-way ties, both labels, a small directional gap — and not the
supply of bread. Nothing about the assertion changed.

## What this does not do

- **No reservations.** Two overlapping meals read the same store and both read `1.0`. A claim system is a
  schema-sized change (who holds it, what clears it) and is recorded as remaining work, not guessed at.
- **Every `satisfies` slot scales, not a per-good one.** Attaching a good to individual `satisfies` slots is a
  definitions-file schema change and belongs in `needs-decision/`. The starter pack has one consuming activity
  and it pays one need, so the distinction is not yet observable — and the docs say so.
- **`DefsFile::merge` does not carry `goods` at all**, so a pack that merges in a new good silently has none.
  Found while moving `bake`; too large for this iteration and recorded as remaining work.
