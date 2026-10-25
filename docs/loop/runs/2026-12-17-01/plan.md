# Plan: contact gossip, gated on news

## Why this over the runner-up

Item 3 was host ergonomics; item 4 the clock decision (a breaking change, still escalated). Item 2 — behavior
that reads as wrong when you watch it — is the silent tavern: a village where people share a bench for an
activity and say nothing. It is also the last unshipped half of `how-can-two-groups-ever-meet.md`, whose own
recommendation was "a 'people who share a place notice each other' system, worth having for its own sake".

## Design

A `contact_gossip` system in `SocietySystems::Relate`, reading the `Perception.contact` list `expose_contacts`
has just written, and queueing a `gossip` `Interact` along a directed tie when **the teller has a rumor the
listener does not know**:

```
mine.best_telling(now, |rumor| {
    book.get(rumor)? alive && subject != listener && !theirs.knows(rumor)
})
```

Paced by one new `Tuning` field, `contact_gossip_cooldown_days`.

Touched: `src/behavior.rs` (the system, a `SystemParam` to stay inside clippy's argument limit),
`src/core.rs` (the field), `tests/contact_gossip.rs` (four tests, new), `docs/loop/scenarios.yaml` untouched.

## Measured

| cooldown | friends | rivals | rumor reach |
|---|---|---|---|
| shipped baseline | 22 | 33 | 0.583 |
| 0.5 d | 23 | **45** | **0.375** |
| **2.0 d** | **25** | **28** | **0.875** |
| 6.0 d | 22 | 33 | 0.583 |
| gate disabled | 22 | 33 | 0.583 |

Mood: -0.0536 baseline, **-0.0523** shipped.

## Why 2 days

It is the only sampled setting where the system **adds** something: reach 50% above baseline while rivals fall
from 33 to 28. At half a day it burns the spreader pool faster than it spreads news and reach *drops* 35%.

## The trap this iteration nearly shipped

The first version omitted `theirs.knows(rumor)` as redundant, because `best_telling` already skips stifled
spreaders. It is not redundant: it is what stops a contact-tell spending a spreader on an informed neighbour.
Without it reach was **0.375** against a **0.583** baseline — a silent regression the village summary did not
show and only the reach panel did.

## Tests

Four, all verified to fail when the behaviour they pin is removed:
1. a spreader who shares a place passes the rumor;
2. a person with no news says nothing (the clique guard);
3. an informed neighbour never costs the teller its spread (the 0.375 trap);
4. the rumor does **not** cross a 40 m gap, so the scenario stays `pending`.
