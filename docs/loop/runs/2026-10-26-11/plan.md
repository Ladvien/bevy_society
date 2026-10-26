# Plan: a price read off the stores, per head

## Why this over the runner-up

Runner-up was the `--people`-scales-people-not-the-village trap (HANDOFF item 2). It is a *harness* defect and
its fix is to refuse a flag; this is a library model that has been the log's first item for four iterations
and whose two defects are both measured and both visible in a host game (bread gets dearer when it piles up; a
famine is invisible). The price also has a consumer, now: `ctx.price` gives a host's triggers something to
react to.

## Design

### 1. Score the store, not the flow

`src/model/price.rs` (engine-free):

```
scarcity_factor(fill)      = 2(1 − r) for r < 0.5,  1.5 − r for r >= 0.5
storage_price(total, n, s_max, nominal) = nominal · scarcity_factor( (total / n) / s_max )
```

- `scarcity_factor` is the paper's `v(r)` verbatim, tested at its three named points.
- `storage_price` divides by **heads** and by `s_max` = a day's supply, which is the design default the plot
  pass chose (see `research.md`).

### 2. `update_prices` sums the stores and applies it

Sums `Stock` over `PersonIndex.sorted` (fixed order), divides by `index.sorted.len()`, applies
`storage_price`, clamps to `[price_floor, price_ceiling]`. No resource to keep between ticks, so no counter to
get out of step. `Tuning::price_speed_per_day` is **removed** (dead: its only reader was the flow integral) and
`Tuning::price_full_store` (1.0) is **added**. `Flow` is **removed** (its only reader was the same line).

### 3. `ctx.price(good)` and `ctx.stock(person, good)`

`Prices`' own doc has promised `ctx.price("bread")` since the economy landed and there was no such function. The
snapshot gains `prices` and each `PersonView` gains `stock`; `src/script.rs` registers the two functions. Both
read the **tick-start** snapshot, one tick behind `update_prices`, which is the same lag every other `ctx` read
has and is documented on the accessor.

## Touched modules

- `src/model/price.rs` — new; `v`, `storage_price`, 5 unit tests.
- `src/model/mod.rs` — `pub mod price`.
- `src/economy.rs` — `update_prices` rewritten; `Flow` and `produce_and_consume`'s flow bookkeeping removed.
- `src/core.rs` — `price_full_store` added, `price_speed_per_day` removed.
- `src/snapshot.rs` — `prices` on the snapshot, `stock` on `PersonView`, `stock`/`price` accessors, goods in `Names`.
- `src/script.rs` — `ctx.stock`, `ctx.price`.
- `src/lib.rs` — `Flow` un-exported.
- `tests/economy.rs` — the shortage test restated as a comparison; new band test unchanged.
- `tests/script_economy.rs` — new, 3 tests.

## Measurement

`examples/evidence.rs --scenario village-evening --seed 7` before (the pre-change binary, kept at
`~/.cache/price-before/evidence`) and after, 67 200 ticks, 24 people. Same seed, same pack, same settings.
`tools/plot_price.py` draws price and store on one pair of axes.

The prediction to test is **not** "the price moves more": it is that the price (a) still rests in a fed village,
(b) is *visible* rather than pinned, and (c) tracks the store rather than the flow. The village's decisions do
not read the price, so the society is expected to be **bit-identical apart from `price`** — and if it is not,
something else was touched by accident.

## Risk

- The price has no behavioural consumer, so "the village improved" is not a claim this iteration can make. The
  deliverable is a number a host can read, plus the read path.
- `price_full_store = 1.0` is a design default, and the plot pass already rejected one value of `S_max`
  (the storage clamp). It is expected to be the number a host most wants to tune.
