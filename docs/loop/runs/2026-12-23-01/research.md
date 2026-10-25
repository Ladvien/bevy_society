# Research: the village is provisioned exactly at capacity

## Why this

Last iteration logged a blocker: 150 sleeps interrupted, seven levers dead, and the instruction that the next
attempt should **instrument what the solver is handed**. This iteration did that, found the mechanism, and shipped
the part that is honestly shippable — a warning — rather than a lever that trades one model for another.

## The instrument

At every timed-out `sleep`, count how many bodies already stand inside `interaction_radius` of the goal:

| bodies already in the disc | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|
| timed-out sleeps | 1 | 1 | 8 | **40** | **55** | 36 | 4 | 2 | 3 |

**Median 6, maximum 10.** The villages' homes declare `capacity: 6`. The disc is full at exactly its declared
capacity, and the failures are the queue behind it.

## Why every lever failed

A `Place` disc of radius 1.5 m seats about `(1.5 / 0.7)² ≈ 4.6` bodies at ORCA's one-body-diameter spacing. Six
residents do not fit — so the six who get in are already over-packed, and everyone else waits.

Every lever tried last iteration either could not change that (steering `stop_radius` was **byte-identical**),
made it worse (scattering goals, honouring capacity alone), or paid for it elsewhere (`interaction_radius` doubles
rivalries because contact scales with it).

This iteration added three more, and all three show the same trade:

| lever | interrupted | rivals |
|---|---|---|
| baseline | 400 | 38 |
| crowd allowance 0.35 m/person | 349 | **59** |
| crowd allowance 0.70 m/person | 290 | **51** |
| physical gate (`r + d·√n`) | 316 | **61** |

Interruptions fall **21-28%**; rivalries rise **13-23%**, every time. The mechanism is not contact radius — that is
4.0 m and the arrival disc is ~2 m — but **time**: arriving earlier means the activity starts earlier and runs its
full duration, so more of the day is spent standing still inside `contact_radius` instead of walking. More contact
time, more tie drift.

That is a real behaviour change, not a bug. It is also exactly the coupling the decision file named: arrival and
contact are entangled because both are made of proximity.

## What shipped

Not a lever. `warn_unreachable_places` — already a `Startup` system measuring the layout — now also counts
residents per home and warns when a home cannot seat them:

```
'well' is home to 8 people, but an arrival disc of radius 1.5 seats about 4. They all go to bed at the same
hour, so the queue behind the disc times out and their sleeps are interrupted. Give them more room: raise
`interaction_radius` to about 2.0 or spread the residents over more homes.
```

The arithmetic is stated as what it is — hex packing is a **lower bound** on the crowding, because ORCA does not
pack around a point, it queues at the centre. The warning says "about", and does not claim to simulate.

## Why a warning and not a fix

Because every fix costs the tie network, and **the choice between them is the host's**: raise the arrival radius
and accept more rivalry drift, spread the residents over more homes, or accept the interruptions. The crate's job
is to make that choice visible with numbers, not to make it silently.

## Sources

No paper. This is packing arithmetic and the crate's own measurements.
