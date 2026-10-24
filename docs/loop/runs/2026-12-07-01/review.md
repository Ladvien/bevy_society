# Review

## Pass 1 — Behavior: is the claim true, and is the cause right?

**GREAT** as an investigation, and the honest verdict is that the scenario **fails**.

| layer | measured | works? |
|---|---|---|
| steering | −1.4 m/s, walked 2.0 → 0.731 m | yes |
| movement | 359 948 person-ticks | yes |
| bridge chosen | 36 489 person-ticks | yes |
| arrived | 6 720 person-ticks | yes |
| both groups arrived | 434 ticks | yes |
| cross-group visibility | 2.89 m | yes |
| **cross-group interactions** | **0** | — |
| **second group reached** | **0 of 6** | — |

Six layers working and a zero outcome is not a single bug. The smallest case — two people placed together —
gives **39 interactions**, so the exchange works; the cluster's pairs simply never share an *arrival*.

## Pass 2 — Zoom: read the numbers closely

**GREAT**, and the decisive numbers are the ones that *eliminated* hypotheses. 434 ticks with both groups
arrived looks like they should talk; measuring that an exchange fires once, on arrival, explains why they do
not. 36 489 ticks of choosing a bridge and 6 720 of arriving eliminate "they never go there". 2.89 m eliminates
"they cannot see each other".

The clock arithmetic closes it: a 20 m walk is **5.4 game-hours** against a **0.75-hour** activity, so arrivals
are hours apart by construction. This is `travel-time-in-two-clocks.md` producing an observable outcome, which
that decision file has never had before.

## Pass 3 — Fresh eyes: what a host would conclude

**GREAT**, and it is uncomfortable. A host reading the scenario name — *rumor crosses a cluster* — would
expect gossip to bridge a gap. What the crate actually models is **meetings happen when someone chooses a
person as a target and walks to them**, not when two people happen to be in the same place. That is defensible
for a chance encounter and wrong for a shared tavern, and the decision file now states which is which.

The narrative:

> Two groups 40 m apart never exchange a rumor, even when they walk to the same market and stand three metres
> apart for hundreds of ticks. The exchange fires when someone *arrives* at an activity, once — so people who
> arrive hours apart share the place and not the moment. Everything underneath works: they walk, they choose
> the place, they arrive, they see each other. The gap they cannot close is a timing one, not a spatial one.

## Verdict

Three GREAT for the investigation, and the scenario **fails** — asserted, measured at two clocks, with the
cause established. Nothing was marked `measured` on a number nobody checked.
