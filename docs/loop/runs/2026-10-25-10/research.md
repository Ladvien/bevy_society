# Research: a full place should cost, not vanish

## Why this

Last iteration's top remaining item, and the one it measured but could not decide: a `Place(tag)` target whose
`used >= capacity` is **dropped before scoring**, so a full workshop makes `work` an option that does not exist. In
a 30-seat village with 48 people, `work` fell from 10.3% to 3.6% of a life and rival density rose from 7.8% to
20.1% of pairs — because the same hostile events landed on a contact graph made dense by unemployment. Scaling the
seats instead took density to **0.00%**, which is the opposite failure.

So both extremes are broken, and the target is stated in advance: **rival density roughly flat at 7-8% of pairs from
24 to 48 people, with `work` holding near 10% of a life.**

## The literature

**Pentheny, G., "Advanced Techniques for Robust, Efficient Crowds", in *Game AI Pro 2* (corpus doc
`gameaipro2-ch17-advanced-techniques-robust-efficient-crowds`).** The chapter's subject is exactly this failure:

> "crowds in games are usually driven by static pathfinding combined with localized steering and collision
> avoidance. This technique ... works well for small or moderate crowd sizes ... **As crowd density increases and
> the agents' goals converge, this approach falls apart. In situations like this, agents fight the rest of the
> crowd to follow their ideal path as closely as possible, seemingly ignoring all other possible routes.**"

and the fix is a **price**, not a prohibition:

> "**Congestion maps** provide a simple means of modeling aggregate behavior ... Agents controlled by this
> technique appear to be more aware of their surroundings, **reconsidering their current path choice if a
> less-congested alternative exists.**"

with the cost itself proportional to the crowd:

```text
congestionPenalty(ideal, aggregate, density):
    cost = dot(ideal, aggregate) / |ideal|²
    if cost >= 1: return 0
    return (1 - cost) * density
```

The structure to take: **an occupied place is more expensive, not impossible.** A person with an alternative goes
elsewhere; a person with none still goes. That is what the crate cannot express today.

**The corroborating crowd-simulation source** (corpus `10.1145_1141911.1142008`, Treuille, Cooper & Popović 2006)
states the same principle from the simulation side: "unlike real people, locally controlled agents do not perform
any global or mid-range planning to avoid non-local congestion", and their fix is a dynamic potential that makes
congested routes costly. It supplies the mechanism's *shape* (a density-dependent cost) and no number.

## What the crate does today

```rust
ActivityTarget::Place(tag) => places
    .iter()
    .filter(|place| place.used < place.capacity && ...)   // a full place has NO candidate
```

`used` is recomputed each tick from who is currently targeting the place. `Home` and `Itself` bypass the check
entirely, and `capacity: 0` is used in `tests/bound_places.rs` to mean "never" — a genuine prohibition that must
survive.

## What a believable output looks like

- **A full place still takes people.** With no alternative, `work` should still be chosen — less often than when
  there is room, never zero.
- **A fuller place is less attractive than an emptier one**, monotonically, and by a *price*: the option is
  ranked lower, not deleted.
- **`capacity: 0` stays a prohibition.** A place that seats nobody is not "expensive", it is closed.
- **An uncrowded village is unchanged.** At 24 people in a 30-seat village no place is full, so every score must
  be bit-identical. That is the regression test that matters.
- **Density holds as the population grows** — the observable, and the thing this iteration is judged on.

## Parameters

- One new `Tuning` field, `crowding_penalty`: the fraction of an option's score that survives when its place is
  full. A **design default**, chosen by measurement; Game AI Pro gives the shape (cost ∝ density) and no number.
- No new draw, no new state, no save-format change.
