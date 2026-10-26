# Plan: trade between two villagers

## Pick

**Pick:** a `trade` activity that transfers a good between two people when their **valuations of it differ
enough**, splitting the surplus.

**Why:** it is the last gap in the economy and the request's own words — "basic economics". The price
exists and has no counterparty. Finding 1 says the emergent claim to test is that "in repeated trades,
[it] balances supply and demand", which is a falsifiable prediction about a number I already measure.

**Runner-up:** fixing the `O(n·endings)` scan in `produce_and_consume` (item 2 last iteration). One line,
no host-visible behaviour, and unexercised by the benchmark — worth doing, not worth an iteration.

## Design

### Valuation falls as the store rises

`value(stock) = 1 / (1 + stock)` — one `Tuning`-free expression, no curve. A person with none values a unit
at 1.0; a person with a full store values it at `1/(1+ceiling)`. That is the textbook shape (diminishing
marginal value) with **no new rate** and no utility function.

### A trade needs a surplus to split

For a good `g` and a pair `(a, b)`:
- `a` sells if `value(a.held − 1) < value(b.held + 1)`… no: simpler and symmetric — **`a` gives a unit if
  `a` holds at least `trade_reserve` and `b` holds less**. The *size* of the incentive is
  `value(b) − value(a)`, and the trade happens when that gap is at least `trade_margin`.

That gap is exactly the joint surplus Hall & Milgrom require the parties to have found before bargaining
begins, and `trade_margin` is a **design default**.

### The split, and why there is no threat to walk

Finding 2: the threat point is *delay*, not disagreement, so the split is by relative patience. The crate
has no patience, so the split is `trade_share` — the fraction of the surplus the *seller* keeps — as a
**design default**, and the code says so in the sentence that would otherwise claim the paper gave it.

### Touched

| file | change |
|---|---|
| `src/economy.rs` | `value_of`, the `trade` resolution path, `TradingPlugin` wiring |
| `src/core.rs` | `Tuning::{trade_margin, trade_reserve, trade_share}` |
| `examples/assets/village.society.ron` | a `trade` activity and its interaction |
| `tests/trade.rs` | **new** |

### Tests

1. **A trade moves a good both ways and leaves both sides better off by valuation.** Arithmetic, not vibes.
2. **No surplus, no trade**: two people with the same store do not trade.
3. **The price stops running to its clamp once trade is possible** — finding 1's emergent claim, measured
   against a village with trade disabled.
4. The four regression numbers do not move.
5. Determinism.

## Not doing

- **Money.** Unchanged from last iteration: Henrich's goods are not a currency.
- **A market place.** Finding 1 says pairwise encounters; `Perception` decides who meets whom.
- **Bargaining with alternating offers.** The paper describes the *process*; a village does not need 12
  rounds of haggling to swap a loaf, and the split is one number either way.

## What changed from this plan, after building it

- **`trade_share` was never built.** A one-way share has nothing to split, and a swap has no patience parameter to
  split it by; the acceptance draw is the only bargaining. `Tuning` gained `trade_reserve` (1.0) and `trade_margin`
  (0.1) only, and **both are read** (`Signal::Surplus` is zero below either).
- **The unit of design is an input, not a system.** `Input::Surplus(good)` plus `actor_gives` / `target_gives` on
  `EffectsDef`; no new plugin and no new message. `Perceive` picks the pair and the acceptance draw decides.
- **The village does sharing, not barter.** It has one good, so a one-way transfer is Henrich et al.'s sharing, paid
  in trust. A two-way swap is built and tested (`a_barter_moves_a_good_each_way_and_conserves_both`).
- **Test 3 was wrong.** "The price stops running to its clamp" does not follow from a transfer, which neither
  produces nor consumes; and it measured the other way. What is true and tested is the **spread of stores**
  (`trade_narrows_the_spread_of_stores`; 38% lower in the village over nine seeds).
- **Test 4 changed from three seeds to nine paired seeds**, because three could not tell a change from the noise.
- **The pack changed more than the plan said**: `bake` makes three loaves and weighs `Skill("craft")`, because at one
  loaf there was never anything to share (20 shares in 17 days).
- **Added, not planned**: a `goods` clip, `shares.txt` evidence, and neighbours' stores are copied only when some
  activity reads `Surplus` (the perf gate).
