# Research - kin (2026-10-31-05)

The paper note is `docs/research/kinship.md` (sources with DOIs, the formula, what is and is not modelled). This file is the pass's own record: why this item, what was read, what was not found.

## Why this item

No open issues (`gh issue list` empty; bevy_emerge #104-#106 closed), no parked tree for it (`wiggum/parked-*` are passes 20-40, all older than the work on `main`). Last tag `v0.9.0` was minutes old at the start (`git log -1 --format=%ct v0.9.0`), so no release this pass.

Ranked candidates (host impact):

1. **Kin.** The relationships note lists "kin, role or structural ties (family, employer)" as left out, and `Tuning::bereavement_familiarity`'s own doc admits the workaround: a family member only grieves because the author typed in a high familiarity. A host that wants a household has no way to say who is related to whom, a pack cannot score by it and a script cannot ask. **Picked.**
2. Satiation after an accepted approach (the previous pass's #3). Baseline run (village, seed 7, 6.6 game-days, `evidence/satiation-baseline/`; `evidence --scenario village-evening --seed 7 --days 6 --track 3`): 36.3 % of exchanges go to the initiator's previous partner and 47 % of accepted approaches are followed within an hour by another to the same person. The narrative shows it (Dena: four gossips from Lira in one hour on day 2; seven compliments to and from Alys on day 3). It loses on source: the corpus search (`distill_search`, `abstract_search`, four queries) returned nothing on within-day partner habituation, and the literature that exists on communication bursts (real pairs do talk in trains) argues that some of it is right. A damping with no source and an unclear target would be a tuned number.
3. A market day (the week makes a periodic gathering expressible; the `rumor-crosses-a-cluster` scenario is still `pending`). Needs a source on periodic markets and a travel clock that reaches it.

(1) beats (2) because the gap in (1) is a missing concept a host cannot work around, with three readable sources that agree on a direction; (2) is a rate with no source.

## What was read

Verified by tool this pass (DOI returned by `paper_get` or `paper_search`; "abstract" means the abstract text was returned):

| source | DOI | status |
|---|---|---|
| Hamilton 1964 | `10.1016/0022-5193(64)90038-4` | record (no abstract); the coefficient `r` and the rule are standard |
| Curry, Roberts & Dunbar 2013 | `10.1111/j.2044-8295.2012.02119.x` | abstract: kinship premium beyond emotional closeness |
| Burton-Chellew & Dunbar 2015 | `10.1093/beheco/aru165` | abstract: support rank rises with degree of relatedness |
| Cassar & Friedman 2026 (working paper) | `10.2139/ssrn.6460000` | abstract: transfers rise with relatedness |
| Roberts & Dunbar 2011 | `10.1016/j.evolhumbehav.2010.08.005` | record only; not downloadable (`paper_download`: no open copy) - **not used** |

`paper_download` found no copy of the first two Dunbar papers either, so the premium's direction rests on abstracts, as the Sommerfeld note in `needs-decision/chat-and-gossip-share-one-need.md` already does. The corpus (`distill_search`) holds none of them; the hits for "kin", "relatedness" and "contact decay" were unrelated papers.

## What believable output looks like

- **Who gives to whom.** In a village where some people are siblings, a bread-holder with a hungry sibling and an equally hungry stranger beside them gives to the sibling more often than chance, and the premium grows with `r` (a cousin less than a sibling). Sharing is still conditioned on a surplus: a relative with a full shelf gets nothing (`Input::Surplus` is unchanged).
- **Pooling.** Within a household, stores equalise more than across households, so bread variance within a household falls and the variance across households rises.
- **Nothing else moves.** Friend and rival counts, label changes, insults, sleep interruptions and the need floors stay inside the seed-to-seed spread, because the premium touches one kind of activity.
- **A relative is close.** A dead sibling is grieved by someone whose tie to them has no `Friend` label and low familiarity; a stranger with the same tie is not.
- **A society with no relatives is the previous release, byte for byte.**
