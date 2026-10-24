# How can two groups ever meet?

> **Investigated 2026-12-08. The mechanism works and the rate cannot be chosen without breaking the
> village.** The finding is below; nothing shipped.

## The state

`rumor-crosses-a-cluster` has been `pending` since November. Its observable:

> With the two groups 40 m apart and **one person from each group brought together once**, a rumor planted in
> the first group reaches at least 40% of the second group.

Measured: **0 of 6** in the second group, at 64 s/day and at 240.

## The cause, established

**An exchange fires on *arrival*, once.** Two people placed together at the same place give **39** interactions
over 20 000 ticks, so the mechanism is sound; but a pair arriving at tick 100 and tick 500 shares 400 ticks of
standing together and shares **no arrival**. They arrive hours apart by construction: a 20 m walk is **5.4
game-hours** against a 0.75-hour activity.

Six layers were checked separately and all work — steering at 1.4 m/s, 359 948 person-ticks of movement, a
bridge place chosen for 36 489, 6 720 ticks arrived, 434 ticks with both groups arrived, and one person seeing
two of the other group at **2.89 m** — while cross-group interactions are **0**.

## The fix that works, and why it cannot ship as built

A `talk_in_contact` system in the `Interact` set, which walks the `Perception.contact` neighbourhood that
`expose_contacts` already computes and queues a `chat` per pair, gated by `contact_talk_cooldown_hours`.

Proximity predicts who interacts with whom — Eckland 1968, quoted in Buss 1993
(DOI `10.1037/0033-295x.100.2.204`): *"the chances are about 50-50 that the 'one and only' lives within walking
distance."* Measured, it takes a pair of co-located people from **30** exchanges in 20 000 ticks to **89**, and
the cooldown works exactly as designed (first gap 1 920 ticks at a 3-hour cooldown, then held).

**But it wrecks the village:**

| cooldown | friends | rivals |
|---|---|---|
| none (shipped) | **22** | **33** |
| 3 h | **267** | **0** |
| 12 h | 182 | 3 |
| 48 h | 72 | 12 |

267 friend pairs in a village of 24 is **97% of all possible pairs** — a clique, not a society. Even 48 hours
triples the baseline. The reason is scale: the contact pass adds **13 742** requests over 17 game-days against a
baseline of roughly **7 500** accepted interactions. It nearly triples all social contact in the village.

So the two goals conflict at these numbers. Any rate high enough to bridge a 40 m gap is high enough to make
everyone friends, because in a village *everyone is always in contact*.

## Options

### 1. Contact talk, but not as an exchange

Queue the conversation as a **bark** or a mood nudge rather than a full `chat` exchange with its tie effects.
Two people who share a place are aware of each other; they do not become friends by standing.

- **For:** decouples "meeting" from "befriending", which is the actual conflation. A rumor could still pass.
- **Against:** a rumor passing *requires* a gossip exchange, so this does not bridge the cluster.

### 2. Gate contact talk on the tie

Only pairs with affinity above a threshold, or ties already acquainted, speak on contact.

- **For:** keeps the village's rate near baseline; new contacts stay rare.
- **Against:** the cluster's groups have no ties between them, so the gate closes exactly where the scenario
  needs it open.

### 3. Restate the observable

Accept that a meeting needs one person to *choose* the other as a target, which needs them within
`perception_radius`, and that a 40 m gap cannot be crossed by the decision model at this clock.

- **For:** honest, and it names the real constraint (`travel-time-in-two-clocks.md`).
- **Against:** retires the scenario without answering it.

## Recommendation

**Option 3 for the scenario, and option 1 as a separate feature.** The scenario asks for a crossing the
decision model cannot produce at its clock; restating it is honest. A "people who share a place notice each
other" system is worth having *for its own sake* — it is what makes a tavern feel occupied — but it must be a
bark or mood effect, not an exchange, or the village becomes a clique.

## What ships now

Nothing. The `talk_in_contact` change was written, measured at four cooldowns, and **reverted**; the crate is
back at friends 22 / rivals 33. The measurements above are the deliverable, so the next attempt does not
re-tune a rate that cannot be right.
