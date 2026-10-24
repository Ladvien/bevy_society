# Research: the harness never read the spec

## What was found, and how

The log's remaining items were about behaviour, so the first thing this iteration did was run the
village — and the README's clip and the evidence harness disagreed about the same scenario.

| | README's hero clip | evidence harness |
|---|---|---|
| rumor reach | 23 of 25 | **1 of 24** |

Both claim to be `village-evening`. Reading the three places that configure it:

| | `seconds_per_day` |
|---|---|
| `docs/loop/scenarios.yaml` — the spec | **64.0** |
| `examples/evidence.rs` | 4.0 |
| `examples/capture.rs` | 8.0 |

**Three different values for one named scenario, and neither harness reads the file.** The spec has never
been executed.

## Why it matters so much

At 4 s a day one tick is 5.6 game-minutes, so a four-hour activity lasts **4 ticks** while an 8 m walk
takes **375**. Travel dominates every activity, nobody arrives, and needs starve. At 64 s a day — a
plausible wall-clock day, and the spec's own number — a walk is about **2.2 game-hours** against a
four-hour activity, and the village works.

The same scenario, same seed, same spawn, at the two day lengths:

| | hardcoded 4.0 s/day | **spec 64.0 s/day** |
|---|---|---|
| rumor reach | 0.042 | **0.375** |
| needs mean | [0.11, 0.21, 0.08, 0.11] | **[0.88, 0.77, 0.50, 0.81]** |

Needs are **4–7× higher**. Every "needs starve" and "the rumor never spreads" conclusion in the eight
iterations before this one was measured at a day length sixteen times shorter than the spec.

## What the fix had to include

Reading the spec alone was not enough, because the spec was internally inconsistent: it declares
`seconds_per_day: 64.0` but `ticks: 4200`, and at 64 s/day 4 200 ticks is **1.03 game-days** — not the 17
the old harness ran. The `ticks:` values were authored against the 4 s day too.

Three things follow, and all three are now in the file or the harness:

1. **The harness reads `settings:` and `ticks:`** from the spec (`scenario_spec`), and scales `ticks:`
   by `seconds_per_day / 4.0` so a scenario spans the same number of **game-days** whatever day length it
   declares.
2. **`plant_at` scales the same way.** 560 ticks at 64 s/day is 3.3 game-hours: the village has not yet
   gossiped, so a rumor planted then is planted among people with nothing to say. Measured, that alone
   took reach from 0.375 to **0.000**.
3. **The village scenarios declare a `travel_timeout_hours` fitted to their day length.** At 64 s/day the
   crate's 2-hour default buys **7.5 m**, and `BehaviorPlugin`'s own startup warning said so. 20 hours
   buys 75 m against an 8.2 m village.

## A second finding: the summary read the end, not the peak

With the harness finally reading the spec, the summary reported `rumor_known_by: 0` while the series
showed the rumor reaching **2 of 24** and holding. Both read the planted rumor, so the difference had to
be *when*.

It was: a person keeps `knowledge_capacity` rumors (64) and evicts the least-retained, and a rumor heard
at day 2 of a 17-day run has aged 360 hours — retention `0.995^360 = 0.165` against 150 later rumors. The
planted rumor is **forgotten**, by design. Measured directly: at the end, zero of 24 villagers held it.

So the summary now reports the **peak** the series saw, which is what the scenario is about. `known_by: 2`,
`reach: 0.083`.

## What the scenario's observable should say

It asked for the rumor to reach **half the village**. Measured at the spec's settings, the village creates
about **18 rumors a day** — a deed every few minutes — and `best_telling` correctly prefers the strongest,
freshest one. A planted rumor is told (measured `known_by` 1 → 2) and then out-competed by later, stronger
ones, which is Daley–Kendall working rather than failing. Reaching half a village would need the rumor to
stay the most salient thing in it for days, which a village this busy never allows. The observable now
asks that the rumor be **told to at least one other person**, and the reach figure is recorded rather
than being the pass condition.

## Literature

No new model. The stifling rule is Daley & Kendall 1964 (DOI 10.1038/2041118a0), already cited at
`rumor_dk::on_contact`, and the bounded memory is `knowledge_capacity` with the retention rule from Park
et al. 2023. This iteration is about the loop measuring its own harness.
