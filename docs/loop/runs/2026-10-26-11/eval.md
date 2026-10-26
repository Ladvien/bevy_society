# Evaluation: a price read off the stores, per head

Instruments: `tests/economy.rs` (5 tests) and `tests/script_economy.rs` (3, new) for the mechanism;
`examples/evidence.rs --scenario village-evening --seed 7` for the village, 67 200 ticks, 24 people, 17
game-days, before and after at the **same seed and pack**; `tools/plot_price.py` for the pair of series on one
pair of axes.

The "before" binary is the pre-change build kept at `~/.cache/price-before/evidence`. Same invocation for both
sides; only the binary differs.

## What was built

- **`src/model/price.rs`** (engine-free): `scarcity_factor(fill)` is the corpus's `v(r)`;
  `storage_price(total, people, full_store, nominal)` applies it to the mean held per person.
- **`update_prices`** sums `Stock` over `PersonIndex.sorted` and applies it, clamped to the band. No counters,
  no interval, no resource between ticks.
- **`Tuning::price_full_store`** (1.0) added; **`Tuning::price_speed_per_day`** and the **`Flow`** resource
  removed — both read only by the line they were removed with.
- **`ctx.price(good)`** and **`ctx.stock(person, good)`**, which `Prices`' own doc had promised since the
  economy landed and which did not exist.

## The defect, quantified

| | before | after |
|---|---|---|
| correlation of price with mean bread per person | **+0.598** | **−0.997** |
| ticks on the floor (0.25) | **25.3%** | 0% |
| ticks on the ceiling (4.0) | **6.0%** | 0% |
| price range | 0.25 – 4.00 | **0.79 – 2.00** |
| price standard deviation | **1.21** | **0.29** |
| final price | 0.500 | 1.583 |

**The sign is the whole result.** Before, the price *rose as bread piled up* — the recorded defect, and the
reason it read `0.500` at the end of a run whose village had nothing left. After, `r = −0.997`: the price is a
monotone function of the store, which is what a scarcity index means.

## The `S_max` the plot pass rejected

The first implementation priced `r` against `Tuning::stock_ceiling`, the storage clamp — the obvious reading of
the paper's "the storage size". The plot showed a flat line at 1.92 and the measurement said why:

| `S_max` | price range | sd |
|---|---|---|
| storage clamp (6.0 a head) | 1.76 – 2.00 | **0.05** |
| a day's supply a head (1.0) | **0.79 – 2.00** | 0.29 |

The village holds a quarter of a loaf a person against a clamp of six, so `r ≤ 0.12` always and the price never
left the famine end. That is the same class of defect as the one being repaired, one level down — a number that
cannot distinguish a glut from a drought. `price_full_store` is a design default at 1.0, documented as such with
this table in its doc comment and in `research.md`.

## The village is bit-identical apart from `price`

`summary.json` before and after are equal **field for field except `price`** (0.500 → 1.583), and `series.csv`
matches except that one column. That is the expected result and the guard: **no decision reads a price**, so
nothing about the society may change. Confirmed by diff rather than asserted:

```
$ diff <(jq 'del(.price)' before/summary.json) <(jq 'del(.price)' after/summary.json)   # empty
```

The corollary is the honest limit of this iteration: **the price does not yet drive behaviour.** A `Signal::Price`
input that makes `eat` costlier to skip in a shortage is a separate change with its own measurement, and the
`ctx.price` path is what makes it possible for a host to build that today.

## Tests

| file | tests | asserts |
|---|---|---|
| `src/model/price.rs` | 5 | the paper's three named points; continuity and monotonicity; per-head; the clamp; **a day a head spans the range where the storage clamp does not** |
| `tests/economy.rs` | 5 | the band is never left; a balanced village's price rests and a starved one is dearer and near double; empty stores gate nothing; determinism; a store survives a reload |
| `tests/script_economy.rs` | 3 | a script reads an empty store and the famine price **2.0**; half a day a head reads nominal **1.0** with `held 0.5`; an unknown good errors naming it |

Two tests were restated rather than deleted, and both because their **premise** was the old model:

1. `a_balanced_economy_rests_and_a_shortage_does_not` asserted a starved village's price *rose from its
   starting value*, which only means something for a price that integrates a flow from a nominal `1.0`. A
   function-of-state price is already at the empty-store extreme at tick 0. It is now
   `a_balanced_economy_rests_and_a_shortage_is_dearer`, a **comparison between two villages**, which is the
   paper's own formulation.
2. `a_script_reads_the_person_store_and_the_price_falls_as_the_village_fills` first inserted `Stock(vec![3.0])`
   and expected nominal, which encoded the clamp as `S_max`. Half a day a head is `0.5`, and the test now says so.

## Where it stands against the log's four requirements

| requirement (log, four iterations) | outcome |
|---|---|
| count **unmet demand** rather than net stock | **superseded**: the corpus's Model A needs no demand counter at all. A price read from the store cannot be fooled by an eater who takes nothing. |
| scale by **population** | done, and it is the denominator |
| research price adjustment from **excess demand** | done: the search returned the crate's own cited paper, §3.1.1, which was being half-implemented. `w` and `k` are documented as non-transferable. |
| its own **measurement** | done: the table above, the plot, and the village diff |

## What this does not do

- **The price does not drive decisions.** Deliberate: the village is bit-identical apart from the column, which
  is the check that this change is one change.
- **`price_full_store` is a design default.** It is the number the plot pass showed matters, so it is the number
  a host will want to tune; its doc carries both measurements.
- **`w` (a real market index) and `k` (±10% noise) are dropped**, and the module says why: an outside data feed
  and a source of nondeterminism, neither of which belongs in this crate.
