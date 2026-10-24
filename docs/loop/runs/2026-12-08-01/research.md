# Research: the fix works and the rate cannot be chosen

## What was being fixed

`rumor-crosses-a-cluster`'s observable, `pending` since November: two groups 40 m apart, one person from each
*"brought together once"*, a rumor reaches 40% of the second group. Measured **0 of 6**.

Last iteration established the cause: **an exchange fires on *arrival*, once.** Two people placed together
give 39 interactions; a pair arriving at tick 100 and 500 shares 400 ticks of standing together and shares no
arrival.

## The mechanism

A `talk_in_contact` system in the `Interact` set, walking the `Perception.contact` neighbourhood that
`expose_contacts` already computes, queueing one `chat` per pair, gated by `contact_talk_cooldown_hours`.

**Proximity predicts who interacts with whom.** Eckland 1968, quoted in Buss 1993
(DOI `10.1037/0033-295x.100.2.204`):

> "the chances are about 50-50 that the 'one and only' lives within walking distance."

The crate already had every piece: `Perception.contact` is `contact_radius`, `Ties` already carries
`last_contact_day`, and the `Interact` message is the same one a host writes. The only new thing is a
per-pair conversation stamp and the request.

## Measured

| | before | after |
|---|---|---|
| exchanges between two co-located people, 20 000 ticks | **30** | **89** |
| the cooldown | — | first gap **1 920 ticks** at 3 h, then held |

The cooldown works exactly as designed. Verified tick by tick: `last_spoke_day` holds at 0.6252 across
ticks 63–1920 and updates at 1921.

## And then it broke the village

| cooldown | friends | rivals |
|---|---|---|
| none (shipped) | **22** | **33** |
| 3 h | **267** | **0** |
| 12 h | 182 | 3 |
| 48 h | 72 | 12 |

**267 friend pairs in a village of 24 is 97% of all possible pairs** — a clique, not a society. The contact
pass adds **13 742** requests over 17 game-days against a baseline of roughly **7 500** accepted interactions:
it nearly triples all social contact.

## Why the rate cannot be tuned to work

The two goals are in conflict at these numbers. Any rate high enough to bridge a 40 m gap is high enough to
make everyone friends, because **in a village everyone is always in contact**. An isolated pair 40 m away needs
contact to be rare; a village needs it to be rarer still. One number cannot be both.

That is the finding: the mechanism conflates *meeting* with *befriending*. Two people who share a tavern are
aware of each other; they do not become friends by standing. The crate has one verb for both.

## Decision

Reverted. The measurements are the deliverable; the decision file records three options, recommending that a
"people who share a place notice each other" effect be a **bark or mood nudge rather than an exchange**, and
that the scenario's observable be restated to what the decision model can produce at its clock.
