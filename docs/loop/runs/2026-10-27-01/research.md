# Research: rivalry needs three things the crate did not have together

The log's #2 was "rivalry in the village: `snub` needs a negative affinity nothing creates".
It turned out to be three defects in a chain, each found by measuring the one before.

## 1. `snub` was gated on *low* affinity, which is backwards

```ron
(input: Affinity, curve: Logistic(-8.0, 0.3))
```

Evaluated, that curve scores **0.92 at affinity 0.0** and falls to **0.08 at affinity 0.6**. So
the village snubbed people it had *not* yet befriended — strangers — and never anyone it knew.
Counted across 4200 ticks: the village chose `snub` for **0 person-ticks**; a cluster of
strangers chose it for **2583**.

People do not snub strangers. They snub someone they have a reason to resent, which is exactly
`Memories::grudge` — already implemented, already exposed as `ctx.grudge(a, b)`.

**Bondü & Richter 2016** (`10.3389_fpsyg.2016.00795`, Frontiers in Psychology) is the mechanism:
rejection and provocation sensitivity positively predicted *reactive* aggression — aggression in
response to perceived negative treatment. It gives direction, not a rate: the paper reports
correlations, so no number is lifted from it and the curve's 2.5 slope is a design default.

## 2. A script consideration could not see its target

The replacement — `Script("ctx.grudge(actor, target)")` — scored 0 for every `Person` option.
`ScriptContext` was built **once per person with `target: -1`**, while a consideration is scored
per `(activity, target)` pair. So `ctx.grudge`, `ctx.affinity`, `ctx.trust` and every other
target-reading `ctx` call were blind inside a consideration, for Person targets.

Fixed by binding the target per candidate. Verified: with a recorded grudge against B, the snub
now resolves to `Doing { activity: snub, target: Person(B) }`.

## 3. `patient_reaction` never moved affinity — the real root

The first two fixes made `snub` fire (7758 person-ticks, 125 insult deeds) and still produced
**0 rivals**. The reason: `patient_reaction` appraises Gratitude/Anger and moves **trust**, and
does nothing to affinity. The victim of an insult therefore keeps liking the insulter.

The SPEC says exactly that — *"Patient P: appraise `Gratitude` … or `Anger` … and
`trust(P→A) += k·harm`"* — so this is not an implementation slip, it is a gap in the model as
written. A person who has just been insulted by someone does not go on liking them.

## 4. Witness approval was the sign of the *norm*, not of the *act*

`witness_reaction` set `approval = opinions.value(deed_topic)`, and then
`affinity += 0.15 · approval`. A witness who values kindness (opinion +0.9) therefore *liked*
an actor more after watching them insult someone: measured, `affinity +0.138, respect +0.09`
after an insult.

Fixed to `approval = norm · sign(harm)`, so a kindness-valuing witness disapproves of a harmful
deed and approves of a helpful one. The SPEC's own phrasing (*"the norm a witness's opinion
judges it by: opinion > 0 approves"*) is ambiguous, which is why it went unnoticed.

## What that produced

| scenario | before | after |
|---|---|---|
| `rumor-crosses-a-cluster` | 19 friends, **11 rivals** | 0 friends, **30 rivals** |
| `need-cycle` | 5 friends, **21 rivals** | — |
| `village-evening` | 171 friends, **0 rivals** | 79 friends, **0 rivals** |

In a synthetic crowd of 12 strangers standing together, **119 of 132 ties** end below −0.4
affinity and are labelled `Rival`. The mechanism works.

The village still has zero rivals, and that is now a finding rather than a defect: its people
chat and compliment each other to positive affinity *before* anyone accumulates a grudge
against them, and `snub` needs a grudge. A village of friends cannot become a village of
enemies in seventeen days — which is believable.

## The corpus

Searched for the negative-tie literature: `relationship deterioration betrayal trust violation`,
`grudge revenge reciprocity negative tie formation dislike`, and
`victim liking aggressor`. Best scores 0.500, and the hits are clinical or unrelated (couple
therapy, intimate-partner violence). The one usable source is Bondü & Richter above, and it is
cited for the mechanism, not for a number. `docs/research/rumor-and-reputation.md` records it.
