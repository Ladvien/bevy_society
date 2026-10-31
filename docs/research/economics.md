# Economics and trades

## What bevy_society does

A person can be good at a **trade**. The definitions file declares `skills`, each bound to one personality
trait; an activity may declare the `profession` it is a **task** for; and a consideration may read
`Skill("craft")`, so a diligent villager scores a craft task higher than an idle one.

There is **no money, no inventory and no price**. The crate models *who does what*, not *what it is worth*.

## The distinction this rests on

- **Acemoglu, D. & Autor, D. (2010).** *Skills, Tasks and Technologies: Implications for Employment and
  Earnings.* NBER Working Paper 16082. DOI `10.3386/w16082` (in the corpus, converted, read in full).

  > "A **task** is a unit of work activity that produces output (goods and services). In contrast, a
  > **skill** is a worker's endowment of capabilities for performing various tasks. Workers apply their
  > skill endowments to tasks in exchange for wages, and skills applied to tasks produce output."

  and, on how the two are matched:

  > "**Workers have different comparative advantages**, a feature that make our model similar to Ricardian
  > trade models... the assignment of skills to tasks is endogenous."

  The crate's `ActivityDef` is the task; `SkillGroup` is the endowment; `Input::Skill` is the matching term.
  The crate does **not** implement the paper's wage determination, its three skill tiers, or its technical
  change — those need a labour market, and this crate has none.

## Equations as implemented

- **Skill level** (`skill_level` in `src/core.rs`): `skill = trait(skill_group)`, clamped to `[0, 1]`, where
  the group reads one Big Five trait:

  | group | trait | reads as |
  |---|---|---|
  | `Craft` | conscientiousness | diligence at a steady trade |
  | `Creative` | openness | invention |
  | `Social` | extraversion | dealing with people |

  **Nothing is stored.** The level is a pure function of personality, so two people with the same traits are
  equally good at the same trade, and no skill state has to be saved or kept deterministic. That is the
  whole reason this addition needs no `Component` and no `SocietySave` change.

- **Choice**: a consideration `(input: Skill("craft"), curve: Linear(0.9, 0.05))` turns the level into a
  utility factor, multiplied into the activity's score exactly like every other consideration. A villager at
  `0.95` conscientiousness scores that task near 1.0; one at `0.05` scores it near 0.05 and chooses
  something else.

## Why this and not a market

The corpus has the market model, and it is recorded rather than built:

- **Edgeworth barter** — "buyers and sellers interact directly with each other in repeated pairwise
  transactions, **without the benefit of an auctioneer**" (corpus doc `10.1007_11678809_10`). Pairwise
  exchange is what a village has and what `Interact` already is.
- **A game-economy production/consumption model** — "If the parameters are chosen such that the inflow of
  resources into the system equals the outflow, then the economy is **stable and prices fluctuate around a
  constant value**... by upsetting this balance we can simulate a variety of interesting real market
  phenomenon, such as **shortages and oversupply**" (corpus doc `eth-cgl-ar_games-Zun16a`). That is the
  honest target: one index that moves, not a microstructure.
- **Goods are not fungible** — "meat and honey are meticulously shared among the Ache, but goods purchased by
  money are not" (Henrich et al. 2005, DOI `10.1017/S0140525X05000142`, in the corpus). A gift that costs the
  giver is already expressible as a deed with a negative `harm` and a positive topic; sharing needs no
  currency.

All three are written up in `docs/loop/needs-decision/professions-and-economics.md` with the measurements
that must precede them.

## The economy: goods, stock and a price that rests

### The acceptance test is the literature's own

**Games and AI** (corpus doc `eth-cgl-ar_games-Zun16a`, converted), on its production/consumption model:

> "If the parameters are chosen such that the **inflow of resources into the system equals the outflow**, then
> the economy is **stable and prices fluctuate around a constant value**. However, by upsetting this balance
> we can simulate a variety of interesting real market phenomenon, such as **shortages and oversupply**."

