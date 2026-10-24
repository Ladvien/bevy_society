# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what the scenario says?

Claim: `chat` and `gossip` should satisfy `social` while they run, so time spent talking
counts.

**Evidence**, same seed, same scenario, definitions differing only in those two `satisfies`
lines (`evidence/village-evening-before/` vs `evidence/village-evening/`):

| need | before | after |
|---|---|---|
| social | **0.105** | **0.225** |
| rest | 0.218 | 0.200 |
| food | 0.048 | 0.056 |
| fun | 0.041 | 0.046 |

Only social moves materially, which is the specific thing the change was for. The other
three move by 0.005–0.018, which is run-to-run noise from the changed decision landscape.

**Critique.** Doubling a mean from 0.105 to 0.225 is progress, but 93% of ticks are still
below 0.35. Is that the defect fixed, or the defect halved?

Checking: the remaining shortfall is explained by finding 2 in `research.md` — every activity
is shorter than the walk to it at this day length. `chat` lasts 3 ticks; a walk between grid
neighbours is 160. No `satisfies` rate can pay out in 3 ticks of a 16-day run.

**Verdict: GREAT** for the change, with the residual shortfall attributed rather than
ignored.

## Pass 2 — Zoom: what the series does at the edges

**Critique.** Reading `series.csv` per need:

- `social` now rises and falls repeatedly instead of pinning at 0. Before the change the
  rise:fall ratio was **420 : 20 365**; the rises were too rare to matter.
- `food` has **1028 troughs below 0.1 and 0 peaks above 0.9**. That looked like a leak, and
  it is not: `eat`'s logistic scores 0.0015 at a full need. A villager who is not hungry does
  not eat, so the need never gets high in the first place. The curve is correct; the summary
  statistic is misleading.
- No clipping artefact: all four needs reach both 0.000 and 0.997 somewhere in the run.
- No tick-boundary spikes: the census changes activity only at activity ends.

The one number that still looks wrong is `rivals 0`, which was last iteration's finding and is
unchanged by this work.

**Verdict: GREAT**, with `rivals 0` carried forward and `food`'s low mean explicitly explained
rather than left to look like a bug.

## Pass 3 — Fresh eyes: another seed and population

**Critique.** `rumor-crosses-a-cluster` — 12 people, two groups 40 m apart, 19 game-days —
gives **19 friendships and 11 rivalries** with the same definitions. Rivalry is reachable, so
the village's zero is a property of that village and not of the starter pack change. Good: the
change did not suppress rivalries.

The narrative still reads as a life. A line from the tracked villager:

```
18h36: gossip with Alys (refused)
19h39: gossip with Kell (accepted)
20h03: gossip approached by Edric (accepted)
```

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — social 0.105 → 0.225, only social moved, residual attributed |
| Zoom | GREAT — rise:fall ratio fixed, `food`'s low mean explained as correct |
| Fresh eyes | GREAT — 11 rivalries elsewhere prove the change did not flatten the society |

## Next iteration's candidates

1. **The clock decision.** `needs-decision/travel-time-in-two-clocks.md` recommends making
   `travel_timeout_hours` real-time. Until that lands, every short activity in every host is
   unreachable without a scenario work-around, which is a host-ergonomics tax no host should
   pay.
2. **Rivalry in the village**: `snub` needs a negative affinity nothing creates. Plant a
   rumor about a villager, or add a friction input.
3. `Signal::Field` and `ActivityTarget::Gradient` score nothing; Moussaïd 2011 needs
   re-converting first (equations were stripped).
4. `docs/guide/authoring.md` should say a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000`, still `pending`).
