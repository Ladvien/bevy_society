# Research: a memory nobody bounded, and a column that lied

## Two defects, one cause: nothing bounds the rumor side of a person's memory

The crate bounds every other per-person collection: `memory_capacity` (64) evicts the least salient
`Memories`, `max_ties` (150, Dunbar) trims `Ties`. `Knowledge` — the rumors a person knows — had no
cap.

Measured, 12 people round one well:

| tick | rumors in the book | max entries one person held |
|---|---|---|
| 300 | 24 | 24 |
| 1 000 | 88 | 78 |
| 2 000 | 173 | 155 |
| 4 000 | 386 | **344** |

At 1 000 people over a longer run the book grew without limit too, and the per-tick
`RumorReach` walk in `rules.rs` scans **every** rumor:

| tick | rumors in book | knowledge mean | ties mean |
|---|---|---|---|
| 1 000 | 1 523 | 64 (capped by this iteration) | 143.6 |
| 3 000 | 4 941 | 64 | 149.9 |
| 5 000 | **8 357** | 64 | 150.0 |

`Ties` settles at Dunbar's 150 and `Knowledge` now holds at 64; the book does not settle. Tick cost
follows: **15.0 ms** over a short run against **61.9 ms** after 3 000 settling ticks at 1 000 people —
4× with no more people.

**Why the book never shrinks.** `Rumor.alive` is cleared in exactly one place, `retire_about`, when
the rumor's **subject despawns**. Daley–Kendall stifling marks a *teller* as no longer spreading, but
nothing notices when *no one* is left who could tell it. Measured on 200 people after 3 000 ticks:
**992 rumors, 70 still tellable, 922 untellable (93%)**.

## The evidence harness was measuring the wrong rumor

`series.csv`'s `rumor_reach` column read `RumorBook::reach(RumorId(0), …)` — the **first** rumor the run
ever created. For the village that is a deed from tick 0; for the cluster it is a stranger's deed. The
rumor the scenario actually plants has a different id. So the column showed a step function at tick
~60 and nothing after, and three iterations recorded "rumor reach flat-lines" as a model defect.

With the planted id followed instead:

| scenario | old column | planted rumor |
|---|---|---|
| `rumor-crosses-a-cluster` | 1 change, flat 0.333 | **6 changes**: 0 → 0.083 → 0.167 → 0.25 → 0.333 over ~350 ticks |
| `village-evening` | flat 0.417 | **0.042 = 1/24**, and it never grows |

The cluster number is the same endpoint, but the shape is a real diffusion curve. The village number is
worse than reported and is a *separate* finding (below).

## Why the village's planted rumor never spreads, measured

The village plant picks `villagers[8]` as origin. `gossip` is gated on `ctx.tellable(actor)` — a villager
with nothing to say scores zero and chats instead. Traced over 3 600 ticks after the plant:

```
gossip exchanges: total 63, from the origin 2
planted reach: 0.042 (known_by = 1)
```

So the origin gossiped twice and its pick was stifled or ineligible both times. The scenario, not the
model, is at fault: it asks an arbitrary villager to be the town crier.

## Literature

| Claim | Source |
|---|---|
| Social memory is bounded; a person cannot track every deed in a large society | Dunbar 2004, DOI 10.1037/1089-2680.8.2.100 — already cited at `max_ties` |
| A rumor stops when its tellers stop telling | Daley & Kendall 1964, DOI 10.1038/2041118a0 — already the crate's gossip rule |
| Eviction by retained salience, not by age alone | Park et al. 2023, DOI 10.48550/arXiv.2304.03442 — already the basis of `telling_weight` |

No new paper is needed: the crate already cites all three, and this iteration applies the rules it
already states to the one collection that escaped them.

## What believable output looks like

A person's memory of rumors should plateau, like its ties and its memories do, and gossip cost should
be flat in the length of the run. Measured: `Knowledge` now holds at exactly `knowledge_capacity`.
