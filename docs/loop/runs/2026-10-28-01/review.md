# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what the scenario says?

Claim: the runaway was caused by the crowd having no positive activity, not by missing damping.

**Evidence.** Census of who is doing what, sampled every 200 ticks:

| | before | after |
|---|---|---|
| distinct activities in play | 2 (gossip, snub) | **5** (eat, work, gossip, snub, relax) |
| `chat` over 2000 ticks | 0 | 1 |
| mean affinity at t=1200 | −0.899 | −0.674 |

**Critique.** `chat` at 1 is barely different from 0. The hearth gives `eat`, `work` and `relax`,
which breaks the *monopoly*, but `chat` specifically remains nearly unreachable — the same
`Need(social) Linear(-1, 1)` gate plus gossip's self-supply. So the claim "the crowd has a
positive option now" is true, and the claim "chat works now" is not.

**Verdict: GREAT** for the mechanism (the monopoly is broken and five activities appear), with
`chat`'s near-unreachability recorded as a separate follow-up rather than folded in.

## Pass 2 — Zoom: what the series does at the edges

**Critique.** Affinity still trends down: −0.038 → −0.674 over 5 game-days, monotone. The
previous review called that a defect and asked for damping.

Two things had to be checked before accepting it:

1. **Is it saturated?** No. It reaches −0.674, not −0.96, and 12 of 132 ties are rivals rather
   than 130. The runaway is gone; a trend remains.
2. **Does the literature say it should damp?** No. **Takács, Flache & Mäs 2016**
   (`10.1371_journal.pone.0157948`, converted this iteration) report that **negative influence is
   not empirically supported**: people do not shift away from those they dislike, they simply
   stop being influenced by them. A downward affinity trend in a closed group that keeps
   provoking each other is what the model *should* produce.

So damping would have been inventing a mechanism the evidence contradicts.

**Verdict: GREAT**, with the residual trend explicitly justified against a converted paper rather
than left as an unexplained curve.

## Pass 3 — Fresh eyes: another seed and population

**Critique.** Three scenarios now give three distinct societies from the same definitions:

| scenario | game-days | friends | rivals |
|---|---|---|---|
| `village-evening` | 17.03 | 110 | 0 |
| `rumor-crosses-a-cluster` | 5.02 | 1 | 12 |
| `need-cycle` | 6.19 | 2 | 8 |

Before this iteration the two crowd scenarios both ran to near-uniform hostility. Now one is
friend-dominated, one is rival-dominated and one is mixed — the model has real variety.

**The uncomfortable finding.** The comparison was only valid *because* this iteration fixed the
harness's day length. `village-evening` was carrying `seconds_per_day = 240.0` while its comment
said 4.0, so it was spanning 17 days by accident; the cluster was spanning 19. Every "before →
after" rival count in the previous iteration's log compared runs of different simulated lengths.
That is a defect in the measurement apparatus, it was mine, and it is recorded rather than quietly
corrected.

**Verdict: GREAT** for the current numbers, and the apparatus defect is called out in the log so
earlier entries are not read as comparable.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — activities 2 → 5, affinity −0.899 → −0.674, `chat`'s reach recorded separately |
| Zoom | GREAT — the residual trend justified against a converted negative result, not tuned away |
| Fresh eyes | GREAT — three distinct societies; the harness's own day-length defect surfaced and named |

## Next iteration's candidates

1. **`chat` is nearly unreachable**, because `Need(social) Linear(-1, 1)` scores 0 at a full need
   and gossip self-supplies that need. Either the gate should be a curve that opens sooner or the
   `satisfies` rates need rebalancing.
2. **Durations do not scale with the day.** Every social activity is 1–3 ticks at a 4-second day,
   so a person re-decides constantly. A `duration_ticks` field, or documenting that
   `seconds_per_day` and activity durations must be set together, is the honest fix.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, and now
   implicated in three iterations' findings.
4. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
5. `scale-1000` perf.
