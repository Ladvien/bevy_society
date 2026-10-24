# Research: two groups can see each other and cannot meet

## The scenario

`rumor-crosses-a-cluster` has been `pending` since November. Its observable:

> With the two groups 40 m apart and **one person from each group brought together once**, a rumor planted in
> the first group reaches at least 40% of the second group.

Two things in that sentence are not what the harness does: nothing brings anyone together, and the harness
divides by all twelve people while the claim is about the second group of six.

## Measured, layer by layer

Six layers, each checked separately. **All of them work.**

| layer | measured |
|---|---|
| steering in isolation | velocity **−1.4 m/s**, walked 2.0 → 0.731 m |
| movement in the cluster | **359 948** person-ticks moved more than 5 cm |
| a bridge place chosen as target | **36 489** person-ticks |
| arriving at it | **6 720** person-ticks arrived |
| both groups arrived simultaneously | **434 ticks** |
| cross-group visibility | one person had two of the other group in `near`, at **5.24 m** and **2.89 m** |
| **interactions between groups** | **0** |
| the rumor reaching group 1 | **0 of 6**, at 64 s/day and at 240 |

## The cause

**An exchange fires on *arrival*, once.** Proven by the smallest possible case: two people, both already at the
same place, within `contact_radius` — **39 interactions** over 20 000 ticks. The mechanism is sound.

But a pair that arrives at tick 100 and tick 500 shares 400 ticks of standing together and shares **no
arrival**. Nothing fires at tick 500, because nobody arrived then.

And they arrive hours apart by construction. A 20 m walk to any shared place is **5.4 game-hours** at 64 s/day,
while the activity that brought them lasts **0.75**. The walk and the visit are on different scales — which is
`travel-time-in-two-clocks.md` again, this time with an observable outcome.

So two groups can occupy the same place indefinitely and never exchange, **not** because the crate cannot see
them. It sees them at 2.89 m.

## Why the bridge idea failed first

A crossroads carrying `gathering` changed the reach from 0.167 to **0.167** — because `Place` targets pick the
*nearest* matching place (the 2026-11-23 fix), and each group's own hearth is 2 m away against the crossroads'
20. A bridge only bridges if it is the nearest match for someone, so it has to carry a tag the hearths lack.

That version works at the "chosen" layer — **36 489 person-ticks** — and still reaches **0 of 6**, because of
the arrival timing above. Two failed fixes, each measured, each narrowing the cause.

## Sources

No new model. The exchange is `resolve_interactions` (plan D7); the timing scale is
`docs/loop/needs-decision/travel-time-in-two-clocks.md`; the nearest-place rule is the 2026-11-23 iteration.

## What believable output looks like

Two people who are standing together talk to each other. The crate does that when they *arrive* together, and
not otherwise — which is a defensible model of a chance meeting and a poor model of a shared place.
