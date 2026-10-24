# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what the scenario says?

The claim is not a new model but two corrections, so each is checked against a number.

**Ties.** `village-evening` at 900 ticks: 0 friends, 0 rivals. At 4200 ticks: 125 friends.
The mechanism is verified directly — a tie inspected at the end of the short run reads
`aff=0.067 peak=0.818 fam=0.818 label=Acquaintance`, so familiarity is high and affinity is
low, which is precisely what `exposure_per_day 2.0` against `mere_exposure_per_day 0.05`
predicts.

**Walking.** Before: 37 interrupted endings against 11 completed, needs pinned at 0.000.
After the grid and timeout change: 0 interrupted against 8 completed in the same run
length.

**Critique.** Both checks are end-state numbers. Neither shows the *curve*, and a change
that only shifts a threshold could pass both while the behaviour stayed ugly.

**Verdict: NOT YET.** Fix: look at the per-tick series, not the summary.

## Pass 1, second attempt

Reading `series.csv` at 100-tick intervals for `village-evening` after the fix:

```
t=   0  (empty)                     needs 0,0,0,0        (sized on tick 1)
t= 700  chat=3 compliment=1 eat=5 gossip=3 sleep=10 work=2
t=1400  chat=2 compliment=2 eat=2 gossip=6 sleep=12
t=2100  chat=7 compliment=1 eat=5 gossip=4 relax=1 sleep=3 work=3
t=2800  chat=3 compliment=5 eat=4 gossip=1 sleep=5 work=6
t=3500  chat=4 compliment=4 eat=2 gossip=3 relax=2 sleep=8 work=1
```

The mix changes across the run; no activity dominates; sleep clusters at night. Needs span
0.000 to 0.998, so they both deplete and recover.

**Critique.** The distribution is plausible but the *mean* need level is low (0.04–0.20).
That is a real observation about the starter pack's balance — `rest` decays 0.7/day while
sleep restores 0.15/hour for at most eight hours — and it is recorded below rather than
tuned away in this iteration.

**Verdict: GREAT** for the two corrections.

## Pass 2 — Zoom: what the series does at the edges

**Critique.** Reading the tick boundaries:

- No oscillation: needs fall monotonically while not served and rise while served, with no
  tick-boundary spikes.
- No saturation artefact: values reach 0.998 and 0.000 without sticking, and the census
  shows activity changing after reaching them.
- `rumor_reach` is a step function, as it must be (integer knowers over persons), and it
  rises monotonically from 0 to 0.800.
- Friendship count rises steadily; rivalry stays 0 for the whole run.

The one number that looks wrong is `friends 256` against 24 people: that is *directed*
ties, halved, so roughly 10 friends each, which is plausible for a Dunbar layer of 15.

**Verdict: GREAT**, with `rivals 0` named as a gap rather than an artefact.

## Pass 3 — Fresh eyes: another seed and population

**Critique.** `rumor-crosses-a-cluster` is a different population (12), a different layout
(two groups 40 m apart) and a longer run (19 game-days). It produces 19 friendships and
**11 rivalries** — so rivalry is reachable in this crate, and its absence from the village
is a property of that village (nobody snubs anyone when everyone is being complimented), not
of the model.

The narrative for Bram, read cold, is coherent:

```
18h11: compliment with Alys (accepted)
18h36: gossip with Alys (refused)
19h39: gossip with Kell (accepted)
20h03: gossip approached by Edric (accepted)
```

Each line is an exchange with a named person, in time order, at the durations the
definitions declare (0.2 h = 12 min). It reads as a person's evening.

**Verdict: GREAT** for the crate, NOT YET for the village scenario's rivalry.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT (second attempt; the first demanded the series, which is the point of the harness) |
| Zoom | GREAT, with `rivals 0` and a low mean need level recorded |
| Fresh eyes | GREAT for the crate; NOT YET for the village's rivals |

## The capture, measured through the same change

| | before | after |
|---|---|---|
| `rumor_reach` | 0.040 | **0.800** |
| `friends` | 0 | **256** |
| `rivals` | 0 | 0 |
| `barks` | 1 | **3076** |

Three of four thresholds met. The fourth is understood and named.

## Next iteration's candidates

1. **Rivalry**: `snub` needs a negative affinity that this village never creates. Either a
   scenario that plants one (`PlantRumor` about a villager, then let gossip do it) or a
   `Friction`-style input the definitions can score.
2. **Mean need level**: `rest` decays 0.7/day and sleep restores at most 1.2 per night, so
   the village lives near 0.15. Either the starter pack's `satisfies` rates or its
   `decay_per_day` are mistuned for a 24-hour day.
3. `Signal::Field` and `ActivityTarget::Gradient` still score nothing: the Moussaïd 2011
   paper is the right model but its equations were stripped in conversion.
4. `docs/guide/authoring.md` should say a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000`, still `pending`).
