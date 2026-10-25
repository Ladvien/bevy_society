# Research: does the village ever make up?

## Why this

Reading `village-evening`'s summary rather than my task list: **329 of 6 925 endings interrupted (4.8%)** — down
from 22.8% a few iterations ago, so the interruption work has largely landed. But the tie network reads **18
friends against 42 rivals**, and the rival count climbs from 0 to 42 across the run while friends sit at 13-18.

A count of 42 says how many rivalries exist. It says nothing about whether any ever **end**. That is the question
a society has to answer, and the harness had no way to ask it.

## The instrument

`TieLabelChanged` fires on every label transition. Counting the ones that *enter* `Rival` and the ones that
*leave* it gives the two halves of reconciliation, and their difference is exactly the number of rival ties
standing:

```
became_rival - left_rival == 2 * rivals   (undirected)
```

Nothing in `examples/evidence.rs` had ever drained that message, so the metric did not exist. It does now.

## Measured, `village-evening`, 24 people, 17 game-days

| seed | became rival | left rival | ratio | final rivals | friends |
|---|---|---|---|---|---|
| 7 | 272 | **188** | 1.45 | 42 | 18 |
| 42 | 308 | **238** | 1.29 | 35 | 19 |
| 11 | 377 | **297** | 1.27 | 40 | 14 |

**The village does reconcile** — roughly a quarter of rivalries end — but it forms them about 1.3-1.45x faster
than it dissolves them, and settles at 13-15% of all pairs being rivals against 6-7% being friends. The rival
count decelerates (second-half growth 0.68x the first half), so it is saturating rather than running away.

That ratio is the number to watch. Before this iteration, no reader of `summary.json` could have distinguished
"the village holds grudges" from "the village never lets them go".

## The lever, measured but not pulled

`affinity_relax_per_day` (0.01 by default) is the reconciliation rate. Raising it:

| relax | rivals | friends | ratio |
|---|---|---|---|
| 0.01 (shipped) | 42 | 18 | 1.45 |
| 0.015 | 28 | 7 | 1.22 |
| 0.02 | 25 | 7 | 1.20 |
| 0.03 | **10** | 5 | **1.08** |

Monotone in the right direction on rivalry and **wrong** on friendship, because relaxation pulls *all* affinity
toward zero — it cannot tell a grudge from a fondness. At 0.03 the village has ten rivalries and five
friendships: nearly friendless.

So the setting is a trade the host should make, not one to pick silently. Recorded in
`needs-decision/what-does-the-village-owe-its-grudges.md`.

## Sources

No paper. `Bondü & Richter 2016` (DOI 10.3389/fpsyg.2016.00795) already supplies the grudge mechanism the `snub`
activity reads; this iteration measures whether the *other* side of that mechanism ever fires, which is a fact
about the crate rather than a claim about people.
