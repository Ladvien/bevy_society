# Research: a basic economy

The question this iteration answers: what is the smallest economy that makes a village's work *matter*?
Three findings shape the answer.

## 1. Prices rest when inflow equals outflow, and move when it does not

**Games and AI** (corpus doc `eth-cgl-ar_games-Zun16a`, converted). A production/consumption model of an
agent town:

> "If the parameters are chosen such that the **inflow of resources into the system equals the outflow**,
> then the economy is **stable and prices fluctuate around a constant value**. However, by upsetting this
> balance we can simulate a variety of interesting real market phenomenon, such as **shortages and
> oversupply**."

> "This imbalance results in an undersupply of resources that causes the **producer prices to rise
> continuously**. The shortage causes consumer prices to spike after about 140 days."

That is the target and, importantly, the *test*: a good economy in this crate must (a) rest at a constant
when production matches consumption, and (b) move when it does not. One index per good, not a
microstructure. The paper also supplies the trading heuristic — agents "try to maximize profits by
conservatively selecting the best trading opportunities along that path" and "avoid selling a resource for
less than it was bought... which is represented by the **resistance price**" — a seller's floor.

## 2. There is no auctioneer in a village

**Parunak et al.**, market topologies (corpus doc `10.1007_11678809_10`, converted):

> "A less-studied alternative to auction systems is the pattern of **Edgeworth barter**, in which buyers and
> sellers interact directly with each other in repeated pairwise transactions, **without the benefit of an
> auctioneer**."

A clearing auction over 24 people would be a lie about scale. Pairwise exchange is also free in this crate:
`Interact { actor, target, interaction }` is already a pairwise transaction and `InteractionResolved` the
record of it.

## 3. Goods are not fungible, and sharing is not selling

**Henrich et al. (2005)**, *"Economic man" in cross-cultural perspective*, BBS 28(6), DOI
`10.1017/S0140525X05000142` (in the corpus, converted):

> "If heuristics for sharing depend on the goods — **meat and honey are meticulously shared** among the Ache,
> but goods purchased by money are not (Henrich et al. 2004) — then the observed behavior ... should also
> depend on the kind of pie."

So a good carries a **shareable** flag: some goods are given freely and some are only traded. That is one
field in the schema, not a currency, and it is what stops a village's economy from being a single price
vector with nothing social in it.

**Mauss**, via Egerer (2023, DOI `10.3389_fsoc.2023.1199474`, in the corpus): "the **reciprocity** has been
established as one of the main functions of a gift". A gift that costs the giver is already expressible as a
deed — negative `harm`, positive `topic` — so reputation responds to generosity without new machinery.

## The design this implies

Four pieces and no more:

1. **A good** — a definition list, `goods: [(id: "bread")]`. Not a currency.
2. **A stock** — per person, one scalar per good. It is *state*, so it must be saved and restored.
3. **Production and consumption on the task.** Last iteration established that a task is an activity; this
   adds `produces:` and `consumes:` to it, so a `bake` task produces bread and eating consumes it.
4. **A price per good** that moves with total stock, resting when production matches consumption — finding 1
   as the acceptance test rather than a flourish.

## What the literature does not supply

- **No number** for a production rate, a consumption rate, a stock ceiling, or how fast a price moves. All
  are **design defaults** chosen by measurement, and each says so in the code.
- No evidence that a village of 24 needs a *market place*. Finding 2 says pairwise barter is the honest
  shape; finding 3 says some goods never enter trade at all.
- Nothing about a **resistance price** in the crate's terms — that is a seller's floor and it is one
  `Tuning` field if the measurement says it earns its place.

## What believable output looks like

- **Bread is produced and eaten, and the stock moves.** Not a constant, not zero.
- **The price rests when production matches consumption** and rises when it does not — finding 1's own test,
  run as two configurations rather than asserted.
- **A villager without bread still eats**, because gate-keeping the dominant activity was measured to
  wreck the village last iteration. Bread should change *what people prefer*, not whether they may eat.
- The four numbers that must not move — friends, rivals, rumor reach, interruptions — stay put.
- Determinism holds; stock is saved and restored bit-identically.
