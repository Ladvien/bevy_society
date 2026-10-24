# Research: an observable nobody had ever asserted

## How it surfaced

Four iterations changed the model and the pack substantially. `village-evening`'s observable says a rumor
planted at tick 300 is "**told to at least one other person**", and its status is `measured` — so I ran it at
ten seeds and read the numbers.

| seed | 3 | 4 | 5 | 6 | 1 | 7 | 11 | 42 | 99 | 8 |
|---|---|---|---|---|---|---|---|---|---|---|
| `rumor_known_by` | 2 | 2 | 2 | 2 | 3 | 5 | 3 | 3 | **1** | **1** |

Two of ten report the rumor reaching nobody but its origin.

## The finding is not the flake

`grep` for the observable across `tests/` returns nothing. `village-evening` is marked `measured` and
**nothing asserts any part of it** — the harness prints the numbers and every reading has been by eye. The
flake could have sat in that file indefinitely.

So the first thing this iteration does is write the assertion, and the second is to find out which of the two
disagreements produced it.

## Two disagreements, both real

### 1. The harness does not plant where the scenario says

`scenarios.yaml`: *"a rumor planted at tick 300"*.
`examples/evidence.rs::plant_at`: `560 × (seconds_per_day / 4)` = **8 960** at this scenario's 64 s/day.

The scaling exists for a good reason and is documented — 560 raw ticks is 3.3 game-hours at 64 s/day, before
the village has gossiped, and reach measured 0.000 there. But the *number in the file* and the *number in the
run* differ by a factor of thirty, and only one of them is the one the observable describes.

Measured, the condition holds at **7 of 7 seeds planted at 300** (reaching 2–8 villagers) and at **8 of 10 at
8 960**. A plant 4.3 game-days in lands past the village's busiest period, where a single rumor competes with
more of the ~39 a day it creates. The flake is the late tick, not the mechanism.

### 2. The harness counts the player as a holder

`known_by` counts `world.villagers`; my test counts `PersonIndex.sorted`, which includes the player. At the two
seeds that report `known_by: 1`, the test found **2** — one villager and the player.

That matters because **the player never gossips**: it is a `Person` without `Autonomous`, so a rumor reaching
only the player has died. An observable that counted it would be true while the mechanism had failed, which is
precisely the reading this file exists to prevent. The test counts villagers only.

## What the mechanism actually does

At the scenario's own tick and counting villagers, **7 of 7 seeds** spread the rumor, reaching 2–8 people. The
gossip model works; the documentation was describing a run the harness does not perform.

## Sources

No new model. The gossip rules are `docs/research/rumor-and-reputation.md` — Daley & Kendall stifling,
`best_telling` by `telling_weight = |valence| · belief · recency`. Nothing here changes them.
