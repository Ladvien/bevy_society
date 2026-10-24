# The loop log

One entry per iteration: the change, why, evidence, before/after numbers, the dead-code inventory delta, the three verdicts, the commit, and the top 5 remaining items.

The loop itself is `/.omp/commands/build-loop.md`; the scenarios it runs are `scenarios.yaml`.

## How to read this file

- **Dead-code inventory** is `cargo +1.98.1 clippy --all-targets --all-features -- -D warnings`: the count of unused items. It should only ever shrink as in-flight systems land, and each entry should name the system that will consume it. Unused items that nothing will consume are dead weight, not work in progress.
- **Verdicts** are always three, in order: behaviour, zoom, fresh eyes. Only three GREATs ship.
- **Evidence** paths are relative to the iteration's `runs/` directory.

## Iterations

_none yet_

## 2026-10-24-01 — `Signal::Script` evaluated through the engine

**Focus asked for:** mental health. **Not built**, and the reason is recorded in the run's
`research.md`: the corpus has no simulation model for it. Best hits peaked at 0.500, and the two
strongest are a neuroscience review and a conceptual allostasis paper with no rate or formula. Adding a
mental-health mechanism would have meant a number no paper supplies.

**Change.** A definition's `Script("…")` consideration was accepted, compiled and documented — and
scored `0.0` in every decision, so an activity whose scoring was a script could never be chosen. Nothing
errored; the activity simply never happened. `signal_value` now evaluates it through the existing
`ScriptEngine::eval_f32`, with the actor bound to the person deciding.

**Evidence.**
- `evidence/` — the before-fix trace (`Some(0)`, idle) and the eval table.
- `review.md` — three passes; the first returned NOT YET and produced the actor-binding test, which in
  turn found a second defect: a missing person bound `actor` to `0`, a real person id, so a script
  would silently read somebody else's state. Now `-1`.

**Before → after.** `Script("1.0")` consideration: idle → activity chosen. `ctx.need(actor, …)`
scenarios: indistinguishable → tired acts, rested idles. Tests 8 → 11 in `tests/behavior.rs`; suite
16 binaries green in both feature sets.

**Dead-code inventory.** 6 → 5. `eval_f32` was the one dead item that was a bug rather than an unused
accessor.

**Verdicts.** Behaviour GREAT (second attempt). Zoom GREAT with a doc note owed (a successful 0 is as
final as a failure — now in the code comment, not yet in the guide). Fresh eyes NOT YET for the crate:
the 480-frame capture reports `friends 0, barks 0, rumor_reach 0.040` against thresholds of 3, 6 and
0.5.

**Commit.** See `git log` for this run's commit.

**Top 5 remaining.**
1. The capture: no friendships, no barks, almost no rumor spread over 480 frames. The clip is the first
   thing a visitor sees.
