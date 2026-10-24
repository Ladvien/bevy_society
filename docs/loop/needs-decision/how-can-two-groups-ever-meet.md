# How can two groups ever meet?

## The state, measured

`rumor-crosses-a-cluster` has been `pending` since November. Its observable:

> With the two groups 40 m apart and **one person from each group brought together once**, a rumor planted in
> the first group reaches at least 40% of the second group.

Measured, the rumor reaches **0 of 6** in the second group at 64 s/day **and** at 240 s/day.

## What is *not* wrong

Each layer was checked separately, and all of them work:

| layer | measured |
|---|---|
| steering in isolation | velocity **−1.4 m/s**, walked 2.0 → 0.731 m |
| movement in the cluster | **359 948** person-ticks moved more than 5 cm |
| choosing a bridge | market chosen as target **36 489** person-ticks |
| arriving at it | **6 720** person-ticks arrived |
| both groups there at once | **434 ticks** with a group-0 *and* a group-1 person arrived |
| cross-group visibility | at one such tick a group-0 person had persons 7 and 11 in `near`, at **5.24 m** and **2.89 m** |
| interactions between groups | **0** |

So two people from opposite groups stand 2.89 m apart, each sees the other, `contact_radius` is 4 m, `chat`
carries no script gate, and **no exchange fires between them**.

## The cause

**An exchange fires on *arrival*, once.** Proven by the smallest possible case: two people, both already at the
same place, within `contact_radius` — **39 interactions** over 20 000 ticks. So the mechanism works.

The cluster's 434 overlap ticks are ticks where both groups *are* arrived. Arrival is a single instant per
activity, so a pair that arrived at tick 100 and tick 500 shares 400 ticks of standing together and shares no
arrival. Nothing fires.

And they arrive hours apart by construction: a 20 m walk to a shared place is **5.4 game-hours** at 64 s/day,
while the activity that brought them lasts 0.75. The walk and the visit are on different scales, which is
`travel-time-in-two-clocks.md` again.

So two groups can occupy the same place indefinitely and never exchange, **not** because the crate cannot see
them — it can, at 2.89 m — but because it resolves exchanges on arrival rather than on contact.

## Options

### 1. Make contact itself an exchange trigger

A system that, for each pair within `contact_radius` who have not exchanged recently, fires an `Interact`.
That is a new mechanism, and it would change the crate's cost profile (pairs, every tick).

- **For:** "two people standing together talk" is what a host expects, and it is the only thing that makes a
  shared place meaningful.
- **Against:** a new per-tick pair pass; needs a cooldown to avoid a firehose; a real model change.

### 2. Bridge with a `Person` target, not a place

Give one person an activity targeting the *other group's* people, reachable by widening `perception_radius`
locally or by shrinking the gap to 8 m.

- **For:** no new mechanism; uses the exchange as designed.
- **Against:** it is not a *cluster* scenario any more — the gap is the subject.

### 3. Restate the observable

Say what the crate does: a rumor crosses when someone chooses a person in the other group as a gossip target,
which needs them within perception.

- **For:** honest, cheap.
- **Against:** it retires a scenario that names a real question — "can gossip cross a gap" — without answering
  it.

## Recommendation

**Find out whether option 1's reading is right before choosing.** The decisive test is one pair, placed within
`contact_radius`, both already arrived at the same place, and a check of whether `InteractionResolved` ever
fires for them. If it does not, the crate's exchange is arrival-driven and the scenario needs option 1 or 2.

## What ships now

Nothing from this exploration. The scenario stays `pending` and its observable unchanged. The measurement chain
above is recorded so the next attempt starts from the containment rather than from the symptom — and because
six layers each checked out while the outcome was zero, which is worth having written down.
