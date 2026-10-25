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

## What the model leaves out

- Money, prices, wages, inventories and trade. No good is scarce and nothing is exchanged for anything.
- **Skills that improve with practice.** The level is a trait reading, constant for a life. Learning needs
  stored state and a save field.
- Assignment: there is no threshold below which a villager cannot take a task. The literature is explicit
  that the allocation is endogenous, and gating on a trait was **measured to hurt the village** when applied
  to its dominant activity (see `docs/loop/runs/2026-10-25-06/eval.md`).
- Occupations as a social category, wages as a source of status, and the labour-market dynamics the paper is
  actually about.

## Open contradictions and departures

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
- Production/consumption game economy · `eth-cgl-ar_games-Zun16a` (converted)
- Becker, G. (1965), on comparative advantage within a household, as cited in Solaz (2005). https://doi.org/10.1007/s11150-005-4941-1 · `10.1007_s11150-005-4941-1` (converted)
- Durkheim, E. (1964), differentiated roles and economic interdependence, as cited in Stalker (2010). https://doi.org/10.29173/cjs6661 · `10.29173_cjs6661` (converted)
