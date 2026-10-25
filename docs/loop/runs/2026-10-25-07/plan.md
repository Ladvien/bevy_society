# Plan: a basic economy — goods, stock, and a price that rests

## Pick, and why it beats the runner-up

**Pick:** the smallest economy that satisfies the literature's own acceptance test — **goods with a per-person
stock, production and consumption attached to tasks, and one price per good that rests when inflow equals
outflow and moves when it does not.**

**Why:** this is the last iteration's item 1 and the request that has now been repeated three times. The
research found the corpus already contains a game-economy model with a *falsifiable* property ("if inflow
equals outflow, the economy is stable and prices fluctuate around a constant value"), which means this can be
shipped with a test that is not a matter of taste.

**Runner-up:** `last_exchange_day` on `Tie` (fix the 43 drift-driven friendship endings). It is smaller, but
it needs a `SocietySave::VERSION` bump for one field, and this change needs the same bump anyway — so doing
both in one version step is cheaper than two.

**Rejected:** gating `eat` on having bread. Last iteration measured that gating a *dominant* activity on a new
condition wrecks the village (rivals 29 → 55). Bread must change what people **prefer**, never what they may
do.

## Design

### 1. Goods, as data

```ron
goods: [(id: "bread")],
```

Exactly like `needs`, `topics`, `deeds`. A good is not a currency and carries no exchange rate.

### 2. Stock, as state

`Stock(Vec<f32>)` on a person, one level per good, `[0, stock_ceiling]`. **This is the first per-person state
added since the save format was written**, so:

- `PersonSave` gains `stock: Vec<f32>`;
- `SocietySave::VERSION` goes **1 → 2**;
- `restore` already refuses a version it does not know, which is the crate's stated behaviour and the whole
  reason `VERSION` exists.

### 3. Production and consumption on the task

`ActivityDef` gains `produces: Vec<(String, f32)>` and `consumes: Vec<(String, f32)>`, both per use (not per
game-hour, because stock is a quantity and `satisfies` is already the rate-shaped field). `bake` produces
bread; the existing `eat` consumes it **if there is any** and never refuses when there is not — the preference
is a consideration, not a gate.

### 4. One price per good

`Prices(Vec<f32>)` as a resource, moved each society tick by excess demand:

```
price[g] += price_speed_per_day · dt · (demand[g] − supply[g]) / scale
```

with demand and supply counted from **what people actually did** in the tick — bread produced, bread eaten —
so the price is a measurement of the village, not a separate simulation. Clamped to a band around a base, so
it can never run away.

That is finding 1: equal inflow and outflow gives a zero delta and the price rests.

### 5. The new `Tuning` fields

| field | meaning | default |
|---|---|---|
| `price_speed_per_day` | how fast a price responds to imbalance | **design default, measured** |
| `stock_ceiling` | the most of one good a person can hold | **design default, measured** |

Both say "design default" in the code. The paper's own `w = 6.0` and "500 units" are for gnomes and are not
transferable.

### 6. Scripts and the snapshot

`ctx.stock(a, "bread")`, `ctx.price("bread")` — read-only, so a host's rules can react without the crate
deciding anything for them.

### Touched

| file | change |
|---|---|
| `src/defs.rs` | `GoodDef`, `goods`, `ActivityDef::produces/consumes` |
| `src/core.rs` | `Stock`, `Prices`, `Tuning::{price_speed_per_day, stock_ceiling}` |
| `src/economy.rs` | **new**: `Stock` component, `Prices` resource, `produce_and_consume`, `update_prices` |
| `src/behavior.rs` | the consumption term in an activity's score |
| `src/save.rs` | `stock` in `PersonSave`, `VERSION` 2 |
| `src/snapshot.rs`, `src/script.rs` | `ctx.stock`, `ctx.price` |
| `tests/economy.rs` | **new** |

### Tests

1. **Production and consumption move stock**: a bake/eat village's bread stock rises and falls, not constant.
2. **The price rests at balanced inflow**, and **rises under undersupply** — the paper's own property, as two
   configurations.
3. **Nobody is blocked**: a villager with an empty stock still eats.
4. **The village does not regress**: friends, rivals, rumor reach and interruptions unchanged.
5. **Save round trip**: stock survives `capture` → JSON → `restore` bit-identically, and a version-1 save is
   refused with `RestoreError::Version`.
6. Determinism.

## Measurement plan

`village-evening`, seeds 7 / 42 / 11: the four regression numbers, plus bread stock over time and the price of
bread. A dedicated `economy` scenario with two production rates (balanced and under-supplied) to show the
price resting in one and climbing in the other.

Plots: (a) bread stock over a run; (b) the price of bread in the balanced and the under-supplied village;
(c) the four regression numbers.

## Not doing

- **Trade between villagers.** Finding 2 says pairwise barter is the honest shape and `Interact` already is
  that, but a `trade` interaction needs a resistance price and a profit rule, both unsourced. Recorded.
- **Money.** A good is not a currency (finding 3: meat is shared, money is not).
- **Needs re-expressed as goods.** That breaks every existing pack's `satisfies` and was rejected in
  `needs-decision/professions-and-economics.md`.
