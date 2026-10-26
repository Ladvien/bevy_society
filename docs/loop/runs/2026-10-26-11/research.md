# Research: a price read off the stores, per head

## Why this

The log's first remaining item, carried by four consecutive iterations: **the price follows net stock and
counts units, not heads**. It was recorded as needing "its own counter (`Flow.unmet`), its own research
(excess-demand price adjustment) and its own measurement".

The research found that the crate already cites the paper that answers it, and was implementing the wrong
half of it. `eth-cgl-ar_games-Zun16a` (*Gnome Trader*, in the corpus, converted) is cited in `src/economy.rs`
for its acceptance test. Its §3.1.1 then gives the model that satisfies that test — a price computed from the
**storage ratio**, not from a flow.

## The source

**Gnome Trader** (corpus doc `eth-cgl-ar_games-Zun16a`, 2008), §3.1.1 "Model A: Production-Consumption":

> "In the Production-Consumption model, each gnome continuously produces or consumes its resources at a fixed
> rate until the storage is full or empty, respectively. The resource price `p` is directly calculated from
> the storage ratio `r = S_cur / S_max`."

with

$$p = w \cdot v(r) + k, \qquad
v(r) = \begin{cases} -2(r - 1), & r < 0.5 \\ 1.5 - r, & \text{otherwise} \end{cases}$$

and the paper's own gloss on `v`:

> "The price modifier term `v(r)` **doubles the price if the storage is empty and halves the price if the
> storage is full**."

The acceptance test the crate already cites (§4.2.1 / the production-consumption discussion):

> "If the parameters are chosen such that the **inflow of resources into the system equals the outflow**, then
> the economy is **stable and prices fluctuate around a constant value**. However, by upsetting this balance
> we can simulate a variety of interesting real market phenomenon, such as **shortages and oversupply**."

**Why this shape satisfies that test by construction, where the flow integral only satisfied it in principle.**
A constant flow with equal inflow and outflow leaves the *store* constant; a price that is a function of the
store is therefore constant too. The old `Δprice = speed · (produced − consumed)` was zero-delta only while the
two counters happened to cancel, and it cancelled the wrong quantities: `consumed` is what `Stock::take`
actually removed, so an eater with an empty shelf contributes nothing and a genuine shortage reads as zero
imbalance.

## Parameters the paper reports, and what transfers

| paper | value | transferable? |
|---|---|---|
| `v(r)` | piecewise, `2(1−r)` / `1.5−r` | **Yes, verbatim.** Both branches meet at `r = 0.5`; empty → 2.0, full → 0.5. |
| `S_max` | "the storage size" | **Yes in spirit, no in value.** "The storage size" of a producer that fills it over a whole run. A village's `Tuning::stock_ceiling` is a *clamp*, not a full store — measured below, the village holds a quarter of a unit a person against a clamp of six. |
| `w` | the real market index (`3 + 1.9 atan(0.005(I − I_AVG60))`, SMI) | **No.** Needs an outside data feed, and the crate must not have one. Dropped; nominal unit is `1.0`. |
| `k` | ±10% randomisation | **No.** Determinism forbids it. Dropped. |
| `w = 6.0`, 500 units | its gnomes | **No.** Different simulation. |

So the crate takes `v` exactly, reads `r` as **mean held per person over a day's supply**, and drops `w` and
`k` with a note. `tests/economy.rs` and `src/model/price.rs` hold `v` against the paper's three named points.

## The `S_max` question, measured

This is the one number that is a design default, and the measurement that chose it is the iteration's own
plot pass. Pricing `r` against the storage clamp (six units a person):

- village stores never exceed **0.71 of a loaf a person** over 17 game-days (seed 7, `mean bread per person`);
- so `r ≤ 0.12` always, and `v` never leaves **1.76 – 2.00** — a standard deviation of **0.05** in the price;
- the price reads the same in a glut and a drought, which is the exact defect being repaired, one level down.

Against a day's supply a head (`Tuning::price_full_store = 1.0`) the same stores span **1.0 – 2.0**. The
reading is "how many days does the village have in hand?", which is the question a price is supposed to answer.

## What believable output looks like

- A fed village's price **rests**: constant store, constant price. No clipping at either end of the band.
- A starved village's price sits at the **famine end**, near double, and stays there.
- The price **moves with the day**: the village bakes at dawn and eats through the day, so the store falls and
  the price rises, and the cycle repeats — not a random walk across the whole band.
- The same six loaves price **dearer** in a village of thirty than in one of three.
- Trade moves it **barely**, because a transfer moves no total; what it moves is the spread of stores.

## Sources

- Gnome Trader / production-consumption economy: corpus doc `eth-cgl-ar_games-Zun16a` (converted). §3.1.1
  Model A, Eq. `p = w v(r) + k`; the `v(r)` gloss; §4.2.1 the stability claim.
- Parunak et al., Edgeworth barter (corpus `10.1007_11678809_10`) — unchanged, and cited for why there is no
  auctioneer.
- Hall & Milgrom via Hall 2005 (`10.3386/w11678`) — unchanged, the joint-surplus precondition for a trade.
- Henrich et al. 2005 (`10.1017/S0140525X05000142`) — unchanged, goods are not money.
