# Evaluation: a basic economy

Instrument: `tests/economy.rs` for the mechanism, `examples/evidence.rs` at `village-evening`'s own settings
(24 people, 64 s/day, 16.4 game-days) for the regression.

## The literature's acceptance test, run as two configurations

The production/consumption model in the corpus (`eth-cgl-ar_games-Zun16a`) states the property: "if the inflow
of resources into the system equals the outflow, then the economy is **stable and prices fluctuate around a
constant value** ... **shortages and oversupply**" move it.

| village | price before | price after 2 400 ticks |
|---|---|---|
| balanced (2 loaves a bake, 1 a meal) | 1.000 | **within 0.2 of where it was** |
| starved (0.1 of a loaf a bake, 1 a meal) | 1.000 | **> 1.000** |

Both asserted in `a_balanced_economy_rests_and_a_shortage_does_not`.

## Two bugs of mine, both caught by that test

1. **The delta was per tick.** `Δprice = speed · dt_days · imbalance` integrates one tick of flow against
   thirty of zero, because an activity ends only every few dozen ticks. The price moved by **0.0016** in a
   1 200-tick run and the flat-looking result read as "the model does nothing". The counters now accumulate
   between price updates and are cleared by the update.
2. **Scarcity was inverted.** Written `consumed − produced`, an *undersupplied* village showed its price
   **falling** (1.000 → 0.550): with a shared store, eaters who cannot find bread stop eating, so consumption
   drops faster than production and the raw difference turns negative. It is `produced − consumed` now.

## The regression, and what it cost

`bake` added to the village pack at `Hours(6, 18)` for 2 hours out-competed `work` and the village **lost its
rivalries**:

| seed | baseline rivals | `Hours(6,18)`, 2 h | `Hours(5,8)`, 1 h | `Hours(6,7)`, 1 h (shipped) |
|---|---|---|---|---|
| 7 | 26 | **1** | 14 | **24** |
| 42 | 33 | **0** | 15 | **17** |
| 11 | 27 | **2** | 15 | **21** |

Removing `bake` entirely restores 26 / 33, so the activity is the cause and not the goods system. The shipped
window is a one-hour dawn errand that sits where `work`'s `Hours(8, 17)` is shut. Rumor reach **improves** at
seed 7 (0.33 → 0.79) and falls at seed 11 (0.67 → 0.08); both are in the log.

**This is a cost, not a win.** Rivals are still 2-10 lower than baseline on every seed. It is recorded rather
than tuned away, because the alternative measured worse.

## What is not good enough

1. **No trade between villagers.** Edgeworth barter is the honest shape (Parunak et al.) and `Interact`
   already is that, but a `trade` interaction needs a resistance price and a profit rule, both unsourced.
2. **No money**, deliberately: Henrich et al. found meat is shared and money is not, so a good is a quantity.
3. **A price is one number per good** with no bids, no spread and no counterparty. It is an observation of the
   village's own flows, not a market.
4. **`bake` costs the village some rivalries.** Above.

## Determinism and the save

`the_economy_is_deterministic` pins stock and price across two identical runs. `Stock` survives
`capture` → JSON → `restore` exactly, and a version-1 save is refused with `RestoreError::Version` — both in
`tests/economy.rs`, the second gated on `serde`.
