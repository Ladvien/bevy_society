# Perf

Release build, `--ticks 200`, two runs each. The change alters *how often* people decide, so the tick
number alone is misleading; per-decision cost is the honest metric.

| Persons | before (ms/tick) | after (ms/tick) | delta |
|---|---|---|---|
| 100 | 0.549 | 0.546 | −0.5% |
| 1 000 | 12.46 | **14.95** | +20% |
| 5 000 | **418.3** | **277.0** | **−34%** |

## Why the 1 000 figure rose and the 5 000 figure fell

Decisions per tick at 1 000 people, counted directly over 200 ticks:

| | decisions / tick |
|---|---|
| before | 50.4 |
| after | **68.6** (+36%) |

Before the fix, 80% of decisions went to `snub` and `gossip`, both of which last 0.1–0.3 game-hours, so
people re-decided slowly *because they were mostly idle*. After the fix `sleep` (3 869), `gossip`
(3 548), `relax` (1 783) and `work` (1 635) all run: people do things, and doing things means deciding
again sooner.

| | µs per decision |
|---|---|
| before | 247.1 |
| after | **218.7** (−11.5%) |

The scoring itself got **faster**, because the per-target `PersonId` lookup no longer scans
`PersonIndex.sorted` linearly for each target; `near` and its ids now come from one pass over
`perception.near`.

At 5 000 the net is a 34% *improvement*: the extra decisions cost less than the scanning and scripting
the old candidate set (one per neighbour) made the decision system do.

No scenario is slower than the change is faster: the 1 000-person cost is more work being done, not
more expensive work, and the 5 000-person cost — the one a host actually worries about — fell by a
third.
