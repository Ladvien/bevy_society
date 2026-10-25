# Research: professions and a basic economy

A deep read of the corpus for a mechanic that makes villagers' *work* mean something. Three findings, in
order of how much they change the design.

## 1. A task is not a skill

**Acemoglu, D. & Autor, D. (2010).** *Skills, Tasks and Technologies: Implications for Employment and
Earnings.* NBER Working Paper 16082, DOI `10.3386/w16082` (in the corpus, converted, read in full).

> "A **task** is a unit of work activity that produces output (goods and services). In contrast, a
> **skill** is a worker's endowment of capabilities for performing various tasks. Workers apply their
> skill endowments to tasks in exchange for wages, and skills applied to tasks produce output."

This is the finding that decides the schema. A profession must not be a *label on a person* — that is a
skill with no task. It has to be a **set of tasks**, and what makes one person better at a trade than
another is a **skill applied to tasks**.

The paper also gives the mechanism the crate can already express:

> "Relative wages of high to medium and low skill workers are determined by relative supplies and task
> allocations... **Workers have different comparative advantages, a feature that make our model similar to
> Ricardian trade models.**"

So: a task is an activity, a skill is a `Tuning`-free per-person scalar, and *which* profession a person is
good at is a comparative advantage — one number each, and the market sorts them. That is Becker's
specialisation argument (**Becker 1965**, cited in Solaz 2005, DOI `10.1007_s11150-005-4941-1`, in the
corpus: "the members of a family allocate their time according to their **comparative advantage** in the
production of market or non-market goods"), and **Durkheim 1964**, same doc: "differentiated economic roles
to new forms of social solidarity based on **economic interdependence**."

**What that means for this crate.** It already has the whole machinery:
- An activity is a task: `work` is `target: Place("work")`, `duration_hours`, `satisfies`, `barks`.
- A consideration is already a skill test: `(input: Trait(Agreeableness), curve: Linear(0.8, 0.4))`.
- So a *profession* is a **tag**, and a *skill* is a **consideration input** the crate does not yet have.

No new component for skills is needed if the skill is expressed as a consideration the definitions file
can name. The one gap: `Input` has `Trait(Trait)` — Big Five — but nothing for "how good is this person at
this task".

## 2. There is no auctioneer in a village

**Parunak, H. V. D. et al.**, on market topologies (corpus doc `10.1007_11678809_10`, converted):

> "A less-studied alternative to auction systems is the pattern of **Edgeworth barter**, in which buyers and
> sellers interact directly with each other in repeated pairwise transactions, **without the benefit of an
> auctioneer**."

A price-clearing auction is the wrong model for 24 people, and it would be a lie about how a village works.
Pairwise barter is also *free* in this crate: `Interact { actor, target, interaction }` is exactly a
pairwise transaction, and `InteractionResolved` is already the record.

**And the game-economy literature gives the tuning shape.** *Games and AI* (same corpus line,
`eth-cgl-ar_games-Zun16a`) models a producer/consumer economy with a market index `w`:

> "If the parameters are chosen such that the inflow of resources into the system equals the outflow, then
> the economy is **stable and prices fluctuate around a constant value**. However, by upsetting this balance
> we can simulate a variety of interesting real market phenomenon, such as **shortages and oversupply**."

That is the honest target for a basic economy: prices that move when supply and demand diverge, and a
stable constant when they do not. One index, not a market microstructure.

## 3. What people actually do with goods is *give* them

**Henrich, J. et al. (2005).** *"Economic man" in cross-cultural perspective: behavioral experiments in 15
small-scale societies.* Behavioral and Brain Sciences 28(6), DOI `10.1017/S0140525X05000142` (in the
corpus, converted).

> "If heuristics for sharing depend on the goods — **meat and honey are meticulously shared** among the Ache,
> but goods purchased by money are not (Henrich et al. 2004) — then the observed behavior ... should also
> depend on the kind of pie."

Two mechanics fall out, and both are *cheap*:

- **Sharing is not a market.** A villager with a surplus of a good shares it, and that is a deed — the crate
  already has `deed`, `harm`, witnesses, `NormViolated` and reputation. A gift that costs the giver is a
  **negative `harm` with a positive `topic`**, which the deed model already scores.
- **Goods are not fungible.** Meat is shared and money is not. A good can be marked shareable, and that is
  one bool in the schema rather than a currency.

**Mauss**, cited in Egerer 2023 (DOI `10.3389_fsoc.2023.1199474`, in the corpus): "the **reciprocity** has
been established as one of the main functions of a gift". A gift creates an obligation, which is the tie
model — and last iteration's `dyad_balance` now makes a one-sided obligation visible.

## What the literature does *not* give

- **No number** for a wage, a price elasticity, a production rate, or how strongly a skill should bias a
  task choice. Every one of those is a **design default**, and the crate's rule says so in the code comment.
- No evidence that a *small* village should have a market at all. Durkheim's interdependence is about
  differentiated roles, not about prices.

## What believable output looks like

- **Different professions do different things.** Today every villager's `work` is `Hour(8, 17)` at any place
  tagged `work`; a miller and a weaver are the same person in a different colour. A profession must change
  the *activity mix* measurably.
- **A skill biases the task choice**, so the best miller mills and the best weaver weaves, and the split
  matches comparative advantage rather than assignment.
- **A price moves when supply and demand diverge** and rests within a band when they match — the stable
  constant of the game-economy model.
- **Existing behaviour does not regress**: `village-evening`'s friends, rivals, rumor reach and
  interruptions stay where they are, because a profession is additive.
- Determinism holds; the schema change is additive and every new field defaults to "off".
