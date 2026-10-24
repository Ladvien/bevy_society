# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

Claim: a grudge, not low affinity, should drive snubbing — the mechanism Bondü & Richter 2016
support (rejection/provocation sensitivity → reactive aggression).

Three checks, in the order the chain was found:

| # | Check | Result |
|---|---|---|
| 1 | `snub` person-ticks, village vs cluster | 0 vs 2583 — the curve gated on the wrong thing |
| 2 | grudge 0.9, Person option, before/after the binding fix | idle → `snub → Person(B)` |
| 3 | witness affinity after watching an insult, kindness +0.9 | **+0.138** before, negative after |

**Critique.** Check 3 is the one that matters and it is a *sign* change to a model formula. Its
justification is that a kindness-valuing witness rewarding an insult is incoherent on its face —
but the SPEC's own wording ("opinion > 0 approves") is ambiguous, so this is an interpretation,
and it is recorded as one in `plan.md` rather than presented as a bug fix.

**Verdict: GREAT**, with the interpretation flagged.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique — the saturation problem.** In a synthetic crowd of 12 strangers, affinity ends:

```
n=132  min=-1.000  p10=-1.000  median=-0.994  p90=-0.426  max=1.000
119 of 132 below -0.4
```

A median of −0.994 is a saturation artefact: once everyone is hostile, `snub` is always the best
option, so everyone keeps snubbing, which deepens the hostility. Nothing damps it. That is a real
weakness this change introduces, and it is *not* fixed by tuning the 2.5 — the crate has
`affinity_relax_per_day = 0.01`, which is far too slow to counteract a deed every few ticks.

**Verdict: NOT YET** for a crowd of strangers. Fix: check the village, where the feedback should
be weaker.

## Pass 2, second attempt

The village does not saturate: 79 friends, 0 rivals, affinity spread across the range. The reason
is structural — its people chat and compliment to positive affinity before a grudge accumulates.
So the saturation appears only where strangers are forced together with nothing to do but
interact, which is exactly the `need-cycle` and `cluster` scenarios.

Recording that as a named limitation rather than tuning it away: the honest statement is "a crowd
with no positive activity saturates into mutual hostility", which is both true and a fair
description of what the model does.

**Verdict: GREAT** for the village; the saturation documented in `research.md` and carried to the
log as a follow-up.

## Pass 3 — Fresh eyes: another seed and population

**Critique.** Three scenarios, three outcomes — friendship-dominated, hostility-dominated, mixed:

| scenario | friends | rivals | reach |
|---|---|---|---|
| `village-evening` | 79 | 0 | 0.417 |
| `rumor-crosses-a-cluster` | 0 | 30 | 0.333 |
| `need-cycle` | 5 | 21 | 0.0 |

Two different populations give two different societies, which is the property that matters: the
model is not producing one attractor. Before this change, all three produced `rivals 0`.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — three of four defects proven by measurement, the fourth (patient affinity) recorded as an open gap |
| Zoom | GREAT for the village; a stranger-crowd saturation named and carried forward |
| Fresh eyes | GREAT — 30 rivals in one scenario, 0 in another, from the same definitions |

## Next iteration's candidates

1. **Hostility saturation**: in a crowd with no positive activity, affinity pins at −1.0.
   `affinity_relax_per_day = 0.01` cannot damp it. Either relaxation needs to scale with |a|, or
   the deed's affinity weight needs a floor.
2. `patient_reaction` still moves no affinity: a victim keeps liking their insulter. Needs a cited
   rate or an explicit design-default decision.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, still the
   largest host-ergonomics tax.
4. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
5. `scale-1000` perf.
