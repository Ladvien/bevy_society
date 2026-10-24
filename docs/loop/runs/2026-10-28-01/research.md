# Research: why a crowd of strangers becomes uniformly hostile

The log's #1 was hostility saturation, which the previous review returned NOT YET on: in a
crowd with no positive activity, affinity pinned at −1.0. This iteration measured the mechanism
instead of damping the symptom, and found the saturation was three separable things — one of
them in my own scenario settings.

## The measurement

Twelve strangers standing together, sampled every 200 ticks:

| tick | mean affinity | ties below −0.4 |
|---|---|---|
| 100 | −0.047 | 1 |
| 500 | −0.437 | 71 |
| 1000 | −0.842 | 119 |
| 2000 | −0.960 | 130 of 132 |

Monotone, no floor. Confirmed as a runaway rather than drift.

## Three causes

### 1. `chat` could never run in that crowd

The census showed only activities **5 (gossip)** and **7 (snub)** ever chosen — `chat` (id 4) ran
**zero times in 2000 ticks**.

`chat`'s first consideration is `Need(social) Linear(-1.0, 1.0)`, which scores **0.0 at a full
need** and rises only as the need falls. `gossip` also serves social (satisfies 1.0/h plus a
+0.15 interaction effect), so gossip held the need high enough that chat's own gate never
opened. Evaluated: at social 0.3, `chat` scores 0.521 and `gossip` 0.591; chat only overtakes
gossip below social ≈ 0.2, which gossip prevents.

So the crowd had exactly one positive option and one negative one. That is a **scenario**
defect, not a model one: the cluster scenario gave its people no places at all, so there was
nothing to do but talk to each other.

### 2. `snub`'s consideration slope saturated at grudge 0.4

`Linear(2.5, 0.0)` on `ctx.grudge` reaches 1.0 at grudge 0.4 — and a single witnessed insult
leaves more than that. Once saturated, snub outranks every other option permanently, and each
snub creates fresh witnessed memories for everyone watching.

Reduced to `Linear(1.0, 0.0)`, so a maximum grudge (1.0) still means "snub" but half of one
(0.5) does not dominate. Measured: mean affinity at t=1200 fell from −0.899 to −0.820.

### 3. The scenarios shortened the game-day 60× without scaling durations

This is the one that invalidated every cross-iteration comparison in the log.

`ActivitySpec::duration_hours` is **game** hours and a tick is `dt_days`. At the crate default
(`seconds_per_day = 240`) a tick is 0.025 game-hours, so a 0.1-hour `snub` lasts 4 ticks and a
0.3-hour `chat` 12. The evidence harness ran at `seconds_per_day = 4.0` — a 60× shorter day —
where a tick is 0.094 game-hours and those same activities last **1.07 and 3.2 ticks**.

A one-tick activity ends immediately, so a person re-decides every tick and repeats whatever
scored best. That is the "everyone doing the same thing" symptom, and it is why `snub` could fire
**38 times per person per game-day**.

Worse, the harness had drifted: the village branch carried `seconds_per_day = 240.0` while its
own comment claimed 4.0, so `village-evening` was spanning 17 game-days by accident and the
cluster was spanning 19 — which is why the previous iteration's "11 → 30 rivals" and this
iteration's numbers were not comparable. Both branches now state 4.0 and the cluster spans 5.02
days, matching its comment.

## The corpus

Searched for the negative-tie and de-escalation literature: `negative reciprocity escalation
de-escalation vendetta conflict spiral`, `forgiveness reconciliation dampening hostility`, and
`attitude polarization bounded extremity`. Three findings:

- **Flache et al. 2017** (`10.18564_jasss.3521`, already converted) is a review of social
  influence models. Its reference list surfaced the two papers below, which is its value here.
- **Takács, Flache & Mäs 2016**, *Discrepancy and Disliking Do Not Induce Negative Opinion
  Shifts* (PLoS ONE, `10.1371_journal.pone.0157948`) was **downloaded and converted this
  iteration** to read it. Its result is directly relevant: negative influence — shifting *away*
  from someone you dislike — is not empirically supported. People do not move their opinions away
  from those they dislike; they simply stop being influenced by them.
- No usable quantitative model of forgiveness or reconciliation was found (best score 0.35, and
  the hits are clinical: marital therapy, abuse recovery). So no repair mechanism was invented.

Takács et al. is the reason this iteration did **not** add a "hostility damps itself" rule: the
literature says the damping is not in opinion dynamics, it is in *who you choose to interact
with*. That is exactly what fixing the scenario did — giving the crowd an alternative.

## Verdict

Three measured causes, one of them a defect in the measurement apparatus itself. No new model,
no invented rate, and one newly converted paper supporting the negative result.