So a price here is not decoration: it is asserted to rest when production matches consumption and to move
when it does not, in `tests/economy.rs`.

### What is implemented

- **A good** (`goods: [(id: "bread")]`) is a quantity, not a currency. "Meat and honey are meticulously
  shared among the Ache, but goods purchased by money are not" — Henrich et al. 2005. No exchange rate.
- **`Stock`** — one level per good per person, in `[0, Tuning::stock_ceiling]`. It is **state**, so it is in
  `PersonSave` and `SocietySave::VERSION` is **2**.
- **`produces` / `consumes` on an activity**, per **completed** use. An interrupted task produces nothing,
  the same rule `satisfies` follows.
- **`Prices`**, one per good, read off **how full the village's stores are, per head**:
  `price = nominal · v(total_held / (people · price_full_store))`, clamped to
  `[price_floor, price_ceiling]`. `v` is the paper's own storage-ratio term, and `price_full_store` is a
  **design default** of one unit a head — a day's supply — with the measurement that chose it below.
- **`GoodDef::initial`** — what a person holds when created (default `0.0`; the starter pack's bread is `1.0`),
  held at `stock_ceiling`. It exists because a society that begins with every larder empty has a start-up
  transient the running society does not: the 24-villager village at seed 99 held no bread for 22 game-hours,
  and game-day 2 had **38.7 %** of its villager-ticks with `food` below 0.05 against **1.4-10.7 %** on days
  3-10 (`docs/loop/runs/2026-10-30-27/`). Starting in a state typical of the running system is the standard
  remedy for initialization bias (Murray & Kelton 1988, DOI 10.1145/318123.318252), and a household that holds
  a store is the typical state (Winterhalder, Puleston & Ross 2015, DOI 10.1179/1749631415y.0000000025; the
  first seasonal strategy of a rural household is "drawing down of stored product", Longhurst 1986, DOI
  10.1111/j.1759-5436.1986.mp17003005.x). No paper gives a number of loaves: **1.0** is a **design default**
  where three of the pack's own numbers agree — `price_full_store` (a full store), `trade_reserve` (a giver
  must hold *more than* it to share, so a village begins with no surplus to give) and a meal of a quarter loaf
  at about one meal a day (four days of food, enough to reach the first dawn bake). A saved person is not a
  new one: `restore` puts back what was saved, and an empty saved stock restores as holding nothing.

**Why the price is a function of the stores and not an integral of the flow.** It used to be
`Δprice = price_speed_per_day · (produced − consumed)` **per interval**, and two defects were measured
against it. `consumed` is what `Stock::take` actually *removed*, so the difference was **net stock**: a
village that baked more than it ate saw its price rise, and a real shortage was invisible because an eater
with an empty shelf takes nothing and adds nothing. It also counted **units, not heads**, so the same six
loaves were a glut in a village of three and a famine in one of thirty — and with a three-loaf `bake` the
price spent **28% of its ticks on the floor**.

**The corpus's Model A prices from the storage ratio instead, and the paper says what it means**
(*Gnome Trader*, `eth-cgl-ar_games-Zun16a` §3.1.1). Each agent's price is

$$p = w \cdot v(r) + k, \qquad r = \frac{S_{\text{cur}}}{S_{\text{max}}}, \qquad
v(r) = \begin{cases} -2(r - 1), & r < 0.5 \\ 1.5 - r, & \text{otherwise} \end{cases}$$

and its own gloss: *"The price modifier term `v(r)` **doubles the price if the storage is empty and halves
the price if the storage is full**."* A full store is half price, an empty one double, half full the nominal
unit — and because `r` is a fraction of capacity rather than a count of units, that is a scarcity signal
**per head** by construction.

**What does not transfer, and what the crate drops.** `w` is the paper's real-world market index (a 60-day
moving average of the SMI), which needs an outside data feed the crate must not have; `k` is a ±10%
randomisation, which determinism forbids. Both are dropped, and the nominal unit is `1.0` — already the
documented value for a good nobody has traded. The village-wide `r` is
`total_held / (people · price_full_store)`: the denominator is a supply per head, so the same six loaves are
dearer in a village of thirty than in one of three. `v` itself is implemented unchanged in
`src/model/price.rs`, tested against the paper's three named points (empty → 2.0, half → 1.0, full → 0.5).

