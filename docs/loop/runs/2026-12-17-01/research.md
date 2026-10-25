# Research: people speak when they have news

## Why this

`rumor-crosses-a-cluster` has been `pending` since November with a measured cause: **an exchange fires on
arrival, once**, so two people who arrive hours apart share standing time and no arrival. The decision file that
recorded it (`how-can-two-groups-ever-meet.md`) tested one repair — queue a `chat` for every pair in contact —
and rejected it: 13 742 requests over 17 game-days against a 7 500 baseline, and a village of 24 became **267
friend pairs**, 97% of every possible pair.

The fault was diagnosed as *rate*. It is actually the *gate*: a `chat` is offered whether or not either person
has anything to say. That is the thing this iteration changed.

## The source

**Civil inattention** — Goffman 1963, as indexed and tested in Cañigueral & Hamilton 2019,
DOI `10.3389/fpsyg.2019.00560`:

> the amount of gaze directed to strangers "should be enough to acknowledge their presence but not so much as to
> indicate that they are of special interest."

Proximity is the channel because that is what proximity is for — Eckland 1968, quoted in Buss 1993,
DOI `10.1037/0033-295x.100.2.204`: *"the chances are about 50-50 that the 'one and only' lives within walking
distance."*

And the self-limiting rule is Daley & Kendall 1964, DOI `10.1038/2041118a0`: a teller that repeats itself
**stifles**. The crate already implements it, in `Knowledge::best_telling`'s `spreader` flag.

## The design

Gate the contact pass on **news**, not on company:

```
telling = mine.best_telling(now, |rumor| {
    the entry is alive
    the subject is not this listener
    the listener does not already know it     // <- load-bearing, see below
})
```

`best_telling` also ranks by `|valence| · belief · 0.995^(hours since heard)` (Park et al. 2023,
DOI `10.48550/arXiv.2304.03442`), so a stale rumor loses to a fresh one. A pair with no news between them is
offered nothing at all, which is the property contact-`chat` lacked.

## Pacing is not tuning, it is the spreader pool

Every telling to a listener who already knows **stifles the teller**. So how often the contact pass runs decides
how fast the village burns its spreaders. Measured on `village-evening`, same seed, over 16.4 game-days:

| cooldown | friends | rivals | rumor reach |
|---|---|---|---|
| 0.5 d | 23 | **45** | **0.375** — 35% *below* baseline |
| 2.0 d | 25 | 28 | **0.875** |
| 6.0 d | 22 | 33 | 0.583 — identical to the system doing nothing |
| gate disabled | 22 | 33 | 0.583 |

The monotonicity is the finding: too fast and reach *falls*; too slow and the system is pure cost. Two days is
the only sampled setting where contact gossip **adds** something — reach 50% above the shipped number while
rivals fall from 33 to 28.

The explicit `theirs.knows(rumor)` check is what makes this possible. Without it, a contact-tell to an informed
neighbour spends a spreader for nothing, and the 0.375 above is the result.

## What it does not do

It does **not** cross the scenario's 40 m gap, and the test says so: the gate opens on news, not on distance, so
two groups that never share a place never exchange. `rumor-crosses-a-cluster` stays `pending`, now for the
honest reason that its observable asks for something the decision model cannot produce at this clock — which is
option 3 of its own decision file.
