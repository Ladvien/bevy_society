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
- **`Prices`**, one per good, moved by
  `Δprice = price_speed_per_day · (produced − consumed)` **per interval**, clamped to
  `[price_floor, price_ceiling]`.

**Why the delta is per interval and not per tick.** The first version multiplied by `dt_days`, and an activity
ends only every few dozen ticks — so the price integrated one tick of imbalance against thirty of zero and
moved by `0.0016` in a 1 200-tick run. The counters accumulate between price updates and are cleared by the
update, which makes the price an integral of what the village did rather than of the last tick.

**Why scarcity is `produced − consumed` and not the reverse.** The first version had it inverted and an
undersupplied village showed its price *falling*: with a shared store the eaters who cannot find bread stop
eating, so consumption drops faster than production and the raw difference turns negative.

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

**The emergent claim, measured and corrected.** Parunak's claim reads like "trade moves the price". It does
not here, and the plan that said it would was wrong: a transfer neither produces nor consumes, so
`Δprice = speed · (produced − consumed)` is untouched by it. What a transfer changes is the **spread of
stores**: `trade_narrows_the_spread_of_stores` (one holder of four loaves, three with none) ends with a standard
deviation under 60% of the control's, and `docs/loop/runs/2026-10-25-08/eval.md` measures the same in the
village.

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
  what wrecked the village in the professions iteration.

### Design defaults, all of them

`stock_ceiling` (6.0), `price_speed_per_day` (0.5), `price_floor` (0.25), `price_ceiling` (4.0),
`trade_reserve` (1.0) and `trade_margin` (0.1) have **no source**. The paper's own `w = 6.0` and "500 units" are
for its gnomes. They are chosen by measurement — see `docs/loop/runs/2026-10-25-07/eval.md` and
`docs/loop/runs/2026-10-25-08/eval.md`.

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

- **The price follows net stock, which is the wrong way round, and it counts units, not heads.** `consumed` is
  what `Stock::take` actually removed, so `produced − consumed` is the net change in what villagers hold: a
  village that bakes more than it eats sees its price **rise**, and a real shortage is invisible, because an
  eater with an empty shelf takes nothing and adds nothing. The sign was flipped in the economy iteration to make
  the shortage case of `tests/economy.rs` pass, and the reason recorded there ("eaters who cannot find bread stop
  eating") describes this artefact rather than explaining it. Found on `2026-10-25-08`: with a three-loaf `bake`
  the village's price spends **28% of its ticks on the floor** (1% at one loaf), because `Δprice = 0.5 × produced`
  moves it 1.5 in one step against a 3.75 band, and sharing does not change that. The repair is to count **unmet
  demand** (what an eater asked for and could not take) as the scarcity signal and to scale by population; that
  needs its own counter, its own research (price adjustment from excess demand) and its own measurement, and it
  is the first remaining item in `docs/loop/log.md`.
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