**`S_max` is the one number that does not transfer, and the obvious reading of it is wrong.** The paper's
`S_max` is "the storage size" of a producer that spends a whole run filling it, so its `r` spans the unit
interval. A village's `Tuning::stock_ceiling` (six units a head) is a *clamp*, and the village holds a
quarter of a loaf a person: measured over 17 game-days at seed 7 the mean never exceeds **0.71**, so
`r ≤ 0.12` always and `v` never leaves **1.76 – 2.00** (price standard deviation **0.05**). That pins the
price at the famine end whatever the village does — a scarcity index that reads the same in a glut and a
drought, the defect this change repairs, one level down. `price_full_store = 1.0` a head spans
**0.79 – 2.00** on the same stores, which is why it is the shipped default. Both numbers are measured in
`docs/loop/runs/2026-10-26-11/eval.md`.

**The acceptance test still holds, by construction rather than by a rate.** A village whose inflow equals its
outflow holds a constant store, and a constant store has a constant price. `tests/economy.rs` runs the same
village balanced and starved and asserts the first rests and the second prices near double.

### Trade between villagers

A good now changes hands between two people, on the same `Interact` path every other exchange takes.

- **`actor_gives` / `target_gives`** on an interaction's `on_accept` are lists of `(good, amount)`. Both sides
  move in one call and each moves what it has: `Stock::take` reports what left and `Stock::add` what landed, so a
  pair cannot create a good. Declared in the data, so no system decides who trades with whom — `Perceive`
  picks the pair and the acceptance draw decides whether it happens.
- **`Surplus("bread")`**, a consideration input, is the **joint surplus** of a transfer from this person to the
  target: `value(target's store) − value(own store)` with `value(held) = 1 / (1 + held)`, in `[0, 1]`. It is
  `0` unless the giver holds **more than** `Tuning::trade_reserve` (1.0) and the gap is at least
  `Tuning::trade_margin` (0.1). It needs a `Person` target.

**Where the literature stops, and the code says so.**

- **Topology: Parunak et al.** (corpus `10.1007_11678809_10`): "a less-studied alternative to auction systems
  is the pattern of **Edgeworth barter**, in which buyers and sellers interact directly with each other in
  repeated pairwise transactions, **without the benefit of an auctioneer**". Pairwise encounters decided by
  perception, no market place. Its emergent claim is that repeated trade "balances supply and demand".
- **Precondition: Hall (2005)**, reporting Hall & Milgrom (NBER WP 11678, DOI `10.3386/w11678`): "once ... they
  have met and found that they would enjoy a **joint surplus**, the threats to disclaim the match are
  hollow". A transfer is offered only when there is a surplus to split, which is what `Surplus` measures.
- **Diminishing marginal value** is the textbook shape and needs no source: a person with an empty shelf
  values a loaf at `1.0`, a person with one at `0.5`. No curve, no utility function, no new rate.

**What the village actually does with it is *sharing*, not a swap, and that is deliberate.** The village has one
good. A one-way transfer of it is what Henrich et al. (2005) report from the small-scale societies — "meat and
honey are meticulously shared" — and the giver is paid in standing, not goods: the interaction's `trust`
deltas and `Liking`/`Joy` emotions, which are the "indirect" half of reciprocity. Two goods make it a swap and
the engine already does that (`target_gives`; `a_barter_moves_a_good_each_way_and_conserves_both`). The
village has no second good because nothing would consume it, and an unconsumed good is a store that only
fills.

