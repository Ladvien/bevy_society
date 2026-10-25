# Review: a basic economy

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

**GREAT, for the property the literature states.** The corpus's production/consumption model gives a testable
claim — "if the inflow of resources into the system equals the outflow, then the economy is stable and prices
fluctuate around a constant value ... shortages and oversupply" move it — and `tests/economy.rs` runs it as two
configurations of one village: balanced rests, starved rises. Bread is produced and eaten, the store moves, and
nothing gates.

## Pass 2 — Zoom: two bugs of mine, and the price the village paid

**GREAT, after two real failures.** Both were found by reading numbers, not code.

1. **The delta was per tick** (`speed · dt_days · imbalance`). An activity ends every few dozen ticks, so the
   price integrated one tick of flow against thirty of zero and moved by **0.0016** over 1 200 ticks. It looked
   exactly like "the model does nothing" and was one `dt` away from shipping.
2. **Scarcity was inverted.** `consumed − produced` made an undersupplied village's price **fall** (1.000 →
   0.550), because in a shared store the eaters who cannot find bread stop eating and consumption drops faster
   than production.

And the village paid: `bake` at `Hours(6, 18)` for 2 hours out-competed `work` and rivals went **26 → 1**,
**33 → 0**, **27 → 2**. Removing `bake` restores 26 / 33, so the activity is the cause. The shipped window is a
one-hour dawn errand: rivals 24 / 17 / 21, reach up at seed 7 (0.33 → 0.79) and down at seed 11 (0.67 → 0.08).

## Pass 3 — Fresh eyes: is it explainable, and does it persist?

**GREAT.** One sentence a host can follow: *the village bakes bread, eats it, and the price rises when it runs
short.* Three seeds measured for the regression. `Stock` round-trips through JSON exactly and a version-1 save
is refused with `RestoreError::Version` — the save-format consequence of adding per-person state, tested rather
than assumed. Determinism pinned.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**, after two measured bugs. Pass 3 **GREAT**.

## What a stranger would still complain about, correctly

- No trade between villagers, so there is a price and no counterparty.
- Rivals are 2-10 below baseline on every seed. Recorded in `eval.md`, not tuned away.
- `produce_and_consume` has an `O(n)` scan per ending that the benchmark does not exercise (no goods declared).
