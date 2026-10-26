# Research: trade between two villagers

The last gap in the economy: a price exists, and nothing trades. This iteration gives a good a
counterparty.

## 1. The topology is pairwise, not a market

**Parunak et al.**, market topologies (corpus doc `10.1007_11678809_10`, converted). Two figures, and the
second is the one this crate wants:

> "In an **auction-based market** ... the marketplace is the environment in which buyers and sellers
> interact, using currency as markers ... Dynamics: integrate offers to compute prices that clear the
> market."

> "A less-studied alternative to auction systems is the pattern of **Edgeworth barter**, in which buyers and
> sellers interact directly with each other in **repeated pairwise transactions, without the benefit of an
> auctioneer** ... Agents: Traders. Environment: **Pairwise encounters**. **Dynamics: Support mixing of
> traders.** Emergent system behavior: **In repeated trades, balances supply and demand**."

Two things follow. A trade here is a `Person`-targeted activity, which the crate already has. And the
*emergent* claim is that repeated pairwise trade balances supply and demand — so the test is whether the
price of a good stops running to its clamp once people can trade it.

## 2. A bargain needs a credible threat point, and "walk away" is not one

**Hall, R. E. (2005)**, *Job Loss, Job Finding, and Unemployment*, NBER WP 11678, DOI `10.3386/w11678`
(in the corpus, converted), reporting **Hall & Milgrom**:

> "Hall and Milgrom (2005) point out that the **threat points considered in the standard labor-market
> bargaining model are not credible**. Once a qualified worker and an employer have met and found that they
> would enjoy a joint surplus, the threats to disclaim the match are hollow. The sequential bargaining
> framework of Binmore, Rubinstein and Wolinsky (1986) considers alternative, credible threats. **The
> parties alternate in making wage proposals to each other. The threat point of each is to prolong
> bargaining by declining an offer and making a counteroffer.**"

That is the design constraint, and it rules out the obvious implementation. A villager will not refuse a
trade that leaves them better off just because it leaves the other side more; the credible alternative to
agreeing is **another attempt later**. So the bargain must be settled by a **split of the surplus** that
depends on relative patience, not by a threat to walk. This crate has no patience parameter, so the split
is a design default — and it must be stated as such.

The paper also supplies the useful half: **the joint surplus is what makes the trade happen** ("found that
they would enjoy a joint surplus"). A trade is offered when the pair's valuations of the good are far
enough apart, and not otherwise.

## 3. Why anyone values a good differently

The literature here is the previous iteration's: **Henrich et al. (2005)** (DOI `10.1017/S0140525X05000142`),
on goods not being fungible. But the *valuation* has to come from somewhere, and the crate already has it:
a `Stock` is a level, so a person with none values a unit more than a person with a full store. That is the
textbook shape without a utility function — valuation falls as the store rises — and it needs **one
`Tuning` field**, not a curve.

## What the literature does not supply

- **No number** for the split of the surplus, for the patience that would set it, or for how much a full
  store is worth. All are **design defaults**, and each says so in code.
- No claim that a village of 24 should have a market place. Finding 1 says pairwise encounters, and the
  crate's `Perception` already decides who meets whom.

## What believable output looks like

- **Trades happen and both sides change.** A villager with surplus bread and no other good swaps.
- **Trade is voluntary and improving**: after a trade neither side is worse off by its own valuation, and
  the pair's total is higher. That is the Edgeworth property and it is checkable arithmetic.
- **A price stops running to its clamp** once the good can move between people, which is finding 1's
  "in repeated trades, balances supply and demand".
- **The four regression numbers do not move**: friends, rivals, rumor reach, interruptions.
- Determinism holds; no new draw.