2. `Signal::Field` scores 0 on every decision, the mirror of the bug fixed here.
3. `ActivitySpec::barks` is never read: the SPEC says an activity's barks are thought lines on start.
4. `docs/guide/authoring.md` does not say a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000`, still `pending`).

## 2026-10-25-01 — The evidence harness, and the two numbers it changed

**Why this and not the log's top item.** The log had said for two iterations that the capture
formed no friendships, and every judgement about it was made from one end-state number. The
loop's step 6 makes plots and a narrative a mandatory gate; neither existed
(`docs/loop/runs/*/evidence/` held a stray cargo log). Building the gate came first, and it chose
the work.

**Change.** `examples/evidence` runs a scenario headlessly and writes `narrative.txt`,
`series.csv` and `summary.json`. Three scenarios now produce evidence under
`runs/2026-10-25-01/evidence/`. It then exposed two defects, both in the *scenario* rather than
the model:

1. **The run was too short for ties.** Same code, same seed, same village: 4.1 game-days gives
   0 friends and 0 rivals; 17 game-days gives 125 and 0. Affinity moves at
   `mere_exposure_per_day 0.05` against `affinity_relax_per_day 0.01`, and a tie measured at the
   end of the short run read `aff 0.067, familiarity 0.818` — a neutral acquaintance, exactly
   what those rates predict.
2. **Walking and its timeout are in different clocks.** `speed` is metres per real second
   (`0.022 m/tick`), `travel_timeout_hours` is game hours (21 ticks at `seconds_per_day = 4.0`).
   A 14 m walk needed 640 ticks. Every activity ended interrupted, `satisfies` never paid out,
   and needs sat at 0.000.

**Evidence.** `runs/2026-10-25-01/evidence/{village-evening,need-cycle,rumor-crosses-a-cluster}/`.
The controlled comparison is `village-evening` at two lengths and
`rumor-crosses-a-cluster`, which forms **11 rivalries** — proving rivalry is reachable and the
village's zero is a property of that village.

**Before → after** (the capture, same seed):

| | before | after |
|---|---|---|
| `rumor_reach` | 0.040 | **0.800** |
| `friends` | 0 | **256** |
| `rivals` | 0 | 0 |
| `barks` | 1 | **3076** |
| interrupted endings | 37 | **0** |

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT (second attempt — the first demanded the series rather than the
summary, which is the whole point of the harness). Zoom GREAT, with `rivals 0` and a low mean
need level named. Fresh eyes GREAT for the crate, NOT YET for the village's rivalry.

**Escalated.** `docs/loop/needs-decision/travel-time-in-two-clocks.md` — three options,
recommendation to make the timeout real-time in a later iteration as a breaking change.

**Commit.** See `git log`.

**Top 5 remaining.**
1. Rivalry: `snub` needs a negative affinity the village never creates. Plant a rumor about a
   villager and let gossip drive it, or give the definitions a friction input.
2. Mean need level: `rest` decays 0.7/day while sleep restores at most 1.2/night, so the village
   lives near 0.15. One of those two starter numbers is wrong for a 24-hour day.
3. `Signal::Field` and `ActivityTarget::Gradient` still score nothing. Moussaïd 2011 is the right
   model and its equations were stripped in conversion — re-convert before implementing.
4. `docs/guide/authoring.md` does not say a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000`, still `pending`).

## 2026-10-26-01 — The social need gets a rate; two low needs turn out to be correct

**Why.** The log said "mean need level" and attributed it to `rest` decaying 0.7/day against
sleep's 1.2/night — arithmetic, not measurement. The harness built last iteration could test
that per need, so it did first.

**Finding.** All four needs sat below 0.35 for 94–96% of ticks, which a cycling need does not do.
Three distinct causes, measured separately:

1. **`social` was genuinely under-supplied.** It decays 1.0/day and its entire supply was
   one-shot interaction effects (+0.2 per accepted chat), while `eat` paid food at 1.2 per
   game-hour and `sleep` paid rest at 0.15. `chat` and `gossip` — the two activities that serve
   social — were the only two declaring `satisfies: []`. Counted: **420 rises against 20 365
   falls**, one rise per 80 ticks where break-even needs one per 51.
2. **Every activity is shorter than the walk to it.** A duration is game-time; a walk is
   real-time (`speed / 64` m/tick). At this day length a 3.5 m walk is 160 ticks and a 0.3
   game-hour chat is 3. This is the second victim of the clock decision escalated last
   iteration; the scenario now starts villagers arrived.
3. **`food` and `fun` being low is correct.** `eat`'s consideration is
   `Need(food) Logistic(-10, 0.35)` — 0.0015 at a full need, 0.50 at 0.35. A villager eats when
   hungry. A mean of 0.056 is that curve's shape, not a leak.

**Change.** `chat` gains `satisfies: [("social", 1.2)]`, `gossip` gains
`[("social", 1.0)]`. Nothing else.

**Evidence.** `runs/2026-10-26-01/evidence/` — before and after at the same seed, plus
`rumor-crosses-a-cluster` as the fresh-eyes run.

**Before → after** (4200 ticks, seed 7):

| need | before | after |
|---|---|---|
| social | 0.105 | **0.225** |
| rest | 0.218 | 0.200 |
| food | 0.048 | 0.056 |
| fun | 0.041 | 0.046 |

Capture: reach 0.640, 252 friends, 3376 barks — within noise of last iteration, `rivals 0`
unchanged.

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT (the residual shortfall attributed to the clock, not ignored).
Zoom GREAT — `food`'s low mean explained as a correct logistic, and the rise:fall ratio fixed.
Fresh eyes GREAT — the cluster scenario still forms **11 rivalries**, so the change did not
flatten the society.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **The clock decision.** Make `travel_timeout_hours` real-time. Until then every short
   activity in every host is unreachable without a scenario work-around.
2. Rivalry in the village: `snub` needs a negative affinity that nothing creates.
3. `Signal::Field` / `ActivityTarget::Gradient` score nothing; Moussaïd 2011 needs re-converting.
4. `docs/guide/authoring.md` does not say a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000`, still `pending`).

## 2026-10-27-01 — Four defects between "rivalry" and one working rivalry

**Why this and not the log's #1.** #1 is the clock decision, which this loop escalates rather than
decides: it is written up in `needs-decision/travel-time-in-two-clocks.md` with three options and a
recommendation. Taking it would be deciding it. #2 is measurable with the harness, so it was the pick.

**Finding.** Four defects in a chain, each found by measuring the one before.

1. **`snub` gated on low affinity.** `Logistic(-8, 0.3)` scores 0.92 at affinity 0.0 and 0.08 at
   0.6, so the village snubbed strangers and never anyone it knew. Counted: **0** person-ticks in
   the village against **2583** in a cluster. Now reads `ctx.grudge(actor, target)`.
2. **Script considerations could not see their target.** `ScriptContext` was built once per person
   with `target: -1`, so every target-reading `ctx` call was blind in a consideration for a Person
   option. Now bound per candidate. Proven: a grudge of 0.9 resolves to `snub → Person(B)`.
3. **Witness approval was signed by the norm, not the act.** A kindness-valuing witness *liked* an
   insulter more: `+0.138` affinity, `+0.09` respect. Now `norm · sign(harm)`.
4. **`patient_reaction` moves no affinity** — a victim keeps liking their insulter. The SPEC
   specifies exactly that and no reachable source gives a rate, so it is recorded open rather than
   invented.

**Evidence.** `runs/2026-10-27-01/evidence/` and `eval.md`.

**Before → after.**

| scenario | friends | rivals |
|---|---|---|
| `rumor-crosses-a-cluster` | 19 → 0 | **11 → 30** |
| `need-cycle` | — → 5 | **0 → 21** |
| `village-evening` | 171 → 79 | 0 → 0 |

Synthetic crowd of 12 strangers: **119 of 132** ties end below −0.4 affinity.

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT (the approval-sign change flagged as an interpretation, since the
SPEC's wording is ambiguous). Zoom GREAT for the village, with a stranger-crowd saturation at
−0.994 named rather than hidden. Fresh eyes GREAT — 30 rivals in one scenario and 0 in another from
the same definitions, so the model has more than one attractor.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **Hostility saturation.** In a crowd with no positive activity, affinity pins at −1.0 and
   `affinity_relax_per_day = 0.01` cannot damp it. Either relaxation scales with |a| or the deed's
   affinity weight needs a floor.
2. `patient_reaction` still moves no affinity; a victim keeps liking their insulter. Needs a cited
   rate or an explicit design-default decision.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, still the
   largest host-ergonomics tax.
4. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
5. `scale-1000` perf.

## 2026-10-28-01 — The hostility runaway was three things, and damping was not one of them

**Why.** The log's #1 was hostility saturation, which the previous review returned NOT YET on:
affinity pinned at −1.0 with `affinity_relax_per_day = 0.01` unable to damp it. The obvious move —
raise relaxation, or floor the deed's weight — would have tuned a symptom.

**Finding.** Reproduced the runaway first: twelve strangers, mean affinity −0.047 → **−0.960**
across 2000 ticks, monotone, 130 of 132 ties below −0.4. Then measured the mechanism, which was
three separable causes:

1. **`chat` could never run.** The census showed only gossip (5) and snub (7) ever chosen —
   `chat` ran **zero times in 2000 ticks**. `chat`'s `Need(social) Linear(-1, 1)` scores 0.0 at a
   full need, and `gossip` self-supplies that need, so chat's own gate never opened. The cluster
   scenario also gave its people no places, leaving Person targets as the only options.
2. **`snub`'s slope saturated.** `Linear(2.5, 0)` reaches 1.0 at grudge 0.4, which one witnessed
   insult exceeds. Now `Linear(1.0, 0)`.
3. **The harness ran a 60× shorter day than the definitions assume.** A 0.1-hour `snub` lasted
   **1.07 ticks**, so people re-decided every tick and repeated the best option: 38 snubs per
   person per game-day. The village branch also carried a `240.0` its own comment contradicted,
   so `village-evening` was spanning 17 days by accident and the cluster 19.

**The uncomfortable part.** Cause 3 means every "before → after" rival count in the previous
iteration's log compared runs of different simulated lengths. The apparatus was wrong, it was
mine, and it is recorded here so those entries are not read as comparable.

**What the literature said, and why no damping was added.** Searched de-escalation, forgiveness
and negative influence. **Takács, Flache & Mäs 2016** (*Discrepancy and Disliking Do Not Induce
Negative Opinion Shifts*, `10.1371_journal.pone.0157948`) was **downloaded and converted this
iteration** (21 pages, 121 s). Its result: **negative influence is not empirically supported** —
people do not shift away from those they dislike, they stop being influenced by them. Damping is
therefore not a rule to add to the influence model; it lives in *who one chooses to interact
with*, which is the decision layer the hearth addresses.

**Evidence.** `runs/2026-10-28-01/evidence/` and `eval.md`.

**Before → after** (crowd of 12, same seed):

| | before | after |
|---|---|---|
| distinct activities in play | 2 | **5** |
| mean affinity at t=1200 | −0.899 | **−0.674** |
| ties below −0.4 at t=1200 | 123 | **12 of 132** |

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT — the monopoly is broken and five activities appear; `chat` at 1 is
recorded separately as still nearly unreachable. Zoom GREAT — the residual downward trend is
justified against a converted paper rather than tuned away. Fresh eyes GREAT — three scenarios now
give three distinct societies (110/0, 1/12, 2/8 friends/rivals), and the harness's own day-length
defect surfaced.

**Commit.** See `git log`.

**Top 5 remaining.**
1. `chat` is nearly unreachable: `Need(social) Linear(-1, 1)` scores 0 at a full need while gossip
   self-supplies it. Either the gate's curve or the `satisfies` balance needs work.
2. Durations do not scale with the day: every social activity is 1–3 ticks at a 4-second day. A
   `duration_ticks` field, or documenting that `seconds_per_day` and durations must be set
   together, is the honest fix.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, now
   implicated in three iterations' findings.
4. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
5. `scale-1000` perf.

## 2026-10-29-01 — A warning for the bug that kept being silent

**Why, and what changed mid-iteration.** Started on the log's #1: `chat` unreachable where gossip
self-supplies the social need. Measured first — the village is healthy (chat:gossip 1:1.7), only
the cluster starves (1:24). Then tried to fix it twice:

| attempt | chat person-ticks | outcome |
|---|---|---|
| baseline | 128 | — |
| gossip `satisfies` 1.0 → 0.3/h | **106** | worse, reverted |
| gossip interaction 0.15 → 0.05 | **153** | +20%, still starved, reverted |

The arithmetic that stopped it: a gossip fires ~17×/person/day, so any per-instance supply above
0.06 covers the 1.0/day decay, and below that gossiping is worthless. **Not a tunable rate** — two
activities serving one need with no differentiation, which is a schema question.

So the iteration built the log's #2 instead, reframed by what #1 taught: the recurring defect is an
activity shorter than one tick, and it has always been silent.

**Change.** `BehaviorPlugin::finish` warns, naming each activity under two ticks with its own
arithmetic; `SocietyDefs::activity_durations()` is public so a host can run the same check.

```
WARN bevy_society::behavior: ... 1 activities are shorter than two society ticks at
seconds_per_day 4 (one tick is 0.094 game-hours), so their people will re-decide every
tick: snub (0.1h = 1.1 ticks)
```

**Evidence.** `runs/2026-10-29-01/eval.md`; the two reverted attempts are in
`evidence/cluster-gossip-satisfies-0.3/` and `evidence/cluster-interaction-0.05/`.

**Before → after.** No behaviour change (both behavioural attempts were reverted). The observable
change is a startup log line where there was silence. Tests 16 → **17**: `tests/degenerate.rs`
brackets the threshold at 4.0 s (degenerate) and 240.0 s (sane).

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT — warning verified by running it, with the missing log-capture
assertion named (the tests pin the invariant, not the line). Zoom GREAT — threshold bracketed from
both sides, `idle` exempt, default day silent. Fresh eyes GREAT — fires at 1.0 s and 4.0 s, silent
at 240 s.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **`chat` vs `gossip` share one need.** Write the `needs-decision/` note: a second need, or a
   gossip variant gated on having something to tell. Oldest unresolved finding.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, implicated in
   three iterations, and provably the same class as the duration warning just added.
3. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
4. A log-capture harness, so warnings can be asserted rather than read by hand.
5. `scale-1000` perf.

## 2026-10-30-01 — Gossip needs something to tell, and the starvation survives

**Why.** The log's #1 was `chat` starving against `gossip`, escalated last iteration with three
options. The recommendation was option 2 — gate gossip on having information — because it needs no
schema change and matches what the literature says gossip is (**Sommerfeld et al. 2007**,
`10.1073/pnas.0704598104`: gossip is reputation *information*, distinct from conversation).

**The gap that had to be closed first.** Option 2 could not be expressed. `ctx.knows(a, rumor)`
requires a rumor id, and an authored file has no way to name one. Added `ctx.tellable(a)`: whether a
person knows a rumor they are still spreading.

**Change.** `gossip` gains `Script("if ctx.tellable(actor) { 1.0 } else { 0.0 }")` as its leading
consideration.

**Before → after.**

| activity | village before | after | cluster before | after |
|---|---|---|---|---|
| `chat` | 8746 | **9312** | 128 | 140 |
| `gossip` | 15050 | 14160 | 3093 | 3186 |
| `snub` | 6004 | **1617** | 2527 | 2188 |

**The negative result, measured before the change was written.** The cluster's 1:24 ratio is
unchanged. That scenario produces **279 rumors**, so `tellable` is true for almost everybody and the
gate never closes. A gate on having something to tell only differentiates where rumors are scarce —
the village, 28 rumors among 24 people. The decision note is updated: option 1 (a second need) is now
the only remaining path, and tuning `tellable` to bite harder would be inventing a threshold to move
a number.

**The surprise.** `snub` falls **73%** in the village. Chain: gossip interactions produce insults →
insults produce grudges → grudges drive snubs. Fewer gratuitous gossips, far fewer grudges. At 1617
person-ticks over 24 people and 17 days that is ~4 ticks per person per day: a village with a few
grudges, not a village without them.

**Evidence.** `runs/2026-10-30-01/eval.md`; capture `{"rumor_reach": 0.360, "friends": 80,
"rivals": 37, "barks": 3648}` — all four thresholds met.

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT — chat up, gossip down, snub −73%, and the second-order effect named.
Zoom **NOT YET** for the cluster's ratio (279 rumors defeat the gate), GREAT for the gate itself at
every edge, including subject-blindness documented in the code. Fresh eyes GREAT — capture clears all
four thresholds and the cluster still reaches 0.333.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **The cluster's 1:24 starvation survives** — option 1: a second need (`belonging` for chat,
   `social` for gossip). A schema change; now the only remaining path.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, implicated in
   three iterations.
3. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
4. A log-capture harness so warnings can be asserted rather than read by hand.
5. `scale-1000` perf.