**Trade moves the price, now, and only slightly — which is the model working.** Parunak's claim reads like
"repeated trade balances supply and demand". A transfer moves no store total — one person's loss is another's
gain — so a price read off the **village-wide** store is barely touched by it: what a transfer changes is the
**spread** of stores, `trade_narrows_the_spread_of_stores` (one holder of four loaves, three with none) ending
with a standard deviation under 60% of the control's. The price moves only while a share is in flight — a
loaf taken by the giver and not yet added to the receiver briefly leaves the total — which is a rounding-level
effect and is not a mechanism to rely on. Recorded rather than dressed up: the price is a village-level
observation and trade is a redistribution inside the village.

**The split of the surplus is not a number.** Hall & Milgrom's resolution is that the parties alternate offers
and the threat is delay, which needs a patience parameter the crate does not have. With a one-way share there
is nothing to split; with a swap, each side gives what the interaction names and the acceptance draw is the
only bargaining there is. Recorded rather than invented.

### What it is not

- **No money**, per Henrich above.
- **No bargaining.** The paper describes the *process*, with alternating offers; a village does not need twelve
  rounds of haggling to pass a loaf.
- **No market place and no auctioneer.** Pairwise encounters only.
- **Nothing gates.** An empty store never stops a villager eating; measured, gating a dominant activity is
  what wrecked the village in the professions iteration. **It does stop the meal feeding them**: `satisfies`
  is paid in proportion to what `consumes` actually found, `min(1, held / asked)`, which is the crate's own
  satiation term rather than a prohibition. `docs/loop/runs/2026-10-26-12/` has the measurement — the village
  used to bake 4.8 loaves a game-day and eat 21 meals, so ~77% of meals were conjured from an empty shelf.

### Design defaults, all of them

`stock_ceiling` (6.0), **`price_full_store` (1.0)**, `price_floor` (0.25), `price_ceiling` (4.0),
`trade_reserve` (1.0) and
`trade_margin` (0.1) have **no source**. The paper's own `w = 6.0` and "500 units" are for its gnomes. They
are chosen by measurement — see `docs/loop/runs/2026-10-25-07/eval.md`, `docs/loop/runs/2026-10-25-08/eval.md`
and `docs/loop/runs/2026-10-26-11/eval.md`. The price *shape* is not a default: it is the paper's `v`, and
`src/model/price.rs` tests it against the three points the paper names.

## What the model leaves out

- Money, wages and inventories beyond a per-person level. A price is an observation of the village's own
  flows, not a market's.
- **Skills that improve with practice.** The level is a trait reading, constant for a life. Learning needs
  stored state and a save field.
- Assignment: there is no threshold below which a villager cannot take a task. The literature is explicit
  that the allocation is endogenous, and gating on a trait was **measured to hurt the village** when applied
  to its dominant activity (see `docs/loop/runs/2026-10-25-06/eval.md`).
- Occupations as a social category, wages as a source of status, and the labour-market dynamics the paper is
  actually about.

## Open contradictions and departures

- **`price_full_store = 1.0` reads the project's own village as permanently short.** A day's supply a head is
  the default's reading of "full", and the village holds a **quarter** of a loaf a head, so its median price is
  **1.50** and it never once reaches the half-price glut end. Recomputed directly from the store series in
  `docs/loop/runs/2026-10-26-11/evidence/after/s7_p24/series.csv` at four defaults, the store's median of 0.25 a
  head sits on the nominal unit at `0.5` (median price 1.00) and at half price at `0.25` (0.50). The default is
  honest — "how many days does the village have?" has the answer *a quarter* — but it is the number a host will
  most want to tune, and moving it moves every price, so it has not been re-tuned on the evidence of one pack.
- **Production runs on the clock, not on the shelf, in the starter pack** (found 2026-10-30-28). `bake` is a one-hour dawn
  errand drawn against nine other options: it scores the hour and the `craft` skill, and nothing that tells a person
  how much bread it holds. That is an **open loop** — stable only when its output happens to equal what the village eats
  (Zunic, corpus doc `eth-cgl-ar_games-Zun16a` §4.2: "if the parameters are chosen such that the inflow of resources
  into the system equals the outflow, then the economy is stable ... by upsetting this balance we can simulate ...
  shortages"; Meadows, *Thinking in Systems*, on balancing loops closed on a stock). A town of sixty on a pack tuned at
  twenty-four made 3 bakes in ten game-days, and by game-day 12 held 0.05 loaves a head with 86 % of person-ticks at
  `food` below 0.05 (evidence harness's town, 240-second day). `Input::Stock(good)` lets a consideration read the actor's
  own holding, and `examples/assets/provisioning.society.ron` closes the loop (a hungry household with a bare shelf
  makes bread; eating from an empty shelf is priced at `0.05` of its score, not forbidden): the same town holds 1.3-1.5
  loaves a head. **Opt-in**, because the one pinned precondition it falsifies (`tests/starting_larder.rs` asserts an
  empty-larder village starves on its second day, and a self-provisioning one does not) is not this loop's to loosen:
  `docs/loop/needs-decision/households-provision-themselves.md`. Appetitive before consummatory (Woodworth 1918 and
  Konorski 1976, in corpus doc `10.1002/hipo.22488`); a smart object advertises only what its precondition allows
  (Zun15a, `eth-cgl-ar_games-Zun15a`); utility scores hunger *with* the availability of food (Game AI Pro, "Utility
  Systems", on *The Sims*).
- **The price does not drive behaviour.** No decision reads a price. That is deliberate in this change (it is
  what makes the bit-identical village a usable check) and it is the natural next step: an `Input::Surplus`-like
  term that makes a good dearer to skip in a shortage.
- **The trait-to-trade mapping is a design default.** No paper in the corpus assigns Big Five traits to
  occupations. It is three lines in `SkillGroup::trait_of` so a host can see and disagree with it in one
  place.
- **`Profession(String)` on a person is a label, not a mechanism.** It exists for scripts and barks; nothing
  in the simulation reads it, and the code says so.
- The crate's skill model is a **substitute for**, not an implementation of, Acemoglu & Autor's: they derive
  wages from task prices; this derives a utility factor from a trait.

## Sources

- Acemoglu, D. & Autor, D. (2010). Skills, Tasks and Technologies. NBER WP 16082. https://doi.org/10.3386/w16082 · `10.3386_w16082` (converted)
- Henrich, J. et al. (2005). "Economic man" in cross-cultural perspective. *BBS* 28(6). https://doi.org/10.1017/S0140525X05000142 · `10.1017_s0140525x05000142` (converted)
- Market topologies, incl. Edgeworth barter · `10.1007_11678809_10` (converted)
- Hall, R. E. (2005). Job Loss, Job Finding, and Unemployment in the U.S. Economy Over the Past Fifty Years (reporting Hall & Milgrom on credible threat points). NBER WP 11678. https://doi.org/10.3386/w11678 · `10.3386_w11678` (converted)
- Production/consumption game economy · `eth-cgl-ar_games-Zun16a` (converted)
- Becker, G. (1965), on comparative advantage within a household, as cited in Solaz (2005). https://doi.org/10.1007/s11150-005-4941-1 · `10.1007_s11150-005-4941-1` (converted)
- Durkheim, E. (1964), differentiated roles and economic interdependence, as cited in Stalker (2010). https://doi.org/10.29173/cjs6661 · `10.29173_cjs6661` (converted)
- Murray, J. R. & Kelton, W. D. (1988). Initializing for bias reduction. https://doi.org/10.1145/318123.318252 (abstract read from OpenAlex; not in the corpus)
- Winterhalder, B., Puleston, C. O. & Ross, C. T. (2015). Production risk, inter-annual food storage by households and population-level consequences in seasonal prehistoric agrarian societies. https://doi.org/10.1179/1749631415y.0000000025 (abstract read from OpenAlex; not in the corpus)
- Longhurst, R. (1986). Household Food Strategies in Response to Seasonality and Famine. https://doi.org/10.1111/j.1759-5436.1986.mp17003005.x (abstract read from OpenAlex; not in the corpus)
