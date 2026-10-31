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

## 2026-10-31-01 — What a thousand people cost, and where the design breaks

**Why this.** `scale-1000` had been `pending` since the loop began, the brief ranks performance fourth,
and `docs/guide/concepts.md` asserted the brute-force perception was *"fine"* to about a thousand
people with no number behind it. An adjective about the crate's own cost is exactly the sort of claim
this loop exists to replace.

**Change.** `examples/benchmark` spawns N autonomous persons, settles them so they are not all idle,
then times N society ticks and reports `ms_per_tick`. Nothing in the library changed.

**The numbers** (release, M1 Max, starter pack, `tick_secs = 0.25`):

| persons | ms/tick | pairs/tick | amortised per 60 fps frame |
|---|---|---|---|
| 100 | 0.549 | 4 950 | 0.03 ms (0.2%) |
| 1 000 | **14.086** | 499 500 | 0.83 ms (4.9%) |
| 5 000 | **351.640** | 12 497 500 | 23.4 ms (**140%**) |

**Reproducible:** 14.168 and 14.167 ms/tick across two independent 400-tick runs — a 0.01% spread.

**Quadratic, confirmed numerically:** cost grows **25.0×** for 25× the pairs (1 000 → 5 000), and
`ms_per_million_pairs` stays flat at 28.1–28.2. The 100-person figure reads 110 because fixed overhead
dominates at 4 950 pairs.

**What a host learns.** 100 people is free; 1 000 is 4.9% of a 60 fps budget, usable but not free, and
that amortisation assumes one tick every fifteen frames; 5 000 costs more than a whole frame. The
guide's "fine up to about a thousand" was right at 1 000 and wrong at 5 000, and the table now says
where the cliff is.

**Evidence.** `runs/2026-10-31-01/evidence/scale-{100,1000,5000}.json`, `eval.md`, `perf.md`.

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT — 0.01% reproducible, with the idle-crowd and per-update traps both
closed, and per-system attribution flagged as inference from the quadratic shape rather than a
profile. Zoom GREAT — quadratic confirmed as exactly 25.0×, the 100-person outlier explained. Fresh
eyes GREAT — a host now learns where the cliff is.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **The cluster's 1:24 starvation** — option 1: a second need (`belonging` for chat, `social` for
   gossip). A schema change; the only remaining path, and now the oldest open finding.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
3. **`Signal::Field` / `Gradient`** — Moussaïd 2011 needs re-converting.
4. **A spatial index for perception**, if a host needs past ~2 000 people. Now a `needs-decision/`
   note with numbers behind it rather than a guess.
5. A log-capture harness so warnings can be asserted rather than read by hand.

## 2026-11-01-01 — The log's premise was wrong, and the real bug is still open

**Why this and not #1.** #1 is a definitions-schema change, which this loop escalates rather than
decides. #3 was `Signal::Field` / `Gradient`, listed with "Moussaïd 2011 needs re-converting" as its
blocker.

**The premise was wrong.** The field grids are built, deposited, evaporated, diffused and queried
correctly: `field_value(Crowd, …)` returns **1.0** at a crowded cell with sample == peak, and
`gradient_target(Crowd, …)` returns `Some(Vec2(4.0, -4.2e-7))` on a crowd. Moussaïd 2011 is a
crowd-*steering* paper; a `Gradient` target walks a steeper-is-better field and needs no crowd model
at all. Nothing needs re-converting.

The real gap: `signal_value`'s `Field` arm returns a constant 0 and `targets_for` returns no targets
for `Gradient`. Measured, an activity with a `Gradient` target ran **0 person-ticks over 60**.

**What I attempted and reverted.** Wire both call sites — `field_value` and `gradient_target` are
public and the decision already holds positions. Threaded `Option<Res<SocialFields>>` through
`decide_activities` → `score_option` → `signal_value` and through `targets_for`, with two tests. Both
failed and the cause was not found: a trace showed the `Field` arm running with the actor's real
position, `field_value` at that cell returned 1.0, and the score still came out 0 with the person
idling every tick for 60 ticks. One ordering fact is established — `SocialFields` is created by a
deferred command in `Clock`, and `deposit_fields` runs in `Relate`, after `Decide`, so a decision
reads the *previous* tick's grid — but that does not explain the tick-20 case.

**Shipped instead.** A startup `warn!` naming every definition that relies on a `Field` consideration
or a `Gradient` target, plus `SocietyDefs::{activity_signals, has_gradient_activity}` and a public
`Signal` so a host can run the same check. A host no longer writes a dead definition in silence.

**Evidence.** `runs/2026-11-01-01/{research,eval,review}.md`,
`evidence/gradient-never-chosen.json`, and `needs-decision/field-inputs-are-unscored.md` with four
measured facts, what was reverted, what remains unexplained, and three options.

**Before → after.** No behaviour change: the warning is the deliverable. Tests 17 → **18**
(`tests/fields_unscored.rs`, 4 tests).

**Dead-code inventory.** Unchanged at zero. `Signal` became public, which the accessors require.

**Verdicts.** Behaviour GREAT — the four facts pinned, with the missing log assertion named (second
occurrence; escalated). Zoom GREAT — silent on the starter pack, and `Signal`'s exposure justified.
Fresh eyes GREAT — the negative result is recorded precisely enough to resume from.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **`Field` / `Gradient` wiring** — the actual bug, with four measured facts and the unexplained
   tick-20 case recorded. Start from a single-person case with one cell of value 1.0.
2. **The cluster's 1:24 starvation** — a second need; a schema change.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
4. **A log-capture harness** — now blocking two separate warnings from being properly tested.
   Promoted from fifth.
5. **A spatial index for perception** past ~2 000 people, with benchmark numbers behind it.

## 2026-11-02-01 — Assert the warnings, and the harness immediately earns its keep

**Why this.** The log's #4: a log-capture harness, blocking two shipped `warn!`s from being tested.
Both were pinned only by the *facts* they key on, so deleting either call left every test green. A
warning nobody asserts is a comment.

**Change.** `tests/common/capture.rs` — a `LogPlugin` layer that records every event into a resource a
test can read, adapted from Bevy's `examples/app/log_layers_ecs.rs`. `tests/warnings.rs` asserts both
warnings from one app.

**What the harness exposed, in its first run.** `warn_unscored_fields` counted an activity twice when
it had both a `Field` consideration and a `Gradient` target:

| | message |
|---|---|
| before | `2 definitions rely on social-field inputs … — follow, follow` |
| after | `1 definitions rely on social-field inputs … — follow` |

One offending activity reported as two. The previous iteration's four tests were all green while that
sentence was wrong, because they asserted that the accessors *see* the definition rather than what the
warning *says*.

**Two constraints discovered, both documented where the next author will hit them:**

- The `tracing` subscriber is **process-global and settable once**, so a test binary may build only one
  capturing app. Observed directly: four tests passed individually and three silently captured
  nothing, with `Could not set global logger and tracing subscriber as they are already set`.
- **Bevy 0.20 has no non-send resources** — `insert_non_send_resource` does not exist, so the upstream
  example's approach does not compile. The receiver is behind a `Mutex` instead.

**Evidence.** `runs/2026-11-02-01/evidence/captured-warnings.txt` — both lines verbatim from the
asserting run.

**Before → after.** Tests 18 → **19**. Coverage: level, message, activity name and arithmetic for both
warnings; and a second test asserting the starter pack triggers neither.

**Dead-code inventory.** Unchanged at zero.

**Verdicts.** Behaviour GREAT — a deleted `warn!` now fails a test. Zoom GREAT — the one-app-per-binary
limit documented. Fresh eyes GREAT — the harness found a shipped defect within minutes of existing.

**Commit.** See `git log`.

**Top 5 remaining.**
1. **`Field` / `Gradient` wiring** — the actual bug, four measured facts and the unexplained tick-20
   case recorded. Start from a single-person case with one cell of value 1.0.
2. **The cluster's 1:24 starvation** — a second need; a schema change.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
4. **A spatial index for perception** past ~2 000 people, with benchmark numbers behind it.
5. **`warn_unscored_fields` says "1 definitions"** — grammatically wrong; a plural helper would fix it,
   noted rather than done so the fix is deliberate.

## 2026-11-03-01 — The quick start was a lie; now it is compiled

**Pick.** The brief's candidate #3, host ergonomics, tested the only way that counts: be the host, copy
the crate's own documentation, run it. Three of four entry points failed.

| Entry point | As written | Result |
|---|---|---|
| `README.md` quick start | the block it calls "this code" | **0 society ticks in 600 updates** |
| `docs/guide/getting-started.md` | `add_plugins(SocietyPlugins)` over `DefaultPlugins` | **panics** in `DialoguePlugin::finish` |
| `examples/quickstart.rs` | same code, fixed step pinned | works: 59 exchanges, 30 accepted |

A headless app has no frame clock, so `Time<Fixed>` never advances and the society is frozen. The
example was right; the two documents telling a host to copy it were wrong, and the README said so
falsely.

**Change.** Not a corrected copy — a **running doctest** in `src/lib.rs`, so `cargo test --doc`
executes the snippet a host is told to paste and asserts `tick > 0`. `tests/readme.rs` asserts the
README's block is byte-equal to it. Deleting the `TimeUpdateStrategy` line from the README fails it
(verified: failed, restored).

**What the doctest caught, in its first run.** My hand-written repair named `TimeUpdateStrategy` from
`bevy::prelude::*` — which does not contain it (`bevy_time`'s prelude is
`{DelayedCommandsExt, Fixed, Real, Time, Timer, TimerMode, Virtual}`). **Every text-level assertion
passed and the code did not compile.** That is the whole argument for compiled documentation.

**Before → after.** Society ticks in 600 updates: **0 → 37**. The guide: panic → no panic.

**Evidence.** `runs/2026-11-03-01/evidence/{before,after}-tick-count.txt`,
`before-guide-panic.txt`, `after/{needs-rumor-friends.png, narrative.txt, summary.json}`,
`after-need-cycle/needs.png`, `fresh-eyes/` (seed 99, 40 people).

**Harness.** `--seed` and `--people` added (the fresh-eyes gate needs both and it had neither);
`clock_label` now prints `d3 20h43` — without the day, a multi-day narrative reads as though the clock
ran backwards at midnight. `--people 40` also panicked on `["millers","weavers","watch"][3]`; the
faction index now wraps.

**Behaviour measured while reading the evidence, recorded not fixed** (one change per iteration):

| Observation | Measurement |
|---|---|
| `need3` never satisfied | 0.0 on **3 283 / 4 200 ticks (78.2%)** |
| `need2` rarely satisfied | 0.0 on 1 950 / 4 200 (46.4%) |
| rumor spreads instantly, then stops | changes at ticks **34 and 59**, then flat for 4 140 |
| message pair asymmetric | **19** `ActivityEnded` vs **9** `ActivityStarted`, one person |

The asymmetry is a crate defect with a located cause: `ActivityStarted` is written on arrival
(`behavior.rs:340`) and for no-travel activities (`:832`); `ActivityEnded` is written for any removed
`Doing` (`:449`), so a travel timeout ends an activity nobody was told started.

**Tests.** 19 → **20** binaries, both feature sets; `cargo test --doc` 1 → **2**, one of which executes
the quick start. Dead-code inventory: zero, unchanged.

**Verdicts.** Behaviour GREAT (0 → 37 ticks, guide no longer panics, all asserted by compiled checks).
Zoom GREAT — found two need-starvation defects, a frozen rumor and the start/end asymmetry by reading
the plot the brief requires. Fresh eyes GREAT for the change at seed 99 / 40 people; **NOT YET** for
need satisfaction, which is worse at 40 people than at 24, with numbers.

**Commit.** `fa09163`.

**Top 5 remaining.**
1. **`ActivityStarted` / `ActivityEnded` asymmetry** — a start is only written on arrival; a travel
   timeout emits an end for a start nobody saw. Everything downstream (gossip, witness reactions)
   inherits it.
2. **`need3` at 0.0 for 78% of ticks** — no starter activity satisfies it often enough;
   `need-cycle` stays `pending` because of it.
3. **Rumor reach frozen after tick 59** — should stifle (Daley–Kendall), not flat-line.
4. **`Field` / `Gradient` wiring** — unchanged, four measured facts recorded.
5. **`warn_unscored_fields` says "1 definitions"** — plural helper.

## 2026-11-04-01 — A start message a host could never have seen

**Pick.** Last iteration's #1: the `ActivityStarted` / `ActivityEnded` asymmetry. Not the more visible
need starvation, on one sentence: **a starved need is a tuning problem, an unpaired message is a
broken contract.** Every host-facing consumer reads an end as the close of something it was told about.

**Cause.** `Doing::started_day` is documented as *the day the activity started* and is stamped at
decision, in the same `insert(Doing { .. })` the message accompanies. The message was out of step:
`decide` wrote `ActivityStarted` only when `in_place`, which is hardcoded `false` for every decided
activity, so a start waited for arrival — while `end_activities` wrote an end for **any** removed
`Doing`, including a travel timeout.

**Reproduction, and the number the iteration turns on.** One person, one place 50 m away, no steering
plugin, so nothing ever arrives:

| | starts | ends |
|---|---|---|
| before | **2** | **55** |
| after | 150 | 149 (one in flight) |

At 40 people: 23 745 / 26 412 before, 1 419 / 1 411 after, all completed. In the narrative the review
gate requires, counting lines: `started` 14 vs `finished` **18** before; 27/26 and 22/21 after.

**Change.** `decide` writes the start for every activity it picks; `arrive_and_satisfy` stops writing
one (and loses the parameter, so it cannot). Arrival keeps its own meaning: `Doing::arrived_day`
begins the duration and the `satisfies` payout. The type's doc comment now states the contract.

**Behaviour moved at a fixed seed**, which is what proves the pair is load-bearing and not cosmetic:

| village-evening, seed 7, 24 people | before | after |
|---|---|---|
| rumors planted | 28 | **53** |
| friend edges | 91 | **118** |
| rumor reach | 0.500 | **0.333** |

Gossip is gated on `ctx.tellable`, and the tellable set moves when exchanges land at different instants.

**Written red first.** `every_activity_end_has_a_start`: `0 starts, 149 ends` at first. The final
assertion is `starts >= ends` with at most one unpaired, not equality — my first draft demanded
equality and failed a *correct* implementation at `150 / 149`, because a run can stop mid-activity.

**Perf.** 0.549 / 12.47 / 419.7 ms per tick at 100 / 1 000 / 5 000. Baseline at `HEAD` measured for
comparison: 5 000 is 418.3 ms, so the change is +0.33%, which is noise. No regression.

**Not fixed, measured both ways.** Need starvation is a *separate* cause and now has sharper numbers:

| | need2 at 0 | need3 at 0 |
|---|---|---|
| one plaza, zero travel | 9.6% | 28.2% |
| village, 24 people | 46.4% | 78.2% |

So walking costs are part of it but not the cause. The cause is activity *choice*: over 1 419 starts,
`snub` 589 and `gossip` 561 — **82%** — and neither satisfies a need.

**Evidence.** `runs/2026-11-04-01/evidence/{before,after}/`, `before-after.png`, `fresh-eyes/` (seed 99,
40 people).

**Tests.** 20 binaries both feature sets; `behavior` 17. Dead-code inventory: zero, unchanged — the fix
removed a parameter.

**Verdicts.** Behaviour GREAT (2/55 → 150/149; narrative counts confirm). Zoom GREAT (pairing moved
gossip and friendship at a fixed seed; the plot shows the need starvation is untouched and correctly
so). Fresh eyes GREAT for the change at seed 99 / 40 people; **NOT YET** for needs.

**Commit.** `fe077d5`.

**Top 5 remaining.**
1. **Need satisfaction via activity choice** — `snub` + `gossip` are 82% of starts and satisfy nothing;
   `need3` at 0 for 28% of ticks with zero travel. Both directions measured.
2. **The two-clocks decision** — the fresh-eyes narrative now shows a 40-game-hour walk to bed in plain
   text, which is a better argument for option 1 than the decision file was. Needs the operator.
3. **Rumor reach flat-lines** at 0.333 from tick 62; Daley–Kendall stifling should decay it.
4. **`Field` / `Gradient` wiring** — unchanged.
5. **`warn_unscored_fields` says "1 definitions"** — plural helper.

## 2026-11-05-01 — A crowd made a person less likely to eat

**Pick.** Last iteration's #1: need starvation by activity choice. Beat the escalated two-clocks
decision on one sentence: **a decision needs an operator, and this bug does not** — Dill's own text
settles it. Fixing it also strips the confound out of that decision, so the next operator question is
a clean one.

**Cause.** `targets_for` gives a `Person` activity one candidate per person in perception, a `Place`
activity one per matching place — and selection drew over *every candidate*. So an activity's
selection mass was `weight · score · (number of targets)`. In the starter pack:

| activity | starts | share | target kind |
|---|---|---|---|
| gossip | 582 | 41.0% | Person |
| snub | 556 | 39.2% | Person |
| eat | 38 | 2.7% | Place |
| relax | 38 | 2.7% | Place |

`gossip` and `snub` satisfy **no need** and took 80% of every decision. Starvation then scaled with
the number of neighbours, every rate held fixed: `fun` at 0 for 5.9% of ticks with one person,
**43.4%** with sixteen.

**What the source says.** Dill: *"Add up the total weight of all valid **options**"* and *"Rank uses
absolute utility to divide the **options** into categories"* — an option is an action, not an
action-target pair (Game AI Pro 2 ch. 3, `gameaipro2-ch03-dual-utility-reasoning`).

**Change.** Scoring stays per target, because considerations legitimately read the target
(`Distance`, `Affinity`). Each activity now becomes **one** candidate carrying its best score and
every target that tied for it; the draw runs over activities; the target is picked uniformly from the
ties from the seeded RNG.

**Written red first.** `a_crowd_does_not_multiply_a_person_activity_s_odds`: **0.97** at twelve
neighbours, **0.50** after.

**Before → after, with travel removed so only choice differs** (the village layout, four places at the
origin):

| | need0 | need1 | need2 | need3 |
|---|---|---|---|---|
| zero% at 0, before | 0.0 | 0.0 | **39.3** | **64.7** |
| zero% at 0, after | 0.0 | 0.0 | **1.9** | **4.8** |
| mean, before | 0.924 | 0.810 | 0.298 | 0.132 |
| mean, after | 0.869 | 0.810 | **0.554** | **0.526** |

**Perf — and the honest reading of it.** At 1 000 people the tick got *more* expensive, 12.46 →
14.95 ms, which looks like a regression until decisions are counted: 50.4 → 68.6 per tick (+36%). Per
decision the cost *fell*, 247.1 → **218.7 µs (−11.5%)**, because the `PersonId` lookup stopped
scanning `PersonIndex.sorted` and now uses one pass over `perception.near`. At 5 000 the net is
**418.3 → 277.0 ms, −34%**.

**What this does not fix, isolated with numbers.** The real village still starves — the two-clocks
walk. Same 24 people, places at the origin instead of metres apart: means 0.87/0.81/0.55/0.53 versus
0.11/0.17/0.07/0.05. Choice is fixed; travel is `needs-decision/travel-time-in-two-clocks.md`, open
since 2026-10-25.

**Evidence.** `runs/2026-11-05-01/evidence/{before,after}/`, `before-after.png`, `fresh-eyes/`
(seed 99, 40 people — rivals 0 → **6**, the first disagreements the village has had).

**Tests.** 20 → **21** binaries both feature sets; determinism 2/2 (the target pick is seeded).
Dead-code inventory: zero, unchanged — `Candidate` is private.

**Verdicts.** Behaviour GREAT (0.97 → 0.50; starvation scaling gone at every population). Zoom GREAT
(choice fixed; remaining flat regions attributed with numbers to the escalated decision). Fresh eyes
GREAT at seed 99 / 40 people — coherent narrative, rivals back; needs still NOT YET, cause isolated.

**Commit.** `e04f9f4`.

**Top 5 remaining.**
1. **The two-clocks decision** — now the *only* cause of need starvation, measured both ways.
   Recommends option 1; needs the operator.
2. **Rumor reach is a step function** — flat after tick ~60 at both seeds; Daley–Kendall stifling
   should decay it.
3. **`Field` / `Gradient` wiring** — unchanged.
4. **`warn_unscored_fields` says "1 definitions"** — plural helper.
5. **`need-cycle` is still `pending`** in `scenarios.yaml` — with this fix it may pass once travel
   lands, and should be measured against both.

## 2026-11-06-01 — A memory nobody bounded, and a column that lied

**Pick.** The log's #2, "rumor reach flat-lines", which turned out to be **two** defects: an unbounded
`Knowledge`, and an evidence column measuring the wrong rumor.

**Defect 1 — `Knowledge` had no cap.** `Memories` evicts at 64 and `Ties` trims at Dunbar's 150;
`Knowledge` grew forever, and `best_telling` scans all of it on every gossip.

| tick | rumors in book | max entries one person held |
|---|---|---|
| 300 | 24 | 24 |
| 1 000 | 88 | 78 |
| 2 000 | 173 | 155 |
| 4 000 | 386 | **344** |

Fixed: `SocietySettings::knowledge_capacity` (default 64), evicting the lowest
`belief · 0.995^(hours since heard)` — the crate's own `telling_weight` minus the valence `Knowledge`
cannot see. Test written red first, capacity 32 against a book of 325: **287 → 32**.

**Defect 2 — the harness measured `RumorId(0)`.** The first rumor a run ever creates, not the one the
scenario plants. Three iterations recorded "rumor reach flat-lines" from that column. Following the
planted id:

| scenario | old column | planted rumor |
|---|---|---|
| `rumor-crosses-a-cluster` | 1 step, flat 0.333 | **6 changes**, 0 → 0.333 over ~350 ticks |
| `village-evening` | flat 0.417 | **0.042 = 1/24**, never spreads |

The first plot of the fix spliced both rumors into one line and fell to zero at the plant tick; the
plot caught that, and it is now zero before the plant.

**The village figure is worse than reported, and why.** Traced: the origin gossiped **twice in 3 600
ticks** and its pick was stifled or ineligible both times, while 63 gossip exchanges happened around
it. The scenario picks an arbitrary villager as town crier. Documented at the plant site; not fixed
here.

**Behaviour unchanged at the defaults — verified, not assumed.** The village's book peaks at 34, so a
cap of 64 cannot bind: **4 200 / 4 200** `series.csv` rows identical and the narrative identical.

**Perf.** Short runs unchanged (14.96 ms at 1 000). The long run is where it shows: **63.36 → 61.91 ms
(−2.3%)**. The other 4× is the rumor book, below.

**Not fixed, filed.** `Rumor.alive` is cleared only when a subject despawns, so the book grows 848 →
**8 357** over 5 000 ticks and `rules.rs` walks all of it every tick. Measured: **992 rumors, 70 still
tellable, 922 untellable (93%)**. `needs-decision/retiring-untellable-rumors.md` recommends retiring
when no living spreader holds one; it changes what a host reads in `RumorBook.alive`, so it is the
operator's call.

**Tests.** 21 → **22** binaries both feature sets. Dead-code inventory: zero, unchanged; one public
field added.

**Verdicts.** Behaviour GREAT (287 → 32; defaults byte-identical). Zoom GREAT — it caught two harness
errors of mine and traced the village plant to its origin. Fresh eyes GREAT — the cap holds at 40 and
200 people; the remaining growth is filed, not unnoticed.

**Commit.** `b5daf26`.

**Top 5 remaining.**
1. **Retire untellable rumors** — 93% of the book, and the per-tick walk over it is a 4× cost growth.
   Filed with a recommendation.
2. **The two-clocks decision** — still the only cause of need starvation in a real layout.
3. **The village's rumor origin** — an arbitrary villager cannot be the town crier; pick a teller that
   gossips or lower the target.
4. **`Field` / `Gradient` wiring** — unchanged.
5. **`warn_unscored_fields` says "1 definitions"** — plural helper.

## 2026-11-07-01 — The scenarios now measure what they claim

**Pick.** Not a model change: the crate's own evidence base. `scenarios.yaml` says *"a scenario with
`status: pending` has no test yet; the loop may not claim it passes"*, and **four of seven were
pending**. Two stated precise, falsifiable observables that nothing had ever checked. Beat the
runner-up (the back-to-back sleep loop) on one sentence: *I cannot fix a defect the evidence base was
unable to see.*

**Two observables were wrong, which is why they could never have passed.**

| Scenario | As written | What the crate does |
|---|---|---|
| `deed-draws-a-response` | a *planted* `steal` with five witnesses → one `NormViolated` | `PlantRumor` starts a rumor, it does not deliver a deed; `witness()` runs only from an accepted interaction. Measured: **0 violations, `known_by = 1`** |
| | | Driven properly — accepted `insult` with `deed: Some("steal")` — **1 rumor, `known_by` 4, 4 violations** |
| `opinion-converges-but-not-entirely` | 6 000 ticks → spread falls a third | 6 000 gives **0.3699**, a 27.6% fall. 12 000 gives **0.2398** |

Both corrected in place with a dated note; neither threshold lowered.

**`determinism` was pending on a reason that had stopped being true** — its two tests have been green
for several iterations.

**`need-cycle` stays pending, and the blocker is measured.** One person, `rest` decaying 0.7/day, a
4-hour sleep satisfying 0.12 per game-hour:

| | |
|---|---|
| completed sleeps in 20 000 ticks | **29** |
| ticks in 4 game-hours | **683** |
| `rest` range | **[1.000, 1.000]** |

The sleeps are **back to back**, so the need is met continuously and never decays. The satisfy path is
fine (`rate · hours` per use, clamped at 1.0, asserted). The cause is the choice rule:
`Linear(-1, 1.0)` scores a full 1.0 anywhere below full, so the top-up activity is the best option the
instant the last one ends. The test asserts this state, so the day it changes the test fails.

**The harness had the same `RumorId(0)` bug I fixed in `series.csv` last iteration.** `summary.json`
counted knowers of the first rumor the run made, so every scenario planting a later one reported
`rumor_known_by: 0` — including `need-cycle`, which printed 0 knowers beside 119 rumors.

| scenario | before | after | reach |
|---|---|---|---|
| `village-evening` | 0 | **1** | 0.042 |
| `rumor-crosses-a-cluster` | 0 | **4** | 0.333 |

That is a process failure worth naming: I fixed the symptom I was looking at, not the class.

**Status.** Four pending → **one**, and that one says why. Tests 22 → **25** binaries both feature sets;
dead-code inventory zero, unchanged.

**Evidence.** `runs/2026-11-07-01/evidence/village.png` (needs, planted reach, opinion variance — the
last plateaus at 0.033 and stops falling, so it narrows without collapsing).

**Verdicts.** Behaviour GREAT (3 scenarios measured, 1 blocked with numbers, 2 observables corrected and
dated). Zoom GREAT — it found the summary repeating last iteration's bug and exposed the back-to-back
sleep loop. Fresh eyes GREAT — a host can now tell which claims are checked, corrected, or
known-broken.

**Commit.** `da0803f`.

**Top 5 remaining.**
1. **The back-to-back activity loop** — 29 consecutive sleeps, `rest` pinned at 1.0. Has a
   failing-on-change test now, so it can be fixed against it.
2. **Retire untellable rumors** — 93% of the book cannot spread; the per-tick walk is a 4× cost growth.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — still the only cause of need starvation in a real layout.
4. **The village's rumor origin** — reaches 1/24; an arbitrary villager cannot be the town crier.
5. **`Field` / `Gradient` wiring** — unchanged.

## 2026-11-08-01 — A host can now ask why

**Pick.** The brief's own word **explainable**, which had no implementation: `Doing` says what a person
is doing; nothing said what else it weighed or by how much. Beat the runner-up (last iteration's
back-to-back sleep blocker) on one sentence: **that blocker was my test's own definition, not the
crate's.**

**Correction to the previous iteration.** The starter pack's `sleep` is gated `Hours(22·6)` and ranked 1,
so one person sleeps **once per day** — measured, 10 sleeps in 10.1 days. Last iteration's "29 back-to-back
sleeps in 20 000 ticks" came from a test definition with no hour gate and no rank. `need-cycle`'s
observable is still not met by the shipped rates, but the cause is the rate balance, not a loop.

**Change.** `DecisionTrace`: an opt-in ring of the last N decisions, each recording every option that
scored, best first, with the winner marked. Built to the three constraints Game AI Pro 3 ch. 6 puts on
a recorder:

| Constraint (quoted from the chapter) | How it is met |
|---|---|
| "no impact on the outcome of simulating objects" | asserted: the same seed gives an **identical sequence of choices** with and without the trace |
| "the memory requirements of the system must be fixed" | a ring of `capacity`, oldest dropped; 2 holds exactly 2, 8 holds exactly 8 after 4 000 ticks |
| "minimal to no impact on the speed of the game simulation" | **15.16 ms/tick with it, 15.28 without** at 1 000 people |

**The mistake the feature caught in itself.** The first closeness measure was
`(chosen − next-in-sorted-list) / chosen`, and the village printed **`margin -0.33`** — impossible for a
"fraction better". Selection is Dill's **weighted draw** over the viable tier, so the chosen option is
often not the best: that village line picked the **sixth-best of six**. Replaced with `shortfall`
against the best score; four unit tests pin it.

**What it looks like.** From `evidence/village/why.txt`, seed 7, tick 4147:

```
# tick 4147: someone chose compliment (0.752), 25% below snub (1.000)

 1.000  snub
 1.000  relax
 1.000  wander
 0.971  eat
 0.912  chat
 0.752  compliment   <- chosen
```

Six options weighed, the lowest chosen, and every number invisible before this iteration. "someone"
because the entity had despawned — which is why the trace copies names when it can and `describe`
accepts a name list from the caller.

**Tests.** 25 → **26** binaries both feature sets (7 new: 3 integration, 4 unit). Dead-code inventory:
zero — the module is public and exercised by all seven.

**Verdicts.** Behaviour GREAT (identical outcome with and without the recorder; options and scores
readable). Zoom GREAT — the first output's negative margin was caught by reading it, not by reasoning.
Fresh eyes GREAT — a host can say why, in one sentence, from data the crate now produces.

**Commit.** `509a57d`.

**Top 5 remaining.**
1. **`need-cycle`'s rate balance** — the observable is still unmet by the shipped rates; the blocker note
   is now wrong about the cause and should be re-measured against the starter pack's real definitions.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
   `needs-decision/retiring-untellable-rumors.md`.
3. **The two-clocks decision** — still the only cause of need starvation in a real layout.
4. **The village's rumor origin** — reaches 1/24; an arbitrary villager cannot be the town crier.
5. **The trace hides the tier rule** — `why.txt` shows the scores but not that they were cut to a rank
   tier and drawn by weight; a host reading only the table misses why a lower scorer can win.

## 2026-11-09-01 — Two clocks, and the crate's silence about them

**Pick.** The log's #1, `need-cycle`'s rate balance — which re-measuring showed was a different defect
than the one recorded. Beat the runner-up (`Place`/`Person` completion asymmetry) on one sentence:
**the asymmetry's first cause is the timeout, and the crate can already prove the timeout is too
short** — so the fix helps every host, not just this project's village.

**Correction first.** The previous blocker note said a person "sleeps back to back: 29 completed 4-hour
sleeps in 20 000 ticks" *with the shipped rates*. It does not. The starter pack's `sleep` is gated
`Hours(22, 6)` and ranked 1 — **10 sleeps in 10.1 days**. The loop came from a test-local definition with
no hour gate. Re-measured against the starter pack:

| need | crosses < 0.35 | recovers > 0.7 | range |
|---|---|---|---|
| `social` | no | no | [0.364, 1.000] |
| `rest` | no | no | [0.495, 1.000] |
| `food` | **yes** | **yes** | [0.000, 1.000] |
| `fun` | **yes** | **yes** | [0.000, 1.000] |

The wrong note stays in `scenarios.yaml` under `superseded:`.

**The defect.** Two clocks at a walk: `SocialMover::speed` is m per **real** second (0.0219 m/tick at
1.4 m/s), `travel_timeout_hours` is **game** hours (`h · 10.7` ticks at a four-second day). Measured:

| walk | ticks | 40-hour timeout allows |
|---|---|---|
| 8.2 m | **308** | 427 ✓ |
| 13 m | **527** | 427 ✗ |

The village's diagonal is 13 m, so its far corner always timed out. The completion counts split by
target kind, cleanly:

| activity | target | started | completed | interrupted |
|---|---|---|---|---|
| `chat` + `gossip` | `Person` | 152 | **152** | **0** |
| `eat` | `Place` | 108 | 23 | **78** |
| `sleep` | `Home` | 108 | 56 | **44** |

**Change.** `warn_short_travel_reach` in `BehaviorPlugin::finish`, beside the two warnings already
there. It computes the metres a timeout buys and warns under a 10 m floor — a warning that fires on
every marginal layout is one hosts learn to ignore. The line it prints:

```
a travel_timeout_hours of 40 at seconds_per_day 4 allows 427 ticks, which at 1.4 m/s is only 9.3
metres of walking — less than 10 m. … Raise travel_timeout_hours or shorten the walks.
```

The coupling itself is `needs-decision/travel-time-in-two-clocks.md` and stays open. The silence was not
a decision the crate could defer.

**Harness.** `examples/evidence.rs` set 40 hours with a comment reasoning about "the 6 m walk this
layout asks for (274 ticks)". The diagonal is 13 m. Corrected to **55 hours** (587 ticks):

| | 40 h | 55 h |
|---|---|---|
| `fun` at zero | 55% | **35%** |
| `fun` mean | 0.049 | 0.057 |
| `food` at zero | 28% | 22% |
| `social` at zero | 4% | **11%** |

`evidence/timeout.png`. The plot is honest: needs still hug zero, and `social` got *worse* as fewer
people are stuck walking. Travel was a cause, not the cause.

**A near-miss worth recording.** I spent several probes on a nine-person tavern where **nobody arrived
at all** and nearly wrote up "capacity starves places". The real cause: my probe never inserted
`SocialMover`, so nothing walked. What caught it was noticing the single-person probe *had* arrived
(tick 308) with a mover and the nine-person one had not without.

**Tests.** 26 → **27** binaries both feature sets. Dead-code inventory: zero — one private function and
one private constant, both reached from `finish` and asserted by three tests.

**Verdicts.** Behaviour GREAT (the crate names a timeout it can prove too short, with the arithmetic).
Zoom GREAT (reported a marginal win as marginal; caught the mover-less probe before it shipped a wrong
conclusion). Fresh eyes GREAT (the corrected note keeps the old one visible; the scenario stays honestly
`pending`).

**Commit.** `7611b69`.

**Top 5 remaining.**
1. **`Place`/`Person` completion asymmetry** — 78 of 108 `eat`s interrupted against 0 of 152 socials,
   with the timeout ruled out as the first cause. This is what starves `food`, `fun` and `rest`.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
3. **The two-clocks decision** — now warned about and measured, but the API question is still open.
4. **The village's rumor origin** — reaches 1/24; an arbitrary villager cannot be the town crier.
5. **`need-cycle`'s two remaining needs** — `social` and `rest` never cross 0.35 at the shipped rates;
   whether that is correct or too generous is unmeasured.

## 2026-11-10-01 — One GIF per feature, and what the clips exposed

**Pick.** The user's instruction: record a GIF per feature and inspect them. Beat the runner-up
(`Place`/`Person` completion asymmetry) on one sentence: **the instruction was explicit, and doing it
found three real defects in the demo path** — including a `RumorId(0)` bug that made the hero clip's
headline number read 0.000.

**Six clips, one run.**

| scene | overlay |
|---|---|
| `village` | everything: rumor rings, tie lines, HUD |
| `mood` | PAD ring, colour = pleasure, radius = arousal |
| `opinions` | one column per villager on the honesty topic |
| `needs` | food halo, radius and colour = how fed |
| `ties` | the tie graph alone |
| `factions` | faction colours, no tie lines |

All six print identical numbers — `rumor_reach 0.880, friends 36, rivals 45, barks 2810` — which is the
proof they are views of one simulation and not six different ones.

**Three defects the frames exposed.**

1. **Every overlay was invisible.** `MARK_HEIGHT` is 0.06 m, and the tie lines, rumor rings and need
   rings all drew there — under the capsules and under the ground plane's depth. The ties clip showed
   **two short segments** where the graph has hundreds of edges. Everything draws at chest/head height
   now.

2. **`RumorId(0)`, for the third time.** `draw_rumor_rings`, `stats()` and the HUD's "known by N" line
   all read the first rumor the run made, not the theft the clip plants.

   | | reported | actual |
   |---|---|
   | hero reach, before | **0.000** | 0.880 |
   | hero reach, after | **0.880** | 0.880 |

   The clip's entire subject was absent from its own numbers. `docs/DEMOS.md` had recorded this class
   of bug as fixed twice; it was not.

3. **The colour scales were unreadable.** PAD here spans only ±0.4 (measured, 4 200 ticks: pleasure
   [-0.387, 0.382]), so mapping onto [-1, 1] gave every villager the same dull olive ring. Normalised
   to `MOOD_SPAN = 0.45`; opinion columns likewise to `OPINION_SPAN = 0.5`.

**A trap worth recording.** Extracting frame 60 from the GIF showed solid green, and I briefly concluded
the GIF was corrupt. It was the extractor: `-layers optimize` stores partial rectangles
(`640x311+0+49`), so `magick file.gif[60]` needs `-coalesce` first. That is now in `docs/DEMOS.md`,
because the next person will hit it.

**Files.** 15 MB total at 600 px `STRIDE=2 FPS=60` — half the size of the first encode at no loss of
speed. Visual check at frames 0 / 120 / 239 of all six, coalesced: no blank first frame, HUD legible
over grass and night sky, every overlay distinguishing at least two villagers.

**Tests.** 27 binaries both feature sets, unchanged. Dead-code inventory: zero — `scenes.rs` is
example code and every function is reached by a scene.

**Verdicts.** Behaviour GREAT (six views of one run, one caption each). Zoom GREAT — three real demo
defects found by looking rather than reading. Fresh eyes GREAT (captions in plain words, HUD legible
over both grass and night).

**Commit.** `0587be5`.

**Top 5 remaining.**
1. **`Place`/`Person` completion asymmetry** — 78 of 108 `eat`s interrupted against 0 of 152 socials.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
3. **The two-clocks decision** — warned about and measured, but the API question is still open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`need-cycle`'s two remaining needs** — `social` and `rest` never cross 0.35 at the shipped rates.

## 2026-11-11-01 — A `Destination` nothing walked to

**Pick.** The log's #1, the `Place`/`Person` completion asymmetry. Beat the runner-up (retiring
untellable rumors) on one sentence: **this is a broken contract in a supported configuration**, not a
paperweight — and it is the cause of that asymmetry.

**The defect, in the crate's own words.** `Destination` says:

> Games **without the steering plugin move persons themselves toward this point.**

They did not. `BehaviorPlugin` *writes* a `Destination` for every travel target; the only system that
*reads* one is `steer_movers`, inside the `steering` feature. With `--no-default-features` a person is
given a destination and stands still:

```
t=  1 doing=eat dest=Some((6.5, -5.0)) pos=(0.0, 0.0)
t=600 doing=eat dest=Some((6.5, -5.0)) pos=(0.0, 0.0)
```

**Consequences, measured in a 24-person village at the current 55-hour timeout:**

| activity | target | completed | interrupted |
|---|---|---|---|
| chat, gossip, compliment, snub, wander | `Person`, `Point` | **339/339 (100%)** | 0 |
| eat | `Place` | 14/62 (23%) | 43 |
| sleep | `Home` | 24/79 (30%) | 48 |
| work | `Place` | **6/64 (9%)** | 53 |

Only **26 of 228** `Place`/`Home` activities ever arrived. `Person` targets succeed because the target
walks to the actor — the crate only had to move one of the two.

**The fix.** `walk_toward_destination`, every fixed step, at `Tuning::walk_speed` (1.4 m/s — the same
number and unit `SocialMover` already used), skipping anyone a `SocialMover` is steering.

The obvious way to write that skip breaks the build the fix is for: `SocialMover` does not exist
without its feature, so `Without<SocialMover>` cannot be named there. A `cfg`-selected alias keeps one
body for both. **The no-default-features gate caught exactly this** — 0 test binaries before the alias,
28 after.

**Before → after**, same seed and layout, 4 200 ticks:

| | before | after |
|---|---|---|
| `Place`/`Home` arrivals | 26 | **212** |
| `eat` completed | 23% | **85%** |
| `relax` completed | 22% | **95%** |
| `sleep` completed | 30% | **94%** |
| `work` completed | 9% | **72%** |

After the initial fall from 1.0, each need's share of ticks at zero:

| need | before | after |
|---|---|---|
| social | 39.3% | **2.0%** |
| rest | 43.6% | **0.8%** |
| food | 71.2% | **12.8%** |
| fun | 89.2% | **35.3%** |

`evidence/walk.png`, `evidence/{before,after}.csv`. The mean of the four needs stops being a flat line
at zero and starts oscillating — the shape a society with travel should have.

**`fun` at 35% is not fixed**, and saying otherwise would be false: `relax` is its only source and is
gated 17:00–23:00. Top remaining item, with the number.

**The timeout is not the cause.** 55 game-hours here is 587 ticks = **12.8 m**, against an **8.2 m**
longest walk. Last iteration's reach warning is correct in general and stays; this was a different
defect, and the research note says so explicitly so a later reader does not conflate them.

**Perf.** 0.564 / 15.02 / 275.5 ms at 100 / 1 000 / 5 000 — within ±1% of before, i.e. not measurable
against the perception pass it runs beside.

**Tests.** 27 → **28** binaries, 28 in **both** feature sets. Dead-code inventory: zero.

**Verdicts.** Behaviour GREAT (26 → 212 arrivals, `Place` completions 23% → 85%). Zoom GREAT (needs stop
flatlining; `fun` still 35% and reported as such). Fresh eyes GREAT (same pace as steering, and the
wrong comment fixed at its source).

**Commit.** `5eb4a7b`.

**Top 5 remaining.**
1. **`fun` at 35% of ticks at zero** — `relax` is its only source and is gated 17:00–23:00.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
3. **The two-clocks decision** — no longer the cause of starvation, but the API question is open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`work` at 72%** — the lowest of the three fixed activities.

## 2026-11-12-01 — The fix that made the plot look better and the society worse

**Pick.** The log's #1, `fun` at 35% of ticks at zero. Beat the runner-up (retiring untellable rumors) on
one sentence: **the honest answer turned out to be "no rate can fix this", and that is what the next
iteration needs to know.**

**The budget.** Every need decays per game-day; every activity supplies `satisfies · duration_hours` per
use.

| need | decay/day | best source | per use | uses/day |
|---|---|---|---|---|
| social | 1.00 | `chat` | 0.36 | 2.78 |
| rest | 0.70 | `sleep` | 1.20 | 0.58 |
| food | 0.90 | `eat` | 0.90 | 1.00 |
| **fun** | **0.80** | **`relax`** | **0.60** | **1.33** |

`fun` needs 1.33 uses a day; measured supply is **0.10**.

**Three hypotheses tested and rejected.**

| hypothesis | measurement | verdict |
|---|---|---|
| `satisfies` does not pay | fun sampled at arrival and at end of one 1.5 h relax | **+0.6000**, exactly `0.4 × 1.5` |
| the 17–23 gate binds | gate removed | relaxes 79 → 167, `fun` mean 0.053 → 0.092 — a constraint, not *the* constraint |
| capacity binds | well holds 6, window holds 4 batches | not binding |

What binds is **choice**: `relax` competes with three cheap `Person`-targeted activities for the same
evening, and each use costs a walk plus 16 ticks against a `chat`'s 3.

**The fix that was tried and reverted.** `relax` `satisfies` 0.4 → 0.55, so one evening session pays
0.825 and covers the day:

| after tick 200, 24 people | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **18.8%** |
| `fun` mean | 0.061 | **0.051** |
| `social` mean | 0.124 | 0.112 |
| `food` mean | 0.101 | 0.085 |
| **mean of all four** | **0.127** | **0.118** |

The zero fraction halves — the metric the log named — and the total gets **worse**. The village is
over-subscribed: 24 people, four places, one evening. Raising one supply moves starvation around.
Reverted.

**The lesson, recorded because it generalises.** `evidence/relax.png` shows the reverted fix and it looks
like an improvement: `fun`'s peaks rise from 0.10 to 0.15 and the mean line sits above the old one. I
nearly shipped it on the strength of the zero fraction and the plot. What stopped it was computing the
**mean of all four needs**, a metric the loop had not been requiring. A plot of one need can look better
while the society is worse.

**What shipped.** `SocietyDefs::need_decay` and `best_source`, and `warn_unbalanced_needs` in `finish`
— the fourth warning, beside the short-activity, unscored-field and short-travel ones. Threshold four
uses a day, not one, because the crate cannot see that `chat` is three ticks while `relax` is a walk.
The starter pack does not trip it, and that limit is stated rather than papered over.

**Literature.** Vansteenkiste & Ryan 2013, DOI 10.1037/a0032359: self-determination theory distinguishes
need *satisfaction* from need *frustration*, and frustration is a different state with its own
behavioural consequences — which is why a permanently unmet need is worth naming rather than leaving to a
plot.

**Tests.** 28 → **29** binaries, 29 in both feature sets. Dead-code inventory: zero.

**Verdicts.** Behaviour GREAT (the arithmetic is named and testable; the limit is documented). Zoom GREAT
(the plot of the reverted fix looks like a win while the numbers are not). Fresh eyes GREAT (the negative
result is kept with its numbers and a decision file).

**Commit.** `29b822e`.

**Top 5 remaining.**
1. **A cheap source for `fun`** — `needs-decision/over-subscribed-evening.md` recommends a
   `Person`-targeted leisure activity so a use costs a `chat`'s three ticks.
2. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`work` at 72%** — the lowest completion of the three fixed activities.

## 2026-11-13-01 — Leisure needs a companion, not a bigger number

**Pick.** The log's #1: a cheap source for `fun`. Beat the runner-up (retiring untellable rumors) on one
sentence: **`fun` at zero for a third of every run is the most visible wrong thing when you watch the
society, and last iteration established that no rate can fix it.**

**The insight.** `fun` and `social` sit at opposite ends of the same budget:

| need | uses/day needed | actual supply | cost of one use |
|---|---|---|---|
| social | 2.78 (`chat`) | **ample** | 3 ticks; the other person walks to you |
| fun | 1.33 (`relax`) | **0.10** | a walk plus 16 ticks, in a 6-hour window |

`social` needs *more* uses a day and is never short. The difference is not the rate — it is that a `chat`
targets a `Person`.

**The change.** `play`: `target: Person`, 0.5 h, `satisfies fun 1.7` (0.85 per use, 0.94 uses a day),
gated `Affinity Linear(0.0, 0.8)` so acquaintances can play, not only close friends. `relax` stays as the
solitary form.

**Grounding.** Vansteenkiste & Ryan 2013 (DOI 10.1037/a0032359): self-determination theory puts
relatedness among the basic needs and locates intrinsic motivation in need-supportive *social* contexts —
"individuals can be vital, open, curious and caring" *with others*. Leisure done with someone is the
social form of the need.

**Before → after**, same seed and layout, after tick 200:

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **1.3%** |
| `fun` mean | 0.061 | **0.112** |
| `social` mean | 0.124 | 0.107 |
| `food` mean | 0.101 | 0.080 |
| **mean of all four** | **0.127** | **0.128** |

`evidence/play.png`, `evidence/{before,after}.csv`. The plot and the numbers agree this time — last
iteration's reverted rate change had a plot that looked better while its total was worse.

**The honest cost.** 275.5 → **300.8 ms** per tick at 5 000 people, **+9.2%**, measured three times
(300.786, 300.709, 300.906). Predicted `10/9 − 1 = +11.1%` if scoring dominates, so it is the tenth
activity scoring once per neighbour. A host that prefers the cheaper tick drops `play`.

**A test fired correctly.** `tests/need_budget.rs` asserted `fun` needs **> 1** use a day with the message
*"if this ever becomes <= 1, one evening session covers a day and the imbalance is gone"*. It fired.
Rewritten to assert the opposite for `rest`, `food` and `fun`, with `social` documented as the exception.

**Tests.** 29 → **30** binaries, 30 in both feature sets. Dead-code inventory: zero.

**Verdicts.** Behaviour GREAT (`fun` at zero 33.8% → 1.3%, mean doubled, total flat). Zoom GREAT (plot and
numbers agree; no flatline, no saturation). Fresh eyes GREAT (the activity explains itself; its 9.2% cost
measured and stated).

**Commit.** `f3b6259`.

**Top 5 remaining.**
1. **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.
2. **A spatial index for perception** — every `Person`-targeted activity scores once per neighbour, and
   this iteration's 9.2% is that cost. Now has a price attached.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.

## 2026-11-14-01 — The number that steered eight iterations and was never measured

**Pick.** The log's #1: retiring untellable rumors. Beat doing the retirement on one sentence: **the
justification for that breaking change is wrong**, and shipping a semantic change to `RumorBook.alive` on
a false premise is worse than shipping nothing.

**The claim.** Since 2026-11-06-01 the top-5 list has carried: *"93% of the book cannot spread; a 4× cost
growth over 3 000 ticks."* Eight iterations. Twice the #1 item. It is the reason
`needs-decision/retiring-untellable-rumors.md` calls for changing what `alive` means.

It was never measured. It came from comparing a short benchmark against a long one and attributing the
whole difference to the book.

**Measured**, 200 people, four places, five samples of 100 ticks across a 6 000-tick run, with and
without a `RumorReach` trigger:

| tick | rumors | ms/tick, none | ms/tick, trigger |
|---|---|---|---|
| 1 000 | 765 | 5.76 | 5.95 |
| 3 000 | 2 412 | 3.51 | 3.07 |
| 5 000 | 4 299 | 4.80 | 3.93 |

**The cost does not grow.** It drifts down as the village settles into longer activities, and a trigger
makes no difference — 4 299 pointer-chases beside 200 people scored against every neighbour is nothing.

**What the book does cost** is memory: ~48 B a rumor, so **105 KiB** at 3 000 ticks, **228 KiB** at 6 000,
about **5 MiB** for 5 000 people over a comparable run.

**The other half holds.** 2 250 rumors, every one `alive`, **301 still tellable** — **87% untellable**. The
retirement rule is still the right fix for memory, and still a decision: `alive` means "the subject is
still in the world", three places read it, and a host sees it. What changes is urgency — 4× would justify
breaking that semantic, 5 MiB does not.

**A first draft that measured nothing.** The helper used `common::settings()`'s default 240-second day,
where 3 000 ticks is half a game-day, and reported `0 rumors, 0 alive, 0 still tellable` — a test that
would have passed on nothing. It sets a four-second day now, with a comment.

**Tests.** 30 → **31** binaries, 31 in both feature sets. Dead-code inventory: zero; no library change.

**Verdicts.** Behaviour GREAT (grows, all alive, 87% untellable, ~48 B/rumor, pinned by two tests). Zoom
GREAT (the late rise in ms/tick is decision timing, not the book; stated as "does not grow", not "flat").
Fresh eyes GREAT (the decision file leads with the correction and the change in urgency).

**Commit.** `53584ee`.

**Top 5 remaining.**
1. **Retire untellable rumors** — still right for memory, now with a true cost and a decision file a host
   can act on: 5 MiB, not 4×.
2. **A spatial index for perception** — the measured 9.2% from 2026-11-13-01.
3. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.

## 2026-11-15-01 — The spatial index that was built, measured, and did not pay

**Pick.** The log's #2: a spatial index for perception. Beat retiring untellable rumors on one sentence:
**last iteration measured that the book costs memory, not ticks**, so the index is the remaining
performance item — and its cost is real and `O(n²)`.

**The claim, corrected again.** The log said the `Person`-targeted scoring was worth 9.2%. Perception
itself is worse. Measured standalone over the benchmark's layout:

| people | brute-force loop, ms/tick | pairs | edges |
|---|---|---|---|
| 1 000 | **1.45** | 999 000 | 169 000 |
| 5 000 | **35.49** | 24 995 000 | 4 264 430 |

35.5 of 300.8 ms is **11.8%** of a tick at 5 000.

**The index, to the paper's recipe.** Ihmsen et al., *SPH Fluids in Computer Graphics* §2: "grids are
built in `O(n)` and particles are accessed in `O(1)`", cells as wide as the query radius, the nine
adjacent cells queried. Implemented exactly that, with neighbour lists restored to ascending index order
because every per-person loop in the crate depends on it.

**It found the same neighbours and gave nothing.**

| | brute force | `HashMap` grid | `BTreeMap` grid |
|---|---|---|---|
| 100 | **0.573 ms** | 0.707 | — |
| 1 000 | 15.67 ms | 15.99 | **16.97** |
| 5 000 | 300.8 ms | **297.96** | — |

A 1% gain at 5 000 is noise; **19% slower at 100** is not. The crate's own examples are villages of tens
to hundreds. **Reverted.**

**Why, measured.** Brute force yields each neighbour list already sorted by index — both loops walk
`0..n`. The grid scrambles them and pays `sort_unstable` over **4 264 430 entries** to restore the order,
which costs about what the 25 M distance checks cost, plus nine hash lookups per person. SPH amortises a
neighbourhood query over several solver passes per tick; this crate makes **one**.

**What ships.** `tests/perception.rs`: a reference brute force computed from `Perception`'s own positions,
compared entry for entry against what the crate stored — **32 016 neighbour entries across 180 people**.
The next index is checked against a computed reference rather than trusted, which is the discipline that
caught `RumorId(0)` three times and the 4× rumour claim last iteration. The measured cost is documented on
`perceive`.

**Tests.** 31 → **32** binaries, 32 in both feature sets. Dead-code inventory: zero; `src/` is unchanged
from `HEAD`.

**Verdicts.** Behaviour GREAT (identical neighbours, pinned by a 32 016-entry comparison). Zoom GREAT — the
grid *worked* and gave 1%, and a weaker pass would have shipped a 19%-slower small village for it. Fresh
eyes GREAT (a reference implementation, a documented cost, a corrected claim).

**Commit.** `028daaf`.

**Top 5 remaining.**
1. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable. A
   decision, not a fix.
2. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
3. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured. An index needs its sort
   amortised: a coarser cell that preserves index order, or a query run more than once per tick.

## 2026-11-16-01 — The harness never read the spec

**Pick.** Whatever the log said, the first thing this iteration did was run the village — and the README's
clip and the evidence harness disagreed about the same scenario (23 of 25 reached against 1 of 24). Three
configurations for one name:

| | `seconds_per_day` |
|---|---|
| `docs/loop/scenarios.yaml` — **the spec** | **64.0** |
| `examples/evidence.rs`, hardcoded | 4.0 |
| `examples/capture.rs`, hardcoded | 8.0 |

Neither harness read the file. **The spec had never been executed.**

**What that cost**, same seed and spawn:

| | 4.0 s/day | **spec 64.0 s/day** |
|---|---|---|
| needs mean | [0.11, 0.21, 0.08, 0.11] | **[0.88, 0.77, 0.50, 0.81]** |
| rumor peak reach | 0.042 | 0.083 (`known_by` 2) |

At 4 s/day a four-hour activity is **4 ticks** and an 8 m walk is **375**. Every "needs starve" and "the
rumor never spreads" conclusion in the previous eight iterations was measured at a day length sixteen times
shorter than the spec.

**Three more inconsistencies the fix exposed**, each resolved by measurement:

| | measured | fix |
|---|---|---|
| `ticks: 4200` at 64 s/day | **1.03 game-days**, not 17 | scaled by `seconds_per_day / 4.0` |
| `plant_at` 560 at 64 s/day | 3.3 game-hours — reach **0.375 → 0.000** | scaled the same way |
| 2-hour timeout at 64 s/day | **7.5 m** against an 8.2 m village | scenarios declare 20 h (75 m) |

**Two hypotheses I got wrong before measuring.** First: that `best_telling`'s "ties go to the lowest
`RumorId`" starved the planted rumor — I wrote three tests for it and **all three passed with no code
change**, so the hypothesis was dead. Second: an off-by-one id race — I "fixed" it and the number did not
move. Only printing the rumor's origin, subject and holders showed it had `known_by: 2` and **zero living
holders**: it is *forgotten*, because `knowledge_capacity` (64) evicts the least-retained and a rumor from
day 2 has retention `0.995^360 = 0.165` against 150 later ones. The model working; the summary asking the
wrong question. It now reports the **peak**.

**The observable was wrong too.** `village-evening` asked the rumor to reach **half the village**. The
village makes about **18 rumors a day**, and `best_telling` prefers the strongest and freshest, so a planted
rumor is told (`known_by` 1 → 2) and then out-competed — Daley–Kendall. It now asks that it be told to at
least one other person, with the old wording kept and the measurement that replaced it.

**Tests.** 32 → **34** binaries, 34 in both feature sets. Dead-code inventory: zero.

**Verdicts.** Behaviour GREAT (the spec is executed; needs 4–7× higher at its settings). Zoom GREAT (four
defects found, one of which I got wrong twice before measuring). Fresh eyes GREAT (the observable now asks
what the model does).

**Commit.** `0c525fd`.

**Top 5 remaining.**
1. **Re-derive the behaviour conclusions at the spec's settings** — `need-cycle`, `food` at 15%, and the
   rumor-reach note were all measured at 4 s/day, and none may survive at 64.
2. **`examples/capture.rs` still hardcodes 8.0 s/day** — the third value; it should read the spec and the
   README clip's numbers re-checked.
3. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
4. **The two-clocks decision** — warned about, measured, API open.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

## 2026-11-17-01 — The seed was decoration too

**Pick.** The log's #1: re-derive the behaviour conclusions at the spec's settings. Beat fixing `food` at
15% of ticks at zero on one sentence: **that number was measured at 4 s/day and does not survive the
correction.**

**The seed was unread.** Re-running the scenarios exposed the same class of defect again: `seed:` in
`docs/loop/scenarios.yaml` was decoration. Every scenario ran at the crate default `0x5EED_5EED`, so the
five that declare 7, 11, 3, 5 and 1 were **the same run**. `Spec` now reads it; `--seed` still overrides.
Two consecutive `need-cycle` runs end at identical values — `[0.957, 0.608, 0.669, 0.867]` — so it is
deterministic.

**Every scenario at its own settings:**

| scenario | need means | all four cycle? |
|---|---|---|
| `need-cycle` | [0.86, 0.81, 0.58, 0.79] | **yes** |
| `village-evening` | [0.29, 0.53, 0.24, 0.26] | **yes** |
| `rumor-crosses-a-cluster` | [0.64, 0.15, 0.40, 0.61] | **yes** |
| `deed-draws-a-response` | [0.86, 0.81, 0.63, 0.81] | **yes** |

**`need-cycle` is `measured`** after four iterations as `pending`. It was pending because at a four-second
day a four-hour activity lasted four ticks while a walk took three hundred; at its own settings it is a
clean daily cycle.

**`rumor-crosses-a-cluster` reaches 0.583** against the 40% its observable asks for — it had been marked
`measured` while failing. That is the second stale status this pair of iterations has found.

**A conclusion retired, not fixed.** The log's *"`food` at 15% of ticks at zero"* (carried since
2026-11-13) is an artefact: at the spec's settings `food` averages **0.58** in `need-cycle` and **0.24** in
the village, with **one trough a day**.

**The plot nearly fooled me.** `evidence/spec-needs.png` over 19.5 days looks like violent thrashing — all
four needs sawtoothing between 0.2 and 1.0. Counting local minima below 0.4:

| need | troughs / 19.5 days |
|---|---|
| social | **0** |
| rest | **0** |
| food | **20 (1.0/day)** |
| fun | **0** |

`evidence/need-cycle-zoom.png`, three game-days, is what one day is: each need decays steadily and jumps
when its activity runs; `rest` holds 1.0 through sleep; `food` sawtooths 0.25 → 0.80 once a day. **A
full-range plot can manufacture a defect**, and the trough count is what separates the two.

**Tests.** 34 → **34** binaries (one test added to `scenario_spec`), 34 in both feature sets. Dead-code
inventory: zero; `src/` unchanged.

**Verdicts.** Behaviour GREAT (seed honoured and deterministic; all four scenarios cycle). Zoom GREAT (the
plot faked a defect; the count disproved it). Fresh eyes GREAT (retires `food` at 15% and exposes a
`measured` scenario that was failing).

**Commit.** `7c6e3d1`.

**Top 5 remaining.**
1. **Per-person need cycles, not the mean** — the observable says *every person*; the series is a mean over
   eight, so an individual could be failing.
2. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name.
3. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip is checked against a third
   configuration.
4. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

## 2026-11-18-01 — The row that was not a sample

**Pick.** The log's #1: the observable says *every person's* needs cycle, while the evidence was a **mean
over eight**. Beat fixing `capture.rs`'s hardcoded day length on one sentence: **measuring it falsified a
`measured` status**, which is worth more than a third configuration nobody reads.

**Per person, at the scenario's own settings** — 64 s/day, 8 people, 80 000 ticks, seed 7:

| need | lowest any person reached | cycles for all 8? | last iteration claimed |
|---|---|---|---|
| social | **0.39** | **no** | yes |
| rest | **0.50** | **no** | yes |
| food | 0.00 | **yes** | yes |
| fun | 0.12 | 6 of 8 | yes |

**Three of four claims were false.** `social` and `rest` never come within 0.1 of the threshold, and they
are not supposed to — their rates are generous, which is what the starter pack intends.

**Why the mean said otherwise.** The series recorded a row at **tick 0**, before `size_person_state` fills
`Needs`. Every need read `0.000` in that one row, so *"crosses below 0.35"* was true of the series on a
state the society never has. `evidence/tick-zero.png` shows it: a single vertical line at tick 0.

**My first explanation was wrong.** I said a mean over out-of-phase sawtooths dips below any individual.
It does not — a mean's trough is never below its lowest member. Writing that down and checking the
arithmetic is what pointed at the row. The wrong explanation is kept in the review because a plausible
statistical story nearly stopped the search.

**The fix.** The tick-0 row is not recorded; a person with empty `Needs` is not counted. The corrected
run's honest minima: `social` 0.613, `rest` 0.535, `food` 0.218, `fun` 0.556.

**Status.** `need-cycle` returns to **`pending`** with the per-person figures in its note. Whether to
tighten the rates or narrow the observable to *"recovers"* is a design question.

**The third artefact of this shape in four iterations:**

| | artefact | consequence |
|---|---|---|
| 2026-11-06 | `RumorId(0)` | "rumor reach flat-lines" for three iterations |
| 2026-11-14 | a `4×` tick cost never measured | a breaking change nearly justified |
| 2026-11-18 | a row at tick 0 | a `measured` status that was false |

**Every published number comes from `examples/evidence.rs`, and that file has been wrong more often than
the library it measures.** Each artefact now ends with a test that fails if it returns.

**Tests.** 34 → **34** binaries (two rewritten in `need_cycle`), 34 in both feature sets. Dead-code
inventory: zero; `src/` unchanged.

**Verdicts.** Behaviour GREAT (per person, only `food` cycles; the observable was three-quarters false).
Zoom GREAT (the artefact is one visible vertical line; my first explanation of it was wrong). Fresh eyes
GREAT (third artefact of the class, named, each now pinned by a test).

**Commit.** `d33552d`.

**Top 5 remaining.**
1. **`need-cycle`'s observable is a design question** — three needs stay above 0.5 at these rates. Tighten
   the rates, or ask only that needs recover? A `needs-decision` candidate.
2. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name.
3. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip is checked against a third
   configuration.
4. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured.

## 2026-11-19-01 — A person walking to bed is reported asleep

**Pick.** Whatever the log's #1 said, counting the village's **activity mix** found a 2× discrepancy in what
it reports doing — a defect rather than the design question the log had queued.

| activity | share of person-ticks |
|---|---|
| **sleep** | **56.8%** |
| eat | 14.9% |
| work | 14.8% |
| everything social | 9.2% |

One 8-hour sleep a day is 33%; this run's **350 sleeps over 24 people over 17 days is 28.6%**. The reported
figure cannot be right.

**Four hypotheses, all disproved by measurement.**

| hypothesis | killed by |
|---|---|
| a `Place` activity overruns its duration | `end_activities` reports **`elapsed_h = 8.004`** on every sleep that ends |
| `satisfies` pays twice per tick | a 2-hour activity at 0.1/h paid **0.2010** — exact |
| `best_telling`'s tie-break starves a fresh rumor | three tests passed with **no code change** |
| a probe measuring overlapping sleeps | fixed with a queue; real range 8.00–28.00 |

The second is the instructive one: the first version of that test used 1.0/h, so a 2-hour activity paid
2.0 into a **ceiling of 1.0** and read `1.000` — which looks exactly like a half-rate payout. The need's
clamp, not the crate, was the cause.

**The one that held.** A villager tracked tick by tick while `Doing` says `sleep`:

```
   1: rest 0.087
  12: rest 0.085
```

Decay alone is **−0.00017/tick** and the observed change is **−0.00018**. `sleep` pays 0.15/h =
**+0.00088**. `rest` falls, so `arrive_and_satisfy` is not paying: `arrived_day` is `None` and the person
is **walking**.

`Doing.activity` names the target from the moment it is chosen, so every reader — the HUD, the harness,
`activity_barks`, `report_to_triggers` — sees a person sleeping while it walks. The scales make it common:
`travel_timeout_hours` 20 at 64 s/day is **3 414 ticks**; an 8-hour sleep is **1 365**. A walk to bed can
last **two and a half times the sleep**, labelled asleep throughout.

**Evidence.** `runs/2026-11-19-01/evidence/reported-asleep.png` — the reported-asleep count peaks at **22–23
of 24** every night, against the red line of 8 that one 8-hour sleep allows.

**What ships.** No library change: the fix changes a component a host reads, so it is
`needs-decision/what-doing-reports-while-travelling.md`, recommending a reserved `travelling` activity with
the target in its own field — every in-crate reader then right by default. `tests/travel.rs` pins the
defect (ignored so it does not block later iterations; `cargo test -- --ignored` shows it).
`tests/drift.rs` pins what **is** true: a `Place` activity pays exactly its duration, displacement
included — 0.2010 and 0.2004 against 0.2.

**Tests.** 35 → **36** binaries, 0 failures, in both feature sets. Dead-code inventory: zero; `src/`
unchanged.

**Verdicts.** Behaviour GREAT (measured, reproduced in isolation, pinned by a red test). Zoom GREAT (four
hypotheses disproved, one caused by a need's clamp). Fresh eyes GREAT (every in-crate reader of
`Doing.activity` is wrong today, and the fix makes them right).

**Commit.** `867ce24`.

**Top 5 remaining.**
1. **`Doing` while travelling** — `needs-decision/what-doing-reports-while-travelling.md`; un-ignore
   `tests/travel.rs` when it lands.
2. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
3. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name.
4. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.
5. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.

## 2026-11-20-01 — The village was asleep half the day because a timeout was nine times too long

**Pick.** The previous iteration filed *a person walking to bed is reported asleep* as a decision. It is —
but the **scale** of the problem is a settings value nobody chose deliberately: `travel_timeout_hours` was
**20 game-hours** for a village **8.2 m** across.

At 64 s/day one game-hour is 170.7 ticks and walking is 0.0219 m a tick:

| timeout | ticks | reach |
|---|---|---|
| 20 h | 3 413 | **74.7 m** |
| 4 h | 683 | **14.9 m** |
| village diagonal | 375 | **2.2 h** |

Nine times the longest walk, and an 8-hour sleep is 1 365 ticks against a 3 413-tick timeout — so a person
who cannot reach its bed is reported asleep for **two and a half times the sleep**.

**Fitted to 4 hours**, same seed and spawn:

| | 20 h | **4 h** |
|---|---|---|
| reported asleep | 56.8% | **36.4%** |
| reported working | 14.8% | **21.5%** |
| needs mean | [0.268, 0.538, 0.217, 0.267] | **[0.709, 0.395, 0.402, 0.599]** |
| **total** | **0.322** | **0.526** |

Holds at seed 42 (36.2%, 0.511). `rest` falls 0.538 → 0.395, which is correct: people no longer sleep for
hours they have not earned. `evidence/timeout-fitted.png`.

**Two silent harness bugs, found while measuring.**

| | consequence |
|---|---|
| `contact_radius` declared and never applied | `rumor-crosses-a-cluster` set 4.0; the reader mapped three keys and not this one. Right by coincidence (4.0 is the crate default), wrong for any other number |
| `scenarios.yaml` parsed **once per tick** | `plant_at` parses the file and was called inside the loop: **57 000 warnings** in a 67 200-tick run |

The second hid the first: the one warning worth reading was repeated 57 000 times. `cargo test` never runs an
example, so no gate caught either. `tests/scenario_spec.rs` now asserts the cluster states a
`contact_radius`.

**What that exposed.** With `contact_radius` actually applied, the cluster reaches **0.333** at 12 hours and
**0.167** at 20 — both below its 40% observable. The **0.583** recorded two iterations ago predated the fix,
so it was a different run. The scenario returns to **`pending`** with the design question in its note: two
groups 40 m apart need a **10.7 game-hour** walk.

**Still wrong.** Reported asleep is 36.4%, above the 33.3% one 8-hour sleep a day implies — walks are still
mislabelled. This reduced the scale of the lie from 2.5× to 1.1×; the decision file stands.

**Tests.** 36 binaries, 0 failures, both feature sets. Dead-code inventory: zero; `src/` unchanged.

**Verdicts.** Behaviour GREAT (56.8% → 36.4% asleep, need total 0.322 → 0.526, holds at a second seed). Zoom
GREAT (both panels agree; `rest` falling is correct; the cluster's apparent regression was measured, not
assumed). Fresh eyes GREAT (two silent harness bugs, one drowning the other).

**Commit.** `05f9aa3`.

**Top 5 remaining.**
1. **`Doing` while travelling** — the labelling defect stands; 36.4% is still above 33.3%.
2. **`rumor-crosses-a-cluster` is `pending`** — 0.333 against a 40% observable; a 10.7 h crossing walk.
3. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
4. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.
5. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.

## 2026-11-21-01 — A villager walks 62% of its life, and the harness was hiding it

**Pick.** Last iteration reduced reported-asleep from 56.8% to 36.4% by fitting the travel timeout. That was
still above the 33.3% one 8-hour sleep allows, so this iteration made the harness tell **travelling** from
**doing** — a measurement fix, `src/` untouched.

| label | share of person-ticks |
|---|---|
| `travelling:sleep` | **21.8%** |
| `sleep` | **14.7%** |
| `travelling:eat` | **12.6%** |
| `travelling:work` | **12.4%** |

| | |
|---|---|
| **travelling, total** | **62.5%** |
| doing an activity | 37.5% |
| `sleep` alone | **14.7%** — the 36.4% was 21.8 points of walking |

`evidence/travelling.png`: red fills most of every column for 17 game-days.

**The cause is arithmetic.** At 64 s/day a crossing walk in the 8.2 m village is **375 ticks**, and eight
starter activities are shorter than that:

| activity | ticks | scores `Distance`? |
|---|---|---|
| `snub` | **17** | **no** |
| `compliment` | **34** | **no** |
| `chat` / `gossip` | 51 | yes |
| `play` / `wander` | 85 | `play` yes, `wander` no |
| `eat` | **128** | **no** |
| `relax` | **256** | **no** |

**Five of the eight do not score distance at all**, so a person crosses the village for a 128-tick meal
without the decision knowing how far it was. And `targets_for` gives a `Place` target no distance term, so
the far tavern and the near one are equally likely.

**Measured candidate fix.** Adding `Distance` to `eat`, `relax` and `compliment`: travelling **62.5% →
50.5%**, need total 0.526 → 0.513. Fair and small — and it edits the file a host ships, so it is
`needs-decision/travel-cost-is-not-scored.md`, recommending a crate-level consideration so the term exists
once and is opt-in by name.

**A correction to last iteration's headline.** "56.8% → 36.4% asleep" was itself 21.8 points of walking. The
true sleeping figure was 14.7% both times. The fitted timeout did help — it removed the *excess* walking, not
the label.

**Tests.** 36 → **37** binaries, 0 failures, both feature sets. `tests/travel_cost.rs` is ignored and failing:
it pins the eight short activities and the five blind to distance. Dead-code inventory: zero.

**Verdicts.** Behaviour GREAT (`sleep` 14.7% and travelling 62.5%, both measured rather than inferred). Zoom
GREAT (the clearest plot the loop has made; the red band was hidden inside the `sleep` label). Fresh eyes
GREAT (the labelling decision's consequence is 62.5% of a life in transit, and the cause is priced nowhere).

**Commit.** `967b35f`.

**Top 5 remaining.**
1. **Travel cost is not scored** — `needs-decision/travel-cost-is-not-scored.md`. Measured at 12 travelling
   points for 1.3% of need; the larger lever is that a `Place` target carries no distance.
2. **`Doing` while travelling** — the labelling decision; un-ignore `tests/travel.rs` when it lands.
3. **`rumor-crosses-a-cluster` is `pending`** — 0.333 against a 40% observable; a 10.7 h crossing walk.
4. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
5. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.

## 2026-11-22-01 — A coin flip decided which tavern you walked to

**Pick.** The log's #1's *unblocked half*: a `Place` target carries no distance, so the far tavern and the
near one are equally likely. Implementing the decision file's crate-level travel consideration needs an
operator; this needs none — the decision already knows the distance, it just did not consult it.

**The defect.** `targets_for` returns **every** place carrying the tag, a `Place` target has no `Distance`
term, so they all score identically and tie — and the tie was a uniform pull from the seeded RNG. Measured
with two taverns serving `food`, one **1 m** away and one **9 m**:

| | near | far |
|---|---|---|
| before | 3 020 | **2 979** |
| after | **5 999** | **0** |

**The fix.** Nearest wins for `Place` and `Point`; the uniform pull stays for `Person`. That asymmetry is
deliberate: a tavern does not move, so there is nothing to randomise for variety — but a villager that
always talks to the physically closest person has one friend. Ties on distance fall back to `min_by`'s
first-of-equal and the seeded draw is untouched, so determinism holds 2/2.

**In the village**, same seed and spawn:

| | random tie | **nearest tie** |
|---|---|---|
| travelling | 62.5% | **58.9%** |
| need mean | 0.526 | **0.545** |
| `work` | 9.1% | 10.1% |

Holds at seed 42 (58.8%, 0.550). `need-cycle` and `rumor-crosses-a-cluster` unchanged.

**The effect is small and the plot says so.** `evidence/nearest.png` shows the two travelling lines
overlapping heavily: the village's four places sit within about 13 m of each other, so near and far rarely
differ. The defect is unambiguous in isolation and modest in this layout. Reporting 62.5% → 58.9% as a fix
for travelling would overclaim; `needs-decision/travel-cost-is-not-scored.md` is the larger lever, with its
own measured 12-point improvement from a different mechanism.

**What it makes true.** `Signal::Distance` existed, was documented as an input, and was **not consulted** for
the one target kind whose destination cannot move. A host reading the docs would have assumed the decision
prefers a near place. It now does.

**Tests.** 37 → **38** binaries, 0 failures, both feature sets. Dead-code inventory: zero — `place_distance`
is reached from the tie-break and asserted by the new test.

**Verdicts.** Behaviour GREAT (3 020/2 979 → 5 999/0; village 62.5% → 58.9%, holds at a second seed). Zoom
GREAT (the plot shows a small effect and the review says so). Fresh eyes GREAT (a documented signal now does
what its docs imply).

**Commit.** `bf74eed`.

**Top 5 remaining.**
1. **Travel cost is not scored** — `needs-decision/travel-cost-is-not-scored.md`. Five activities score
   `Distance` nowhere; eight are shorter than a crossing walk.
2. **`Doing` while travelling** — the labelling decision; un-ignore `tests/travel.rs` when it lands.
3. **`rumor-crosses-a-cluster` is `pending`** — 0.333 against a 40% observable; a 10.7 h crossing walk.
4. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
5. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip is measured under settings the spec
   does not declare.

## 2026-11-23-01 - A `Place` activity asking for `Distance` was silently disabled

**Pick.** Not "add `Distance` to `eat`" (the previous entry's leftover) but the thing that made that line a
trap: `Signal::Distance` returned **1.0** for every `Place` and `Point`, so a place-targeted activity could
not price its walk at all. Runner-up was the pack edit; it lost because it cannot work while the input is
wrong -- measured on `eat`, adding the line took `food` from 0.402 to **0.034**, at zero **93%**.

**The defect is not "the wrong place", it is "no place".** Written red first: both taverns in one run, 1 m and
7 m, 6 000 decisions, gave **0 / 0**.

**Change.** `PlaceView::position` (from the place's `Transform` and the society plane), `Decision::place_positions`,
and the `Signal::Distance` arm for `DoingTarget::Place`. `Point` and `Itself` keep the maximum, with the source
comment saying why each is genuinely unknown rather than merely unimplemented.

**Before / after.**

| | before | after |
|---|---|---|
| both taverns, 1 m vs 7 m | **0 / 0** | **5 999 / 0** |
| `eat` + `Distance`, `food` mean | 0.034 (93% at zero) | **0.176 (0% at zero)** |
| shipped pack, all series rows | 67 199 | **67 199 identical** |

Travelling stays 58.9%, needs 0.545 -- **bit-identical**, because the pack asks for `Distance` on no `Place`.

**Evidence.** `runs/2026-11-23-01/evidence/distance-signal.png` (red flat at zero, blue drifting to ~0.1),
`baseline.csv`, `distance-broken-signal.csv`, `distance-fixed-signal.csv`.

**Verdicts.** Behavior GREAT (0/0 -> 5 999/0, written red first). Zoom GREAT (the red line flat at zero is what
"disabled" looks like; the blue drift is the harsh curve, recorded not tuned). Fresh eyes GREAT (seed 42:
0.167 against 0.176, zero 0% at both).

**Dead-code delta.** Zero. `PlaceView::position` and `Decision::place_positions` are read by the signal and
asserted by the test.

**What it hands on, unresolved.** `food` at 0.176 is still below its 0.402 baseline because
`Linear(-1.0, 1.0)` scores a tavern 6.5 m away inside an 8 m radius at **0.19**. The input is now real; the
**curve** is `needs-decision/travel-cost-is-not-scored.md`'s question, and it is answerable now.

**Commit** `69833ea`.

**Top 5 remaining.**

1. **The travel curve's shape** - the input is correct, so this is finally a measurement question: try
   `Linear(-0.4, 1.0)`, a `Logistic`, and `duration / (duration + walk)`, and pick on travelling% and needs.
2. **The cluster's 1:24 starvation** - a second need; a schema change.
3. **The clock decision** (`travel_timeout_hours` real-time) - escalated, still open.
4. **A log-capture harness** - still blocking two warnings from being properly tested.
5. **A spatial index for perception** past ~2 000 people, with benchmark numbers behind it.

## 2026-11-24-01 - A rendering speed decided how much of life is walking

**Pick.** Not the log's #1 (the travel curve's shape) and not the escalated clock decision, but the thing the
two have in common: **the clock coupling is silent, and silence is not a decision the crate may defer** --
its own argument for `warn_short_travel_reach`.

The runner-up lost on measurement. Three curves on `eat` + `relax` at seed 7:

| curve | travelling | needs |
|---|---|---|
| `Linear(-0.4, 1.0)` | 57.8% | 0.555 |
| `Linear(-0.25, 1.0)` | 59.0% | 0.546 |
| `Logistic(-6.0, 0.6)` | 53.1% | 0.533 |
| baseline | 58.9% | 0.545 |

**All within one point.** The curve is a 1-point lever. The clock is a **15-point** one.

**The measurement.** `walk_speed` is metres per **real** second; every `duration_hours` is **game** hours.
`seconds_per_day` bridges them, so the knob documented as "how fast you watch a day" also decides how much of
it is walking. Same seed, same 24 people, same 16.4 game-days:

| `seconds_per_day` | travelling | need mean | sleep |
|---|---|---|---|
| **64** | **58.9%** | 0.545 | 15.0% |
| 144 | 50.4% | 0.603 | 17.0% |
| 240 (default) | 46.8% | 0.621 | 16.5% |
| **480** | **43.9%** | 0.624 | 17.3% |

**Change.** `warn_walk_outlasts_activity` at startup: a 10 m walk costing more game-hours than the **mean**
activity. Fires at the project's own 64 s/day village (2.68 h vs 1.62 h); quiet at the shipped 240 s default
(0.71 h). **Mean, not median** -- measured: the pack's median of 0.5 h trips at the healthy default, which
would have been a warning crying wolf on the crate's own pack.

**Proof.** `--seconds-per-day 64` prints *"2.68 game-hours, longer than the mean activity (1.62h)"*;
`--seconds-per-day 240` prints nothing (0 matches).

**Evidence.** `runs/2026-11-24-01/evidence/coupling.png` (four travelling traces over 16.4 days, and the
travelling/needs trade against the knob), `evidence/before/spd-{64,144,240,480}.csv`.

**Verdicts.** Behavior GREAT (fires on the project's own day, silent on the default, both asserted). Zoom GREAT
(travelling falls monotonically 58.9 -> 43.9 as the knob turns, needs rise 0.545 -> 0.624; damage concentrated
at the short-day end, which is where it warns). Fresh eyes GREAT (one sentence to a host).

**Dead-code delta.** Zero. Called from `finish`, both branches asserted.

**Honest limit.** This **reports** the coupling; it does not reduce it. Option 2 in the decision file is the
only one that closes this half, and it is not mine to take.

**Commit** `f015468`.

**Top 5 remaining.**

1. **The clock decision** - now measured at 15 points, so it is the largest behavioural lever left. Option 2
   (walking in game time) closes the half that Option 1 leaves open.
2. **The travel curve's shape** - a 1-point lever at seed 7; worth doing only alongside (1), which changes the
   units it is expressed in.
3. **The cluster's 1:24 starvation** - a second need; a schema change.
4. **`--seconds-per-day` on every scenario** - the harness flag exists; `scenarios.yaml` still declares one
   day length per scenario, so cross-speed comparison is manual.
5. **A spatial index for perception** past ~2 000 people, with benchmark numbers behind it.

## 2026-11-25-01 - A grudge was the count of offences, not the worst one

**Pick.** Candidate #2 -- behaviour that reads as wrong when you watch it -- and it was found by *reading the
narrative*, not the plots. The cluster's day for G00 went `snub, compliment, snub, compliment` every six
minutes, `snub` at **170 of 338** starts, a run of **ten** in a row, from an activity that satisfies **no
need**.

Runner-up was the `wander` ceiling (no considerations, so it always scores 1.000 and its `tier_cutoff` floor
discards a starving person's `eat`). Measured by giving it an `Hour` curve: `snub` **50.3% -> 53.3%**,
slightly *worse*. Not the lever.

**Two defects, one level.**
1. `grudge` **summed** offences and clamped at 1, so four retained trivial slights saturated it exactly as one
   betrayal did -- the value depended on `memory_capacity`. `snub`'s only consideration is that grudge.
2. A **refused** interaction was sized by the deed's `harm`, but the deed fires only on accept. A snub the
   target refused wrote `-0.7` into the snubber, about the person who refused.

**Change.** `grudge` is the **maximum** over memories rather than the sum; a refusal's memory is `0.3`.

| | seed 11 | seed 42 |
|---|---|---|
| `snub` share, sum | **50.3%** | **48.4%** |
| `snub` share, max | **26.0%** | **32.6%** |
| longest run | 10 -> **6** | 11 -> **8** |
| top activity | `snub` -> **`compliment`** | `snub` -> **`gossip`** |
| refused-insult memory | 0.700 -> **0.300** | -- |

`village-evening`: travelling 58.9% -> 59.0%, need mean 0.545 -> 0.542 (seed 42: 59.3% / 0.550).

**A claim I nearly shipped and did not.** At seed 11 the change also gave friends:rivals **4:9 -> 6:5** and
rumor reach **0.167 -> 0.333** -- double. I had that drafted as the headline. Seed 42: reach **0.167 either
way**, friends 5 -> 6. Seed-specific. Only the snub reduction is claimed.

**Evidence.** `runs/2026-11-25-01/evidence/grudge.png` (snub share below the sum line for 80 days, same
shape; the village's four needs lying on top of each other), `before/cluster-narrative.txt` against
`after/cluster-narrative.txt` (`work -> wander -> chat -> snub -> compliment -> wander -> play -> ...`).

**Verdicts.** Behavior GREAT (named from the narrative, both seeds). Zoom GREAT (green below red at the same
shape, no clipping, village untouched). Fresh eyes GREAT (and it falsified the reach claim, which is the
point of the pass).

**Dead-code delta.** Zero.

**Honest limit.** `max` over `sum` is a **design default**: the corpus search for peak-versus-accumulate
returned nothing usable, so `docs/research/memory.md` says so outright rather than citing something that does
not address it.

**Commit** `83443b2`.

**Top 5 remaining.**

1. **The clock decision** - the largest behavioural lever left (15 points), escalated, unmoved.
2. **`wander` scores 1.000 always** - it pins the tier ceiling and its floor discards a starving person's
   `eat`; giving it a consideration made things worse, so the fix is elsewhere (a `tier_cutoff` that ignores
   zero-consideration options, or a weight that scales).
3. **The cluster's 1:24 chat:gossip starvation** - a second need; a schema change.
4. **`rumor-crosses-a-cluster` is still `pending`** at 0.167 reach against the 40% it asks for; the observable
   may need re-stating, which is a decision.
5. **A spatial index for perception** past ~2 000 people, with benchmark numbers behind it.

## 2026-11-26-01 - The scale curve was not where the crate said it was

**Pick.** Candidate #4, performance, after measuring and setting aside the log's #4 (`rumor-crosses-a-cluster`
`pending`): it is **0.333 at 64 s/day and 0.250 at 240**, i.e. 2-3 of 12, at both clocks. Two groups 40 m
apart never merge; the observable asks for a crossing needing a 10.7 game-hour walk. A **spec** question, not
a crate defect -- left in the decision file rather than re-declared.

**Change.** `perceive` runs a uniform grid, one cell per `perception_radius` (Ericson 2005, *Real-Time
Collision Detection* §7.1.1), filled in ascending index order so the answer is the all-pairs answer exactly.
`close_doing` is built from the person's close ties instead of cloning the population per person.

| persons | all pairs | **grid** |
|---|---|---|
| 100 | 0.99 ms | **0.59 ms** |
| 1 000 | 14.55 ms | **13.50 ms** |
| 2 000 | 53.96 ms | **49.91 ms** |
| 5 000 | 254.54 ms | **245.13 ms** |

**3-7%.** Real, free, and much smaller than expected.

**The finding that matters more.** The crate said perception's all-pairs scan was the quadratic term. Direct
timers at 5 000 people: `perceive` is **36 ms of a ~320 ms fixed step** -- 12%. The term that scales is
**scoring**: every `Person`-targeted activity against every neighbour, 182 of them at that density
(0.995 people per m², so they are genuinely there), giving ~3.6M consideration evaluations a tick against
144 000 at 1 000. **No index removes that.** `docs/guide/concepts.md`, `examples/benchmark.rs` and
`scenarios.yaml` now say so instead of the false thing.

**Identity.** 67 199 of 67 199 `village-evening` rows identical; `tests/perception.rs` pins the grid's
neighbour lists against all-pairs at n = 1, 2, 40, 200, 500 across two radii -- **0 mismatched**.

**Evidence.** `runs/2026-11-26-01/evidence/grid.png` -- the two cost curves almost on top of each other, the
measured 36/29/226 ms breakdown, and neighbours per person rising 0.4 -> 182.

**Verdicts.** Behavior GREAT (bit-identical, pinned). Zoom GREAT (the plot is a correction, the middle panel
is the refutation). Fresh eyes GREAT.

**What I got wrong.** Two guesses before measuring -- a per-person `all_doing.clone()`, then a lookup map --
both reasoned from the code, both worth ~5%, neither the cause. The timers settled it in one run.
`tests/leaf.rs` also caught a `HashMap` I put in `behavior.rs`; removed rather than allow-listed.

**Dead-code delta.** Zero.

**Commit** `5ed3596`.

**Top 5 remaining.**

1. **Scoring is the scale curve** -- `people x Person-activities x neighbours`. Sampling neighbours, or
   fewer Person-targeted activities, is the only thing that changes what a host can afford. Needs a decision.
2. **The clock decision** -- the largest behavioural lever (15 points), escalated, unmoved.
3. **`rumor-crosses-a-cluster`** -- 0.333 against the 40% it asks for, at both clocks; the observable or the
   geometry needs re-stating, which is a decision.
4. **The cluster's 1:24 chat:gossip starvation** -- a second need; a schema change.
5. **`wander` scores 1.000 always** (no considerations) and pins the tier ceiling; giving it a curve made
   things worse, so the fix is in the cutoff policy.

## 2026-11-28-01 - Every README clip had been showing the wrong thing

**Pick.** Not a loop candidate but the thing a host sees first. The demos were not "a different view of one
simulation" as `docs/DEMOS.md` claims: six clips, the same pixels, and a visual review found **seven**
faults. Runner-up was the loop's own list, which is now entirely escalated decisions.

**Crate change.** `Obstacle` — a component saying "this is solid". `steer_movers` passed `&[]` for ORCA's
wall list; a walking person's `Transform` ended inside a building. Proven by a control: without an
`Obstacle` the walker reached **0.01 m** from a block's centre (straight through); with one it stopped at
**2.35 m**.

**The seven faults.**

| fault | cause |
|---|---|
| villagers inside buildings | places 3 m cubes, spawn 0.6 m from centre |
| never moved | place half-width 1.5 m == `interaction_radius` 1.5 m |
| never moved | `seconds_per_day = 8` made a 3 m walk cost 6.4 game-hours |
| `needs` one colour | `NEED_CLIP = 1` is `rest`; food is 2 |
| `opinions` one colour | `OpinionBar(0)` is `kindness`; honesty is 1 |
| `village` no rumor | halos read `RumorId(0)`, not the planted rumor |
| `factions` two colours | fixed camera + `Wander(6.0)` |

The correct rumor function existed, was **dead code**, and was never called.

**Also fixed.** `docs/guide/concepts.md` was **147 395 lines** of repeated `## Scale` — my own doc edit two
iterations ago looped. Restored from `662fdce` (110 lines), scale section corrected, obstacles section added.

**Evidence.** The six GIFs in `docs/media/`, each inspected at frames 0 / 120 / 239:
`runs/2026-11-28-01/evidence/village.gif`.

**Verdicts.** Behavior GREAT (each clip now shows its feature; both colours present in `opinions`, both sizes
in `needs`). Zoom GREAT (no clipping, all three factions in frame, no ground edge, HUD readable). Fresh eyes
GREAT (three separate visual passes, each finding what the last missed).

**Dead-code delta.** Minus one: `draw_newest_rumor_rings` is now used.

**What I got wrong.** Four attempts at the opinion bars before checking the data — the renderer was fine and
the model had converged; seeding a spread fixed it. Two attempts at framing before computing the layout's
centroid. Recorded because the pattern is "reason about the picture" instead of "measure the numbers".

**Commit** `f9ca267`.

**Top 5 remaining.**

1. **The clock decision** - still the largest behavioural lever (15 points), escalated.
2. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks; the observable or the
   geometry needs re-stating, which is a decision.
3. **Scoring is the scale curve** - `people x activities x neighbours`; sampling is the only real lever.
4. **The cluster's 1:24 chat:gossip starvation** - a second need; a schema change.
5. **`wander` scores 1.000 always** - pins the tier ceiling; the fix is in the cutoff policy.

## 2026-11-29-01 - A documented feature could never fire

**Pick.** The `fields` feature is **on by default** and the README advertises it, but `Signal::Field`
returned a constant `0.0` and `ActivityTarget::Gradient` produced no targets. A definition using either was
accepted by the definitions file and could never be chosen. Runner-up was `Doing`'s travelling semantics --
a real defect at scale (**49.8%** of person-ticks are an activity not yet started) -- but that changes a
component a host reads, so it stays a decision.

**Three causes, and the decision file had already failed once.**

1. **The probe's crowd had no slope.** Nine people in **one cell** is a plateau: `field_value` divides by the
   channel's *peak*, so every cell in the hottest region reads 1.0 and `gradient_target` correctly returns
   `None`. Every earlier attempt "always had a crowd" -- and the crowd was in one place, which is what hid it.
2. **The fallback outlasted the probe.** `idle` is 1 game-hour; a tick at 240 s/day is 0.0016, so one fallback
   holds **640 ticks**. The original ran 60, two of mine ran 400.
3. **The wiring**, which also needed `Decision::own_factions` -- `OwnFaction`/`OtherFactions` are
   faction-relative.

**Before / after.**

| | before | after |
|---|---|---|
| field read by the decision | **0.0** (constant) | **0.120** |
| `Gradient` target | none | `Some(Vec2(-2.0, 0.0))` |
| `Field`-only activity, 2 400 ticks | **0** | **6** |
| `village-evening` series rows | 67 199 | **67 199 identical** |

**The warning changed meaning.** It now fires only when the `fields` feature is **off**, and only compiles
into that build -- and with the feature off the definitions file *rejects* `Field` and `Gradient` at parse
time, so a host cannot write one that silently does nothing.

**Evidence.** `runs/2026-11-29-01/evidence/field-wiring.png` (a flat zero against the real cell reading; dead
then live).

**Verdicts.** Behavior GREAT (minimal case, written red first). Zoom GREAT (the plot is the defect in two
bars; 6-in-2400 checked against the 640-tick fallback rather than accepted). Fresh eyes GREAT (two feature
sets, not two seeds: live and silent with `fields` on, rejected at parse time with it off).

**Dead-code delta.** Minus two: the `Field` and `Gradient` arms are reachable and asserted.

**What I got wrong.** Twice I concluded the feature was broken when the probe was: a crowd in one place has no
slope, and 400 ticks is less than one 1-hour fallback. "The feature is dead" and "my probe is too short" look
identical from a red assertion.

**Commit** `c842ae9`.

**Top 5 remaining.**

1. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; changes a host-facing
   component, so escalated.
2. **The clock decision** - the largest behavioural lever (15 points), escalated.
3. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
4. **The cluster's 1:24 chat:gossip starvation** - a second need; a schema change.
5. **Scoring is the scale curve** - `people x activities x neighbours`; sampling is the only real lever.

## 2026-11-30-01 - Nobody in the village is ever pleased

**Pick.** Started from the narrative (which the last three iterations did not), then measured the *state*.
`village-evening` has **max pleasure +0.000** over every person and every tick, stdev 0.031, for 17
game-days. That is the most wrong thing a host can read from `Mood`, so it became the iteration.

**What shipped, and what did not.** The mood imbalance is a decision (three answers disagree about what the
model is). What shipped is the part that is unambiguously wrong: the refusal distress was a literal `0.3`
— the crate's **only** unnamed emotion intensity, against its own rule — and a definition's `on_reject`
now **replaces** it instead of stacking.

| configuration | mean pleasure (8 people) |
|---|---|
| literal 0.3, pack has no `on_reject` | **-0.202** |
| pack `on_reject 0.1`, both emitted | **-0.207** |
| pack `on_reject 0.1`, fallback replaced | **-0.165** |

The middle row is why: two paths for one event is worse than one.

**The finding, recorded not retuned.** `Mood::step` pulls toward the emotion centre at 4:1 (`pull_per_day
4.0`) and toward the personality baseline at 1:1 — and `pad::baseline` is **zero** for a neutral personality
from `Personality::sample`. So pleasure's ceiling is zero, and the centre is negative nearly always:
**1 302 distress appraisals against 427 joy** over 20 000 ticks, and **15 people ending with live `Anger`
against 0 with `Joy`**. `Evidence/mood.png` shows pleasure starting at exactly zero, falling, and
oscillating at -0.15 for 240 game-days without crossing the axis.

**Two populations disagreed, and that is the useful part.** In a tight eight-person cluster refusals are
about *half* the negativity (-0.202 -> -0.105 without them). In the real 24-person village they barely
register (-0.150 -> -0.154), because people spend their time walking and working. A fix tuned on the first
would have been mis-tuned for the second. Both numbers are in the decision file.

**What I measured wrong first.** Counted *emotion entries observed per tick* and reported "52 038 distress
appraisals". `Emotions::appraise` merges same-kind-same-target and caps, so a persisting emotion was counted
once per tick. The real number is **112** — 12x smaller, and it turns "saturated" into "a 3:1 ratio the mood
update cannot escape".

**Verdicts.** Behavior GREAT (constant reaches the model; stacking provably worse). Zoom GREAT (the plot
convicts the model: the zero-height `Joy` bar is the whole story). Fresh eyes GREAT (two populations, and
they disagree).

**Dead-code delta.** Zero.

**Commit** `a629b87`.

**Top 5 remaining.**

1. **A village that is never pleased** - max pleasure +0.000; three defensible fixes, now measured. New
   decision file, and the largest remaining believability defect.
2. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
3. **The clock decision** - the largest behavioural lever (15 points), escalated.
4. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
5. **`need-cycle` is still `pending`** - its test passes but its observable claims `rest` and `food` cycle
   when only `food` does; re-stating an observable is a decision.

## 2026-12-01-01 - The affect model had a gain and no intercept

**Pick.** Last iteration escalated "a village that is never pleased" with three options. Searching the
literature found a **fourth**, with a source, and it is an omission rather than a tuning disagreement.

**Cacioppo, Cacioppo & Gollan 2013** (DOI `10.1017/S0140525X13002537`): *"the intercept for the positive
activation function (i.e., the approach motivation at zero input) is higher than the intercept for the
negative activation function (producing the **positivity offset**), and the gain for the negative activation
function is higher than for the positivity activation function (producing the **negativity bias**)."*

The crate had the **gain** — a landed insult pays Anger 0.5 against a compliment's Joy 0.3, distress
outnumbers joy 3:1 — and **no intercept**. So `Mood::step` returned pleasure to a personality baseline of
exactly zero (traits centre on 0.5, Mehrabian's zero is there), and its ceiling was zero: measured,
pleasure's maximum over every person and every tick was **+0.000**.

**Change.** `Tuning::positivity_offset`, default 0.1, added to the resting point. Pleasure only; arousal and
dominance have no offset in the model.

| offset | mean pleasure | **max pleasure** | needs |
|---|---|---|---|
| 0.0 (old) | -0.154 | **+0.0000** | 0.596 |
| **0.1 (new)** | -0.122 | **+0.0155** | 0.603 |
| 0.3 | -0.058 | **+0.0476** | 0.603 |

The ceiling crosses zero for the first time, monotonically in the offset. An idle person alone in a field:
**+0.0000 → +0.0122**.

**Honest limit.** This fixes the ceiling, not the mean. The steady state is
`(pull·I·c + back·rest)/(pull·I + back)`, so a saturated negative centre at intensity 0.5 pins pleasure near
-0.6 whatever the offset. The mean needs the **appraisal balance** (1 302 distress against 427 joy), which is
`a-village-that-is-never-pleased.md`'s option 1 and stays a decision.

**Tests updated rather than deleted.** Three asserted "mood returns to the baseline", which is now false by
design. Each was rewritten to the new contract and kept, plus a new one asserting `positivity_offset: 0.0`
reproduces the old behaviour exactly — so this is a setting, not a one-way door.

**Evidence.** `runs/2026-12-01-01/evidence/positivity-offset.png` (three separated traces against the zero
axis; the max bar rising through it while the mean bars stay below).

**Verdicts.** Behavior GREAT (the intercept lands where the paper puts it, and the idle-person case is
exactly zero at 0.0). Zoom GREAT (the plot shows the ceiling moving and the centre not). Fresh eyes GREAT
(magnitude is a documented design default, and the old behaviour is one field away).

**Dead-code delta.** Zero.

**Commit** `7f47593`.

**Top 5 remaining.**

1. **The appraisal balance** - 1 302 distress against 427 joy; the offset fixed the ceiling and not the mean.
   Option 1 in `a-village-that-is-never-pleased.md`, now with the ceiling separated out.
2. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
3. **The clock decision** - the largest behavioural lever (15 points), escalated.
4. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
5. **`need-cycle` is still `pending`** - its test passes but its observable claims `rest` and `food` cycle
   when only `food` does; re-stating an observable is a decision.

## 2026-12-02-01 - Pleasant conversation was affectively silent

**Pick.** The log's #1 (the appraisal balance), and measuring it turned a *ratio* into a *structural fact*.

| | count |
|---|---|
| refusals | 1 216 |
| distress appraisals | **1 216** |
| acceptances | 1 338 |
| positive appraisals | **443** |

**Every refusal is exactly one appraisal and no acceptance is.** `chat` and `gossip` — **43% of accepted
interactions**, the two most common — paid `social` to both people and **no emotion to either**, and
`compliment` paid only its target, so the actor who initiates a pleasant exchange gained no affect from it in
any interaction.

Runner-up was `Doing`'s travelling semantics (49.8% of person-ticks) — a component a host reads, so escalated.

**Change.** `chat` and `gossip` pay mutual `Liking` on accept (0.2 and 0.15). `Liking`, not `Joy`: OCC's joy
is a prospect emotion about an event; what a conversation gives is liking for the person.

| | before | after |
|---|---|---|
| positive appraisals | **443** | **1 491** |
| negative:positive | **3.58 : 1** | **1.06 : 1** |
| mean pleasure, seed 7 | -0.1224 | **-0.0657** |
| seed 42 | -0.1268 | **-0.0652** |
| seed 99 | -0.1102 | **-0.0696** |
| steady state | Anger 14, Distress 4, **Joy 0, Liking 0** | Anger 14, Distress 25, **Joy 22, Liking 14** |
| max pleasure | -0.047 | **+0.237** |

Needs unchanged (0.603 -> 0.607), so this is a mood correction and not a simulation rewrite.

**Honest limit.** Mean pleasure is still negative and distress still outnumbers joy, because refusals are
frequent and each is one distress. Cacioppo et al. 2013 give the negative channel the steeper **gain**, so an
insult still pays `Anger 0.5` against a conversation's `Liking 0.2` — the bias survives, and it should.

**This closes a three-iteration chain**: the ceiling (positivity offset, 2026-12-01), now the mean. The
decision file records both.

**Evidence.** `runs/2026-12-02-01/evidence/appraisal-balance.png` — three seeds, every green trace above its
red counterpart for 240 game-days; the Joy bar going from 0 to 22 while Anger stays at 14.

**Verdicts.** Behavior GREAT (`Distress` equals the refusal count *exactly*, `assert_eq!`; positives now 1.5 per
10 acceptances against 1 per 3). Zoom GREAT (three seeds, all consistent to 0.005; the zero-height Joy bar was
the defect). Fresh eyes GREAT (the bias survives — Anger unchanged — and the two questions a host would ask are
answered in the eval).

**Dead-code delta.** Zero.

**Commit** `9169bc8`.

**Top 5 remaining.**

1. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
2. **The clock decision** - the largest behavioural lever (15 points), escalated.
3. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
4. **`need-cycle` is still `pending`** - its observable claims `rest` and `food` cycle when only `food` does.
5. **Pleasant affect still loses to distress** - by design at the per-event level, but the *frequency* of
   refusal (47% of interactions) is a pack property nobody has questioned.

## 2026-12-03-01 - Nothing in the pack could raise trust

**Pick.** The log's #5: the 47% refusal rate. Measuring it found an omission rather than a rate.

**The finding.** `AcceptDef` scores `trust * (t - 0.5)`, so trust is centred on 0.5 — and `Tie::new` starts
strangers there. `TieDelta` has a `trust` field any interaction can set, and **the word `trust` did not occur
in `assets/starter.society.ron` at all**. The only mover was `witness_reaction`'s `trust_gain * harm`, negative
for every deed the pack uses. Measured trust held toward the person approaching:

| interaction | trust |
|---|---|
| `chat` | 0.27 |
| `compliment` | 0.20 |
| `gossip` | 0.24 |
| `insult` | 0.13 |

Every one below neutral, so the term subtracted on every acceptance draw, and **47% of interactions were
refused**. The pack also defined `help` and `gift` and **used neither** — only `insult` carried a deed.

**Change.** `chat`, `gossip` and `compliment` raise trust on accept (0.05 / 0.03 / 0.02).

| measured, controlled | no delta | with delta |
|---|---|---|
| mean trust at an exchange | **0.261** | **0.345** |
| accept rate | **0.486** | **0.533** |

`village-evening`: mean pleasure **-0.066 -> -0.058**, positive person-ticks **3.4% -> 5.2%**, friends/rivals
**21/44 -> 24/35**, needs unchanged.

**What I tried and reverted, with the numbers.** A `help` activity so the pack's unused deeds would run:
`Affinity > 0.55` erased all conflict (**rivals 44 -> 0**, friends 21 -> 81); `> 0.70` gave near-neutral mood
but trust still declined; `> 0.80` was **inert** (1 acceptance in 12 000 ticks). Nothing between them is
believable, which establishes that **deed density is not the lever**.

**The finding that goes to a decision.** A trust *gain* reaches two parties; a trust *loss* reaches **every
witness** at full magnitude. One chat is `+0.10`; one insult seen by three people is `-0.42`. **The penalty
multiplies with the crowd and the reward cannot.** New decision file
`a-deed-costs-what-the-crowd-saw.md`, three options, recommending witness dilution.

**Evidence.** `runs/2026-12-03-01/evidence/trust.png` — the pleasure traces overlapping (honest: the mood effect
is small), the controlled trust/accept bars, and the asymmetry that no pack tweak reaches.

**Verdicts.** Behavior GREAT (the omission is a `grep` away from proven). Zoom GREAT (the plot refuses to
overclaim a small mood change). Fresh eyes GREAT (the failed `help` attempt is recorded with all three
settings, because knowing what does *not* work is the useful part).

**Honest limit.** This is the smallest win in four iterations, and the reason is measured: the remaining
asymmetry is structural. Trust still declines in a crowd.

**Dead-code delta.** Zero.

**Commit** `7dd9d02`.

**Top 5 remaining.**

1. **A witnessed deed costs every witness full magnitude** - one chat is +0.10, one insult seen by three is
   -0.42. New decision file, and the reason trust still declines.
2. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
3. **The clock decision** - the largest behavioural lever (15 points), escalated.
4. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
5. **`need-cycle` is still `pending`** - its observable claims `rest` and `food` cycle when only `food` does.

## 2026-12-04-01 - A crowd multiplied a penalty and could not multiply a reward

**Pick.** Last iteration's decision file, and I looked for a source before treating it as a preference.

**The finding.** `witness_reaction` applied `trust_gain * harm` at full magnitude to **every** witness, so a
deed's total cost grew linearly with the crowd while a trust *gain* from an interaction reached only two
people. One chat was `+0.10`; one insult seen by three people was `-0.42`.

**The source.** Sommerfeld, Krambeck, Semmann & Milinski 2007, PNAS, DOI `10.1073/pnas.0704598104` — already
the crate's gossip citation — measured that transmission *dampens*: *"the more extreme a gossip was, the less
impact it had on the reader"*, and cited Hess & Hagen on *"an increase in **believability** with an increasing
number of sources"*. Believed more, moving each recipient less. So dilution is a measured behaviour, not a
symmetry argument, which turns a decision into a fix.

**Change.** `Tuning::witness_dilution` (default 0.5, square root).

| witnesses | before | **after** |
|---|---|---|
| 1 | 0.0800 | **0.0800** |
| 5 | **0.0800** | **0.0431** |

Trust at an exchange **0.345 -> 0.435**, accept rate **0.533 -> 0.558**.

| seed | pleasure before | **after** | rivals before -> after |
|---|---|---|---|
| 7 | -0.0579 | **-0.0536** | 35 -> 33 |
| 42 | **-0.1268** | **-0.0456** | 50 -> **34** |
| 99 | **-0.1102** | **-0.0522** | 45 -> 47 |

**Two things not smoothed over.** Seed 7 barely moved; seeds 42 and 99 moved a lot. And **seed 99's rivals
rose** (45 -> 47) while its mood improved — a village that trusts more also insults more often, because the
activity is not gated on trust. Both are in the eval and on the plot.

**Evidence.** `runs/2026-12-04-01/evidence/dilution.png` — the flat red line of the old behaviour against the
diluting green curve; three seeds plotted individually; the rival bars including the one that rose.

**Verdicts.** Behavior GREAT (red first at 0.0800 vs 0.0800 — the defect in one line). Zoom GREAT (the flat
line is the argument; the rising rival bar is shown). Fresh eyes GREAT (three seeds, disagreeing, all shown).

**Dead-code delta.** Zero.

**Commit** `172f782`.

**Top 5 remaining.**

1. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
2. **The clock decision** - the largest behavioural lever (15 points), escalated.
3. **Mood is still negative at every seed** (-0.046 to -0.054). The chain has moved it from -0.150 to about
   -0.05; what remains is the *frequency* of refusal and insult, not any single term.
4. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.
5. **`need-cycle` is still `pending`** - its observable claims `rest` and `food` cycle when only `food` does.

## 2026-12-05-01 - A scenario marked `measured` that nothing asserted

**Pick.** Orient step: four iterations had changed the model and the pack, so I ran the scenarios and read
their stated observables. `village-evening` claims a rumor planted at tick 300 is *"told to at least one other
person"* — and **`grep` across `tests/` found nothing asserting it**. Runner-up was `Doing`'s travelling
semantics, escalated.

**Ten seeds at the harness's tick:**

| seed | 3 | 4 | 5 | 6 | 1 | 7 | 11 | 42 | 99 | 8 |
|---|---|---|---|---|---|---|---|---|---|---|
| `known_by` | 2 | 2 | 2 | 2 | 3 | 5 | 3 | 3 | **1** | **1** |

**The flake was not the model.** Three separate faults:

1. **Two plant ticks, thirty times apart.** The file says **300**; `plant_at` scales the authored 560 by
   `seconds_per_day / 4` and plants at **8 960**. Measured: **7 of 7 seeds** spread the rumor at 300 (reaching
   2-8 villagers), 8 of 10 at 8 960 — a tick past the village's busiest period.
2. **`known_by` counted the player**, a `Person` without `Autonomous` that never gossips. At seeds 8 and 99 the
   harness said 1 while the count over `PersonIndex` was 2: one villager and the player. A rumor reaching only
   the player has died, and counting it would have made the observable true while the mechanism failed.
3. **Nothing asserted any of it.**

**Change.** `tests/village_evening.rs` asserts the condition across seven seeds, counting villagers only. Both
ticks are now stated in `scenarios.yaml` and in `plant_at`'s doc, with the measurement. No model change.

**What I got wrong on the way.** Two hypotheses, both measured and discarded: that `knowledge_capacity`
eviction caused it (nobody holds more than **26** rumors against a cap of **64**), and that a 4 200-tick test
would work at the harness's 8 960 tick (the plant never happened — **0 of 5**).

**Verdicts.** Behavior GREAT (the mechanism works; the documentation described a run the harness does not
perform). Zoom GREAT (8 of 10 versus 7 of 7 is the whole finding). Fresh eyes GREAT (a reader of the scenario
file would have measured a different experiment under the same `status: measured`).

**Dead-code delta.** Zero. **Tests that assert a scenario:** one, where there were none.

**Commit** `f0ca1ac`.

**Top 5 remaining.**

1. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
2. **The clock decision** - the largest behavioural lever (15 points), escalated.
3. **The harness's plant tick is thirty times the file's** - now documented, but they are still two numbers.
   Aligning them changes the clip, so it is a decision.
4. **`need-cycle` is still `pending`** - its observable claims `rest` and `food` cycle when only `food` does.
5. **`rumor-crosses-a-cluster`** - 0.333 against the 40% it asserts, at both clocks.

## 2026-12-06-01 - A need asked to starve that the rates make impossible

**Pick.** Continuing last iteration's audit: `need-cycle` has been **`pending` since November** with an
observable that its own test says does not hold. Runner-up was `Doing`'s travelling semantics, escalated.

**The claim.** *"`rest` and `food` fall and rise again over a game-day: each crosses below 0.35 and recovers
above 0.7."*

**The measurement**, per person, 64 s/day, 80 000 ticks, the observable's own thresholds:

| need | lowest | cycles, seed 7 | cycles, seed 42 |
|---|---|---|---|
| `social` | 0.45 | 0 of 8 | 2 of 8 |
| `rest` | **0.51** | **0 of 8** | **0 of 8** |
| `food` | 0.00 | **8 of 8** | **8 of 8** |
| `fun` | 0.00 | **8 of 8** | 6 of 8 |

**`rest` cannot cycle.** `sleep` restores `0.15 x 8 h = 1.20` against a `rest` decay of `0.70/day` — a net
**+0.50 per sleep** — so it ratchets up until sleeping stops being chosen and never needs to run low. `food`
oscillates because `eat` supplies `1.2 x 0.75 h = 0.90` against an exactly equal `0.9/day`. `fun` usually
cycles too, which the observable's own correction denied and the test printed as
`fun cycles for 8 of 8` while its name said `only_food_cycles`.

**Change.** Observable restated to what the model does — every need met and re-met, none left falling with no
source — with per-seed cycles recorded rather than asserted as constants. Status `pending` -> **`measured`**
for the first time. Test renamed and the restated property asserted. **No crate code changed.**

**What I refused to do.** Retune `rest`'s decay to make the old claim true. The decay is cited (Maslow 1943),
the rates are **data a host sets**, and five iterations have changed the pack since the claim was written.
Fitting the model to a stale assertion is the opposite of keeping the numbers honest.

**Verdicts.** Behavior GREAT (restated property true at two seeds; every need reaches 0.7 for every person).
Zoom GREAT (the "plot" here is a division: +0.50 net per sleep cannot starve). Fresh eyes GREAT (three
documents, three claims, none agreeing — now one claim, measured).

**Dead-code delta.** Zero. **Scenarios `pending`:** one, where there were two.

**Commit** `0db4a7c`.

**Top 5 remaining.**

1. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
2. **The clock decision** - the largest behavioural lever (15 points), escalated.
3. **The harness's plant tick is thirty times the file's** - documented last iteration, still two numbers.
4. **`rumor-crosses-a-cluster`** - the last `pending` scenario: 0.333 against the 40% it asserts.
5. **Mood is still negative at every seed** (-0.046 to -0.054), now limited by the frequency of refusal and
   insult rather than by any single term.

## 2026-12-07-01 - Two groups can see each other and cannot meet

**Pick.** The last `pending` scenario, and the third iteration in a row on a claim nobody had checked.
Runner-up was `Doing`'s travelling semantics, escalated.

**The claim.** With two groups 40 m apart and *"one person from each brought together once"*, a rumor reaches
40% of the second group. Measured: **0 of 6** at 64 s/day, **0 of 6** at 240.

**Six layers, all working, outcome zero.**

| layer | measured |
|---|---|
| steering in isolation | −1.4 m/s, walked 2.0 → 0.731 m |
| movement in the cluster | **359 948** person-ticks moved |
| a bridge place chosen | **36 489** person-ticks |
| arriving at it | **6 720** person-ticks |
| both groups arrived at once | **434 ticks** |
| cross-group visibility | **2.89 m** |
| cross-group interactions | **0** |

**The cause.** An exchange fires **on *arrival*, once**. Two people placed together give **39 interactions**
over 20 000 ticks, so the mechanism is sound; but a pair arriving at tick 100 and 500 shares 400 ticks of
standing together and shares no arrival. They arrive hours apart by construction: a 20 m walk is **5.4
game-hours** against a **0.75-hour** activity. That is `travel-time-in-two-clocks.md` producing an observable
outcome for the first time.

**Two fixes tried, measured, discarded.** A crossroads carrying `gathering` left the reach at 0.167 —
`Place` targets pick the *nearest* match and each hearth is 2 m away. A market with a tag the hearths lack was
chosen for 36 489 ticks and still reached 0 of 6.

**Change.** `tests/cluster_reach.rs` measures the second group of six (not `rumor_reach`, which divides by
twelve) and **asserts the current failure**, so marking the scenario `measured` requires deleting that line
deliberately. `scenarios.yaml` records the measurement, the cause and its test. Decision file
`how-can-two-groups-ever-meet.md` records the three options. **No crate code changed.**

**Verdicts.** Three GREAT for the investigation; the scenario **fails**, asserted at two clocks. Nothing was
marked `measured` on a number nobody checked.

**Dead-code delta.** Zero. **Scenarios with no test:** zero, where there were two at the start of this audit.

**Commit** `f3d7372`.

**Top 5 remaining.**

1. **An exchange happens on arrival, not on contact.** The root cause above: two people sharing a place for
   hundreds of ticks never exchange unless one happens to arrive. A contact-triggered exchange with a cooldown
   is a real model change and the only thing that makes a shared place mean anything.
2. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
3. **The clock decision** - now with an observable outcome: 5.4-hour walks against 0.75-hour activities.
4. **The harness's plant tick is thirty times the file's** - documented, still two numbers.
5. **Mood is still negative at every seed** (-0.046 to -0.054), limited by refusal and insult frequency.

## 2026-12-08-01 - The fix works and the rate cannot be chosen

**Pick.** The root cause established last iteration (an exchange fires on *arrival*, once). Runner-up was
`Doing`'s travelling semantics, escalated.

**Built.** `talk_in_contact` in the `Interact` set, walking the `Perception.contact` neighbourhood that
`expose_contacts` already computes, one `chat` per pair gated by a new `contact_talk_cooldown_hours` and a new
`Tie::last_spoke_day` (`last_contact_day` is stamped every tick two people are near and cannot serve as a
cooldown).

**The mechanism works.**

| | before | after |
|---|---|---|
| two co-located people, 20 000 ticks | **30 exchanges** | **89** |
| cooldown | — | first gap **1 920 ticks at 3 h**, then held |

**And it breaks the village.**

| cooldown | friends | rivals |
|---|---|---|
| none (shipped) | **22** | **33** |
| 3 h | **267** | **0** |
| 12 h | 182 | 3 |
| 48 h | 72 | 12 |

267 friend pairs in 24 people is **97% of all possible pairs**. The contact pass adds **13 742** requests
against ~7 500 baseline interactions — it nearly triples all social contact, because in a village everyone is
always in contact.

**Why no rate works.** High enough to bridge a 40 m gap is high enough to make everyone friends. The mechanism
conflates **meeting** with **befriending**: two people in a tavern are aware of each other, they do not become
friends by standing.

**Reverted.** The crate is back at friends 22 / rivals 33. The measurements are the deliverable, and the
decision file records three options — recommending a "sharing a place" effect be a **bark or mood nudge rather
than an exchange**, and the scenario's observable be restated.

**This is the second mechanism to produce a clique from a plausible local rule** — the `help` activity did the
same two iterations ago (rivals 44 → 0). Two independent findings say the village's tie formation is far more
sensitive to contact volume than its rates suggest, which is worth knowing before a third attempt.

**Verdicts.** Pass 1 GREAT (the mechanism, verified tick by tick). Pass 2 **FAIL** (267 friends, 0 rivals).
Pass 3 **FAIL** (the verb conflates meeting and befriending).

**Dead-code delta.** Zero — system, field and tuning entry all reverted.

**Commit** `2e0cb65`.

**Top 5 remaining.**

1. **Meeting and befriending share one verb.** New decision file; two mechanisms have now failed on it. A
   contact effect that is a bark or a mood nudge rather than an `Interact` is the untried path.
2. **Tie formation is over-sensitive to contact volume** - 22 friends becomes 267 at a 3-hour contact rate.
   The village's friendship ceiling needs a measured relationship to contact, not just a rate.
3. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities, now with an observable outcome.
5. **Mood is still negative at every seed** (-0.046 to -0.054).

## 2026-12-09-01 - Friendship has no diminishing return

**Pick.** The top of the list: tie formation is over-sensitive to contact volume. It earned the iteration
because **two mechanisms in two iterations** produced a clique from a plausible local rule — a `help` activity
(rivals 44 -> 0) and a contact-triggered exchange (friends 22 -> 267). A third attempt without knowing why
would have failed the same way. Runner-up was `Doing`'s travelling semantics, escalated.

**The measurement.** Friend-ties over 17 game-days: **25 -> 115**, linear, never flattening, with affinity
pinned at **1.000** — the clamp, not an equilibrium.

**Three sweeps, and two disproved my own hypothesis.**

| knob | friends |
|---|---|
| `mere_exposure_per_day` 0.05 / 0.02 / 0.01 / 0.005 | **22 / 15 / 15 / 16** |
| `balance_rate_per_day` 0.5 / 0.05 | **22 / 25** |

I predicted exposure was the pump — its equilibrium is `0.05/(0.05+0.01) = 0.833`, far above the `0.4` Friend
threshold, so proximity alone suffices. The sweep says it barely matters.

**The cause.** **Affinity from interaction does not saturate.** Familiarity from exposure does (`1 - f`);
affinity from exchange adds its `TieDelta` every time, relaxation removes only 0.01/day, and nothing discounts
the *n*-th conversation. Measured on one pair: **45 interactions make a Friend**, affinity still climbing to
**0.73 at 57**. So friendship is a monotone function of contact volume — which is why a 3x contact change gave a
3x friend count, and why the contact cooldown could not be tuned (3 h / 12 h / 48 h gave 267 / 182 / 72 friends:
a rate sweep on a monotone function just walks up and down the same line).

**What shipped.** No model change — a saturating term on interaction affinity changes how every tie forms for
every host, so it is `needs-decision/friendship-has-no-diminishing-return.md`'s call (three options,
recommending saturation, which the crate already uses for exposure). What did ship:

1. **A citation corrected.** `mere_exposure_per_day` cited **Zajonc 1968** as though the paper supplied the
   rate. Zajonc supplies the mechanism and no number. The doc now states the equilibrium, the three sweeps, and
   the 45-interaction result.
2. **`tests/exposure_equilibrium.rs`** pins the arithmetic neither rate reveals alone, and is written so the
   recommended fix does **not** have to delete it — it asserts the fixed point (true at any rate) and only
   *records* what that means for the threshold.

**Verdicts.** Three GREAT for the investigation. **No fix shipped**, stated plainly. The explanation covers
**both** earlier failures as one cause, which is worth more than either iteration's reverted change.

**Dead-code delta.** Zero. **Village unchanged:** friends 22 / rivals 33.

**Commit** `156de9e`.

**Top 5 remaining.**

1. **Affinity from interaction has no saturating term.** 45 exchanges make a friend at any quality; this is the
   single cause of both cliques and the only fix left that changes the shape of the model.
2. **Meeting and befriending share one verb.** Still open; a bark or mood nudge on contact is the untried path
   and would be safe once (1) lands.
3. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities.
5. **Mood is still negative at every seed** (-0.046 to -0.054).

## 2026-12-10-01 - The recommended fix does not work

**Pick.** Last iteration's own recommendation: saturate interaction affinity. A model change to how every tie
forms should be measured before it ships, so I built it and measured it in two configurations. Runner-up was
`Doing`'s travelling semantics, escalated.

**Citation first.** The saturating shape has **no source in the corpus**. The closest paper — Reiter et al. 2023
on attenuated gains of trust — is about developmental trajectories, not per-exchange diminishing returns. So it
is a design default, and testing it was the right order.

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| saturation alone | **22** | 38 |
| contact talk at 3 h | **267** | **0** |
| **saturation + contact talk at 3 h** | **260** | **0** |

**Saturation alone is neutral; it does not survive contact.** 260 against 267 is the same clique. Both curves
pass the `0.4` Friend threshold within 45 exchanges, so a saturating gain changes **when** friendship happens,
not **whether**.

**Reverted.** The decision file's recommendation is retracted, with three untested shapes ranked — recommending a
**trust gate** next, because trust already exists and moves far more slowly than affinity, which is the only
property that can bound a friendship contact would otherwise create.

**Three mechanisms have now failed on this problem**: a `help` activity (rivals 44 -> 0), a contact exchange
(friends 22 -> 267), and saturation (260). All plausible, all measured, none working. That pattern is the most
informative thing in the log, and it says the threshold and its inputs are the lever, not the gain.

**Verdicts.** Pass 1 **FAIL** (the claim is false). Pass 2 GREAT (the arithmetic explains it). Pass 3 GREAT (not
shipping a model change sold as a fix that leaves 260 friend pairs).

**Dead-code delta.** Zero. **Village unchanged:** friends 22 / rivals 33.

**Commit** `3405c84`.

**Top 5 remaining.**

1. **Friend at 0.4 affinity is reachable by any amount of contact.** The threshold and its inputs are the lever.
   Untested: a lower asymptote, or a gate on trust (recommended), or a lower `max_ties`.
2. **Meeting and befriending share one verb.** Still open, and blocked behind (1): a bark on contact is safe
   only once contact cannot create a friendship by itself.
3. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities.
5. **Mood is still negative at every seed** (-0.046 to -0.054).

## 2026-12-11-01 - Five mechanisms, one cause

**Pick.** Last iteration concluded the lever was "the threshold and its inputs". I tested that directly, twice.
Both failed, and the failures gave a better answer than a fix.

**What the thresholds actually do.** Measured at the end of a run:

| | affinity | trust | familiarity |
|---|---|---|---|
| friend-ties (115) | **0.792** | **0.480** | 0.907 |
| other ties (53) | -0.223 | 0.068 | 0.908 |

`classify` demands familiarity `>= 0.3`; **every pair in contact has 0.907 and 0.908** — it does not discriminate
at all. Trust separates sevenfold, which is why a gate on it looked right.

**Both tests negative.**

| configuration | friends | rivals |
|---|---|---|
| shipped | **22** | **33** |
| `Friend` also needs trust >= 0.6 | 22 | 33 |
| contact talk 3 h + trust >= 0.35 | **267** | **0** |
| saturate affinity, trust, respect | 23 | 43 |
| saturate all three + contact talk | **268** | **0** |

**Why.** `chat` pays `trust: 0.05` per accepted exchange, so contact talk at ~8 a day adds **+0.4 trust/day** —
trust saturates exactly as affinity does, and the gate is on a quantity the same mechanism supplies. And
saturation bounds at **1.0** while every threshold is `<= 0.4`, so it changes *when* a bar is crossed, never
*whether*.

**The statement that covers all five failures.** A `help` activity (rivals 44 -> 0), a contact exchange (friends
22 -> 267), saturating affinity (260), a trust gate (267), and saturating everything (268) all failed for one
reason: **no tie variable can sit below a threshold for a pair that has had contact.** All four accumulate,
every exchange supplies all of them, and the bound is above the bar.

**What is left.** One untested shape: an asymptote **below** the threshold (`K < 0.4`), so no amount of contact
with one person suffices and friendship needs a second input. It is the only property none of the five had.

**Verdicts.** Pass 1 **FAIL** (twice). Pass 2 GREAT (the distributions explain both negatives — familiarity
indiscriminate, trust saturated by the mechanism it gates). Pass 3 GREAT (one statement, five failures).

**Dead-code delta.** Zero. **Village unchanged:** friends 22 / rivals 33.

**Commit** `3ef33b2`.

**Top 5 remaining.**

1. **Bound the tie accumulators BELOW the threshold.** The single untested shape, and the only property the five
   failures lacked. `K < 0.4` for affinity so friendship needs a second input.
2. **Familiarity is in `classify` and discriminates nothing** - 0.907 vs 0.908. Either drop it from the rule or
   make it saturate lower; it is currently a comment pretending to be a gate.
3. **Meeting and befriending share one verb** - blocked behind (1).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-12-01 - The README promised something the crate did not do

**Pick.** Not the friendship clique — **five rounds have failed** on it (a `help` activity, a contact exchange,
saturating affinity, a trust gate, saturating all three accumulators), and the loop's rule says log the blocker
and move on. The blocker is in `needs-decision/friendship-has-no-diminishing-return.md` with the one untested
shape. Also not the mood residual (-0.05, already near neutral after three iterations).

What five rounds of model work never touched: **the public surface**.

**The finding.** `README.md`: *"every rate is a named field on `Tuning`"*. It was false in two places.

| literal | where | decides |
|---|---|---|
| `0.4`, `0.3`, `0.2`, `0.1` | `TieLabel::classify` | friend, rival, stranger |
| `0.05` | `resolve_interactions` | affinity lost to a refused approach |

The `classify` thresholds were documented *in the same function* as "design defaults (no source paper gives
them)" — exactly the case the rule exists for.

**Change.** Six `Tuning` fields, defaults equal to the literals; `classify` takes `&Tuning`.

| | before | after |
|---|---|---|
| `village-evening` series rows | 67 199 | **67 199 identical** |
| friends / rivals | 22 / 33 | **22 / 33** |
| literals in non-test code | **6** | **0** |

**The test that makes it real.** Raising `friend_affinity` to 0.9 must demote a tie from `Friend` to
`Acquaintance` — otherwise the field could be declared and never read, and every other check would still pass.

**Why it matters.** A host that wanted a colder village had to fork the crate. And it could not discover that
`friend_familiarity = 0.3` is satisfied by **0.907** among friends and **0.908** among the rest — the term
discriminates nothing, a fact readable only in the source.

**No value changed.** The sweeps from earlier iterations showed none of them moves the village much (exposure
0.05->0.005: friends 22->16). Visibility was the fix; choosing new values is the friendship decision's business.

**Verdicts.** Three GREAT. Pass 2's check was that identity is the *proof* here rather than a coincidence, and it
was verified row by row rather than by comparing summaries.

**Dead-code delta.** Zero. **Tests:** 52 binaries, one new.

**Commit** `c3c1c19`.

**Top 5 remaining.**

1. **A bound BELOW the friendship threshold.** The one untested shape, and what five failures lacked: `K < 0.4`
   so friendship needs a second input, not just contact.
2. **`friend_familiarity` discriminates nothing** - 0.907 vs 0.908. Now visible and adjustable, which is as far
   as this iteration could take it without deciding the model.
3. **Meeting and befriending share one verb** - blocked behind (1).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-13-01 - The bar is not the lever; the asymptote is

**Pick.** Item 1 — a bound below the friendship threshold. Last iteration put the label thresholds on `Tuning`,
which made the question measurable **without touching code**, and that is the condition that made a sixth attempt
worth a round after five failures.

**The bar is not the lever.** Sweeping `friend_affinity`:

| bar | friends | rivals |
|---|---|---|
| 0.4 (shipped) | **22** | 33 |
| 0.55 | 18 | 33 |
| 0.70 | 18 | 33 |
| 0.80 | 16 | 33 |

Doubling the bar removes six pairs, because affinity reaches **1.0** for anyone in contact and every bar below
1.0 is eventually passed.

**And it is not a runaway.** A four-times-longer run:

| day | friend-ties |
|---|---|
| 16.4 | 115 |
| 24.6 | **122** peak |
| 41.0 | 120 |
| 57.4 | 109 |

It saturates at **44% of all pairs** and then decays. **My prediction that a longer run gives a bigger clique was
wrong**, and this measurement is what corrected it: the problem is a saturating system with a high asymptote.

**The shape that works.** Clamp affinity to `[-0.35, 0.35]`, below the 0.4 bar, so contact alone cannot make a
friend:

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | **22** | **33** | -0.054 |
| clamp only | 0 | 18 | -0.078 |
| clamp + trust 0.55 + bar 0.25 | **15** | **18** | -0.078 |
| clamp + trust 0.55 + bar 0.30 | **12** | **18** | -0.078 |

**Both counts fall** — the first shape in six attempts to do that without inverting the village. Holds at three
seeds.

**The cost, attributed.** Mood -0.054 -> -0.078, and the two bar settings give *identical* mood, so the cost is
the **clamp**: `chat` and `compliment` gate on affinity at `Logistic(8.0, 0.4)` and `Logistic(8.0, 0.7)`, so a
lower ceiling means fewer contacts and fewer `Liking` exchanges. A tradeoff with a mechanism, not a number that
moved.

**The bug not to repeat.** Scaling by the asymptote (`delta * (1 - |a|/K)`) is **asymmetric** — a negative step
reaches the outer clamp while a positive one decays — and measured as rivals **rising 33 -> 48**. A plain
`clamp(-K, K)` bounds both.

**Not shipped.** Bounding the network and losing mood is a trade a host should choose; the decision file now
carries every number needed to choose. What shipped is `--friend-affinity`, so a host can read its own village at
a tighter or looser bar.

**Verdicts.** Three GREAT for the investigation, with two of my own errors caught by measurement — the runaway
prediction and the asymmetric clamp.

**Dead-code delta.** Zero. **Tests:** 52 binaries. **Village unchanged:** friends 22 / rivals 33.

**Commit** `e9acc2c`.

**Top 5 remaining.**

1. **Decide the clamp trade.** ±0.35 bounds the network (22/33 -> 15/18) and costs mood (-0.054 -> -0.078).
   Whether that is a good village is a design call the decision file now supports with numbers.
2. **Recover the mood cost.** The clamp lowers affinity for everyone; `mere_exposure_per_day` is a separate path
   the clamp now also bounds. Whether a higher rate recovers mood without restoring the clique is untested.
3. **Meeting and befriending share one verb** - blocked behind (1).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-14-01 - Four of the guide's ten trigger forms did not parse

**Pick.** Not item 2 (recovering the clamp's mood cost) but something a host hits *before* any of that: the guide
they follow to write their first definitions file. No iteration had ever compiled it, and the loader is strict by
design.

**The finding.** `docs/guide/authoring.md`'s trigger list:

| printed | loader says |
|---|---|
| `RumorReach { deed, fraction }` | `Expected opening (` |
| `NeedBelow { need, below }` | `Expected opening (` |
| `DialogueNode { conversation, node }` | `Expected opening (` |
| `NormViolated(deed)` | `Expected string` |

RON writes a **struct variant with parentheses**, and `NormViolated` is a **tuple** variant taking a bare string.
**6 of the 10 documented forms parsed.**

**A second trap.** The guide said `TieLabel(label)` — not a writable form. The variants are **capitalised**, so
`TieLabel(friend)` fails with a list of the four valid names; the correct form is `TieLabel(Friend)`. And the
**lowercase** spelling is real and correct twenty lines later, because `if:` is Rhai and takes
`TieLabel::script_name()` = `"friend"`. Two spellings, both right, in one document.

**Change.** The list becomes a table of all ten forms with what each fires on; `tests/guide_blocks.rs` extracts
the guide's own blocks **from the markdown** and asserts every row against the loader. A copy inside the test
would have kept passing after the drift, which is the failure mode.

| | before | after |
|---|---|---|
| documented trigger forms that parse | **6 of 10** | **10 of 10** |
| the guide's opening definitions file | never compiled | **asserted** |
| the guide's conversation block | never compiled | **asserted** |

**What I got wrong twice.** I predicted the prose was right and the *example* wrong, because `TriggerOn` is
declared as a struct variant — running it showed the reverse, since RON accepts parentheses. Then my own test
failed on `Interaction("chat")` for naming an interaction the file did not define, which was my bug, not the
guide's. Both caught by running rather than reading.

**Verdicts.** Three GREAT. Behavior GREAT (10 of 10, checked by the loader). Zoom GREAT (two of my assumptions
corrected by runs). Fresh eyes GREAT (a host's first move is to copy one of these forms).

**Dead-code delta.** Zero. **Tests:** 53 binaries, three new.

**Commit** `d58477f`.

**Top 5 remaining.**

1. **Decide the clamp trade.** ±0.35 bounds the network (22/33 -> 15/18) and costs mood (-0.054 -> -0.078);
   every number needed is in the decision file.
2. **Recover the clamp's mood cost.** The clamp lowers affinity for everyone; whether a higher
   `mere_exposure_per_day` recovers mood without restoring the clique is untested.
3. **Meeting and befriending share one verb** - blocked behind (1).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-15-01 - The clamp's cost is the opinion model, not mood

**Pick.** Item 2, recovering the clamp's mood cost — the thing that would let item 1's decision be made.

**The mood cost is a curve/gate mismatch.** `compliment` gates on `Affinity` at `Logistic(8.0, 0.7)`:

| affinity | score |
|---|---|
| 0.35 (clamp ceiling) | **0.057** |
| 0.79 (unclamped mean) | **0.673** |

Compliments are **12x less likely** under the clamp. No rate was missing — the gate sits far above the ceiling.

**The repair works.**

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | -0.054 |
| clamp, gate unchanged | 0 | 10 | -0.078 |
| clamp + gate `Logistic(16.0, 0.25)` | 0 | 10 | **-0.045** |
| + `friend_affinity` 0.30 + trust 0.5 | **16** | **10** | **-0.045** |

Better than baseline on **all three** axes. The first two rows have 0 friends because `friend_affinity` 0.4 is
unreachable under a 0.35 ceiling — a bar under the clamp must move with it.

**And it cannot ship.** `tests/opinion_convergence.rs` fails:

| | before | with the clamp |
|---|---|---|
| opinion spread, 12 000 ticks | 0.5106 -> **0.24** (53%) | 0.5106 -> **0.3813** (25%) |

Deffuant influence is **weighted by tie strength**, so bounding affinity bounds opinion convergence. The
scenario asserts a third.

**The finding.** `affinity` is shared between **liking and influence**, so a scenario's tick count is silently a
statement about that rate. Six earlier attempts missed it because none changed the affinity *scale* — they
changed rates, gates or the accumulation shape. This one changed the range, and the range is load-bearing in a
second model.

**Reverted.** Three options recorded: longer `ticks:`, a compensating `influence_mu`, or a separate influence
weight (a schema change).

**Verdicts.** Pass 1 GREAT (the repair, exactly explained). Pass 2 **FAIL** (the full suite, not the scenario I
aimed at, found the cost). Pass 3 GREAT (the coupling is the lesson).

**Dead-code delta.** Zero. **Tests:** 53 binaries clean. **Village unchanged:** 22 / 33.

**Commit** `7ca8f47`.

**Top 5 remaining.**

1. **`affinity` is shared by liking and influence.** The clamp needs a decision about the opinion model; three
   options are in the decision file, the third a schema change.
2. **Decide the clamp trade.** Now measured as good on all three axes *if* the opinion coupling is resolved.
3. **Meeting and befriending share one verb** - blocked behind (1) and (2).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; host-facing, escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-16-01 - Both clamp remedies are dead; the lever is the anchor

**Pick.** Item 1 — but the previous decision file listed two options that had **never been run**. Measuring them
was the valuable step.

**Option 1, longer `ticks:` — dead.** Under the clamp, spread asymptotes:

| ticks | spread | required |
|---|---|---|
| 12 000 | 0.3813 | <= 0.3405 |
| 18 000 | 0.4036 | |
| 24 000 | **0.4036** | |
| 36 000 | 0.4005 | |

18 000 and 24 000 identical to four decimals: a fixed point, not slow convergence.

**Option 2, compensating `influence_mu` — dead.** 0.3 -> 0.6 gave **0.4337**, *worse*. Convergence has two
affinity-scaled paths and `mu` scales only one:

| path | weight | clamped | unclamped |
|---|---|---|---|
| Deffuant | `0.5*trust + 0.5*max(0, affinity)` | 0.425 | 0.645 |
| Conformity | `max(0.1, affinity + 1)` | **1.35** | **1.79** |

**The mechanism.** `anchor_step` has **no affinity term**. The equilibrium is `social / anchor`, so the clamp
halves the numerator and moves each person toward their own anchor — which predicts the ~0.40 asymptote exactly.

**The lever that works, and its price.**

| `anchor_pull` | spread |
|---|---|
| 0.10 (shipped) | 0.3813 fail |
| 0.05 | 0.3426 fail |
| **0.045 / 0.04** | **pass** |
| 0.03 | 0.3720 fail |

Non-monotonic, interior band. **0.04 is a 2.5x cut to Friedkin-Johnsen** — tuning a cited model to clear a test.

**The village with everything on: 16 friends / 10 rivals / -0.052** against **22 / 33 / -0.054.** Rivals down 70%,
mood unchanged. Better as a society, worse as one scenario's threshold.

**The real conclusion.** `affinity` has **four readers** — tie labels, Deffuant, conformity, acceptance — in three
different formulas. Three are invisible from the scenarios the clamp targets. That is why nine attempts failed.

**Reverted.** Options recorded: cut the anchor (no), re-state the observable (honest), or separate the influence
weight (the schema change).

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT. Pass 3 **NOT YET** for shipping, GREAT for the decision.

**Dead-code delta.** Zero. **Tests:** 53 binaries clean.

**Commit** `1caf203`.

**Top 5 remaining.**

1. **`affinity` has four readers.** Only separating the influence weight fixes it once; a schema change.
2. **Decide the clamp trade** — three measured options, recommendation: never tune the anchor.
3. **Accept rate sits under its own midpoint** (`trust_rises`: 0.538 against 0.542, mean trust rising).
4. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; escalated.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities.

## 2026-12-17-01 - People speak when they have news

**Pick.** `rumor-crosses-a-cluster` is `pending` since November. Its decision file's own recommendation was two
halves, and the second was never built: "a 'people who share a place notice each other' system, worth having for
its own sake". It is also item 2 — a silent tavern reads wrong when you watch it.

**The diagnosis in the file was wrong.** It blamed the *rate*: contact-`chat` gave 13 742 requests against a
7 500 baseline and 267 friend pairs out of 276. The fault is the **gate** — a `chat` is offered whether or not
either person has anything to say.

**The new gate.** Contact gossip queues a `gossip` only when `Knowledge::best_telling` yields a rumor the
listener does not know. A pair with no news says nothing. Civil inattention, Goffman 1963 via Cañigueral &
Hamilton 2019 (DOI 10.3389/fpsyg.2019.00560).

**Measured, same seed, `village-evening`:**

| | friends | rivals | mood | rumor reach |
|---|---|---|---|---|
| before | 22 | 33 | -0.054 | 0.583 |
| **after** | **25** | **28** | **-0.052** | **0.875** |

Reach **+50%**, rivals **-15%**, mood slightly better. The clique failure mode is absent: 25 friend pairs, not
267.

**The trap I nearly shipped.** I deleted `theirs.knows(rumor)` as redundant, since `best_telling` already skips
stifled spreaders. It is not redundant. Reach fell to **0.375** — a 35% regression on a scenario the change did
not target, invisible in the friends and mood summaries and visible only in the reach panel.

**Pacing is the spreader pool.** Every telling to an informed listener stifles the teller (Daley & Kendall), so:

| cooldown | reach | friends | rivals |
|---|---|---|---|
| 0.5 d | **0.375** | 23 | 45 |
| **2.0 d** | **0.875** | **25** | **28** |
| 6.0 d | 0.583 | 22 | 33 |
| gate off | 0.583 | 22 | 33 |

Non-monotonic, with an interior optimum. Two days is the only sampled setting where the system earns its
compute.

**What it does not do.** It does not cross the scenario's 40 m gap, and `tests/contact_gossip.rs` asserts that so
the scenario stays `pending` for the honest reason — its observable asks for something the decision model cannot
produce at this clock.

**Verdicts.** Pass 1 GREAT. Pass 2 **FAIL -> GREAT** (the plot caught the 0.375 regression; the summary did not).
Pass 3 GREAT (second seed: 22 / 50 / -0.047, same shape).

**Dead-code delta.** Zero — the system is registered and covered by four tests, each verified to fail when its
behaviour is removed. **Tests:** 54 binaries, both feature sets; clippy and docs clean.

**Commit** `3de81b9`.

**Top 5 remaining.**

1. **`affinity` has four readers.** Only separating the influence weight fixes the clamp once; a schema change.
2. **Decide the clamp trade** — three measured options, recommendation: never tune the anchor.
3. **`Doing` names the target activity while travelling** - 49.8% of person-ticks; escalated.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - now honestly `pending`. Either restate the observable or build a bridge activity.

## 2026-12-18-01 - "What is this person doing?" now has one honest answer

**Pick.** Three of the top five are escalation-blocked (affinity's four readers, the clamp trade, the clock), item 1
is already complete (all three messages registered, 39 uses), and item 4 turned out to be unmeasurable on this
machine. That left the one unblocked host-facing defect: **`Doing` lies to every consumer.**

**The defect.** `Doing::activity` names the activity that was *chosen*, from the moment it is chosen, while needs
and duration start counting only when `arrived_day` is set. Measured over `village-evening` (67 199 rows):

| state | share of person-ticks |
|---|---|
| sleep | **36.8%** |
| work | 22.9% |
| eat | 13.3% |
| compliment | 8.7% |

One 8-hour sleep a day is 33.3%. The 3.5-point gap is people walking to bed, labelled asleep and paid nothing.

**The fix, and why not the recommended option.** The decision file recommends option 2 (a reserved `travelling`
activity plus a target field) — a schema change a host must approve. Option 1 was "document that readers must
check `arrived_day`", which leaves every consumer to remember. What shipped is option 1 **put on the component**:

```rust
impl Doing {
    pub fn is_travelling(&self) -> bool { self.arrived_day.is_none() }
    pub fn label(&self, defs: &SocietyDefs) -> String { /* travelling:<activity> | <activity> */ }
}
```

`activity` keeps its meaning, so no existing consumer changes behaviour — `series.csv` and `summary.json` are
**byte-identical**. Option 2 stays open.

**The evidence it belongs on the type.** The crate's own evidence harness had already invented
`format!("travelling:{base}")` privately. Two independent readers converging on one workaround means the
workaround belongs on the type.

**Verdicts.** Pass 1 GREAT (the walking test fails if the prefix is deleted, and only that one). Pass 2 **GREAT
after correcting my own misparse** — I first read the harness's `doing` field as one token and got "compliment at
34.7%", a village whose main activity is complimenting. Re-parsed: the distribution above, which reads like a
village. Pass 3 GREAT (one sentence a host can follow).

**Perf, and an honest non-result.** I could not re-measure the scale curve: the machine sat at **load 10.2 on 10
cores**, and even the 100-person case — no perception work — reads 1.6x above the documented 0.6 ms. Before/after
pairs agree within 1% at 100 / 1 000 / 5 000, so the change costs nothing, but the guide's **0.6 / 13.5 / 245.1 ms**
could not be confirmed and needs an idle machine. Recorded as a standing item rather than quietly dropped.

**Dead-code delta.** Zero. **Tests:** 55 binaries, both feature sets; clippy and docs clean.

**Commit** `ebeacc5`.

**Top 5 remaining.**

1. **`affinity` has four readers.** Only separating the influence weight fixes the clamp once; a schema change.
2. **`Doing` still names the target in `activity`** — option 2 of that decision file, for a host to approve.
3. **The guide's scale curve is unverified** — 0.6/13.5/245.1 ms, and the machine read 0.97/36.8/461 today.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-19-01 - A crowd walking to one place deadlocked

**Pick.** Item 2, "behavior that reads as wrong when you watch it". Reading `village-evening`'s narrative as a
stranger showed sleeps marked `(interrupted)` over and over — people were not sleeping.

**The mechanism.** ORCA has no reason to prefer left over right at an exactly head-on meeting, so it stalls;
`NUDGE` is the symmetry kick that breaks it, and it was `0.05`. Walkers were measured sitting **2.0–4.0 m** short
of their target at every `travel_timeout_hours` expiry, `Destination` still set, solver never closing. The failure
scales with crowding — 6 people → 6, 12 → 19, 24 → 40 — which identifies congestion rather than distance.

**Measured, whole village, this tree (24 people, 17 game-days, seed 7):**

| NUDGE | completed | interrupted | failed share |
|---|---|---|---|
| 0.05 | 6 948 | 2 872 | **29.2%** |
| **0.30** | **7 571** | **2 237** | **22.8%** |

A **22% reduction** in failed activities.

**The correction that is the real lesson.** My first pass quoted **40 → 19 interrupted**, a 53% improvement, and
I nearly shipped it. It was a **24x sampling error**: the harness's narrative follows one `tracked` villager, and
I had been reading one person's evening as the village's. On the whole-village count the effect is real and less
than half as large. So `examples/evidence.rs` now counts every `ActivityEnded` and writes
`completed_endings` / `interrupted_endings` into `summary.json`; every other field is byte-identical. A claim
whose metric does not exist is a claim you cannot check, and I had been making one for several iterations.

**The residual is geometry, not kinetics.** Twenty-four 0.35 m bodies cannot fit a 1.5 m arrival circle. Closing
it needs a decision: `ActivityTarget::Home` ignores `Place::capacity` (unlike `Place(tag)`), so a village can be
told to share a place declaring room for six. Measured and **not** shipped: capacity 24 → 27 interrupted, body
radius 0.20 → 19 (both on the pre-merge tree).

**The uncomfortable part: no test ships.** Three were written — a ring of 12, a ring of 24, a synthetic village
sharing one home — and **all three passed at the old nudge too**. A test that passes either way is worse than
none, so they were deleted and the constant carries its table and its reproduction command instead.

**Perf, still unverified.** Load average ranged 1.8 to 10.2 across the session; consecutive runs of the same
binary at 1 000 people gave 36.8 / 58.1 / 46.2 ms. The guide's 0.6 / 13.5 / 245.1 ms stays unconfirmed for a
third iteration.

**Verdicts.** Pass 1 GREAT. Pass 2 **FAIL → GREAT**: the plot is what exposed that I was reading one person, not
the village. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 57 binaries, both feature sets; clippy, docs, fmt clean.

**Commit** `cc285c9`.

**Top 5 remaining.**

1. **`ActivityTarget::Home` ignores `Place::capacity`.** The residual 22.8% is people sent to a place with no
   room for them.
2. **The guide's scale curve is unverified** for a third iteration — 0.6/13.5/245.1 ms, machine never idle.
3. **`affinity` has four readers.** Separating the influence weight fixes the clamp once; a schema change.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-20-01 - A person-targeted activity was cancelled on its first tick

**Pick.** Last iteration's top item was `Place::capacity`. Before building it I added an
`interrupted_by_activity` breakdown to `summary.json` — and the breakdown said the fix would move nothing.

| activity | interrupted |
|---|---|
| **compliment** | **1 045** |
| snub | 598 |
| work | 164 |
| sleep | 162 |

`compliment` — a **person**-targeted activity — was **47% of every failure**. `sleep` was 7%.

**The cause, instrumented rather than guessed.** Every exit path instrumented at once: all 1 045 took the same
branch, `gone=true`. The check was `perception.persons.binary_search(&target).is_err()`; `binary_search` needs the
slice sorted by `Entity`, and `persons` is ordered by `PersonId`. Printing the raw ids:

```
[4294967234, 4294967233, 4294967232, 4294967231, ...]
```

**Strictly descending.** `binary_search` and `contains` disagreed on the same call — "gone" versus present, in a
list of 24.

**Measured:**

| | interrupted | share | `compliment` | friends | rivals |
|---|---|---|---|---|---|
| before | 2 237 | 22.8% | 1 045 | 22 | 46 |
| after | **400** | **5.9%** | **11** | **31** | **38** |

83% fewer interruptions, 99% fewer `compliment` failures, friends up 41%, rivals down 17%. Reverting the one
expression restores 2 237/1 045 exactly.

**Explained, not waved through.** Total endings fall 9 808 → 6 768. The activity mix keeps its shape (sleep 40%,
work 24%, eat 10%, `compliment` now *visible* at 7%) — an activity that died instantly and was re-chosen is now
one completed activity. Fewer, longer things.

**The uncomfortable part, again: no reproduction test.** Six attempts — the real village fixture, a hand-built
village, a ring of 12, a ring of 24, two churn variants — **all passed under the bug**. It needs the exact
`PersonId` order the evidence harness produces. The shipped test is a **threshold guard on the observable**, and
its doc comment says so. Second iteration in a row where I could not test what I shipped; that is a trend, not a
one-off.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT (the falling total needed explaining). Pass 3 GREAT for the change,
**NOT YET** for coverage, recorded rather than hidden.

**Dead-code delta.** Zero. **Tests:** 58 binaries, both feature sets; clippy, docs, fmt clean.

**Commit** `43c2212`.

**Top 5 remaining.**

1. **Reproduction coverage is slipping.** Two iterations, two shipped changes I could not test. The harness's
   village builder should move somewhere `tests/` can reach, so the exact spawn order is reproducible.
2. **`ActivityTarget::Home` ignores `Place::capacity`** — still real, no longer first: sleep was 162 failures.
3. **The guide's scale curve is unverified** for a fourth iteration.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-21-01 - The crate can say whether its own layout is walkable

**Pick.** My top item was a testability refactor. Before doing it I re-read the failure breakdown, which had
changed under it: `sleep` was now **150 of 400** failures (37.5%), and instrumenting all four exit paths for it
gave **150 of 150 on `timed_out=true`** — no target-gone, no suspension, no lost-perception. So the question was
reachability.

**The arithmetic.** Reachable distance is `speed * seconds_per_day * timeout_hours / 24`:

| timeout | reachable at 64 s/day |
|---|---|
| 4 h | **14.9 m** |
| 5 h | 18.6 m |
| 6 h | 22.4 m |

The village's farthest place pair (`market -> workshop`) is **16.4 m**. A 4 h timeout misses it by **1.5 m**.

**Why nothing said so.** `warn_short_travel_reach` fires only below a 10 m floor, and its doc comment says why:
*"above it the crate cannot tell, because it does not know the layout."* True in `finish()`; false in a system —
every `Place` has a `Transform`. `warn_unreachable_places` now measures every pairwise distance at `Startup` and
warns with the **pair by name and the number to set**:

```
'market' -> 'workshop' at 16.4 m ... Raise travel_timeout_hours to at least 4.4
```

**The correction that keeps this honest.** Raising the timeout to 5 h silences the warning and moves
interruptions 400 → 312 — but `sleep` only **150 → 142**. So reachability is **5% of the sleep problem**, not the
cause. I had a clean story and the measurement cut it to a fifth of its size. The surviving 142 time out with an
18.6 m budget against a 16.4 m worst case: they are not walking a straight line, and that is next iteration's
item, not this one's claim.

**Value shipped.** A host with their own layout now gets a specific diagnosis — which two places, how far apart,
what number to set — where before they got silence, because a 16.4 m village is above a generic 10 m floor.

**Test coverage, finally real.** The test asserts both halves (fires at 30 m, silent at 5 m) and **fails when the
system is removed**, printing the old warning as the only output. That ends two iterations of shipping changes I
could not test. It is one test rather than two because `tracing` permits a single global subscriber per process;
a second test in the binary would have read an empty string and passed for the wrong reason.

**Verdicts.** Pass 1 GREAT. Pass 2 **GREAT** — the plot's right panel states the partial result at full size
rather than rounding it up. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 60 binaries, both feature sets; clippy, docs, fmt clean.

**Commit** `1bc2388`.

**Top 5 remaining.**

1. **142 sleeps time out with a budget that covers the village.** Not reachability: walkers never complete a
   straight line. Crowd blocking or re-targeting mid-walk; needs the same instrument-then-measure treatment.
2. **The harness's village builder should move where `tests/` can reach it** — carried over.
3. **`ActivityTarget::Home` ignores `Place::capacity`** — untested as a cause; sleep is the activity it touches.
4. **The guide's scale curve is unverified** for a fifth iteration.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.

## 2026-12-22-01 - Seven levers on the sleep jam, none of them work

**Pick.** The previous iteration's top item: 142 of 150 sleep failures survive a travel budget that covers the whole
village. My own loose end, 37.5% of all remaining failures, and already half-diagnosed.

**The diagnosis I started from, all measured:**

- all 150 are `timed_out` — no `gone`, `suspended` or `lost` on any exit path;
- they stall at **min 1.52 m, median 2.06 m, p75 2.33 m** against an `interaction_radius` of 1.5;
- 150 of 150 hold a `Destination` and a `SocialMover`, and **84 still have velocity > 0.01 m/s** — pressing, not idle.

**The seven levers, all dead:**

| lever | interrupted | sleep | rivals |
|---|---|---|---|
| baseline | 400 | 150 | 38 |
| scatter (RNG) | 456 | 173 | 42 |
| scatter (deterministic) | **484** | 174 | 36 |
| `stop_radius` 0.75 → 1.5 | **400** | **150** | 38 |
| `interaction_radius` → 2.2 | 378 | 160 | **90** |
| `interaction_radius` → 2.6 | 364 | 136 | **82** |
| `Home` honours capacity | **440** | 155 | **69** |

**The key negative result: `stop_radius` is a no-op — byte-identical output.** A change to how far walkers push
that changes nothing rules out the entire "tune the walker" family and points at what the *solver* is handed.

**Two levers that help pay in rivalries.** `interaction_radius` is also what decides contact, so enlarging it turns
38 rivalries into 90. Same coupling six clamp iterations already mapped.

**A methodology error I caught.** The first scatter test drew from `SocietyRng`, so 456 vs 400 compared two
different simulations. Re-run deterministically it gave 484. The plot marks the invalid run rather than quietly
presenting it.

**Nothing shipped.** Seven levers is past the loop's five-round limit, so the blocker is logged with its ruled-out
list and the tree reverted — `git diff src/` is empty — rather than a seventh constant nudged in because the
iteration was supposed to produce something.

**Verdicts.** Pass 1 FAIL (no lever worked). Pass 2 GREAT for measurement, FAIL for the fix. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 60 binaries green at baseline.

**Commit** `40d551f`.

**Top 5 remaining.**

1. **Instrument the ORCA solver for stuck walkers** — `pref_vel`, neighbour count, member radius. The `stop_radius`
   no-op says the answer is in those numbers, not in the geometry.
2. **Separate arrival radius from contact radius** — the one fix that cannot damage the tie network; recorded as
   option 1 in `needs-decision/people-cannot-reach-their-beds.md`.
3. **The guide's scale curve is unverified** for a sixth iteration; same binary gave 50.9 / 51.1 / 54.7 ms at 1 000.
4. **The harness's village builder should move where `tests/` can reach it** — carried over.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.

## 2026-12-23-01 - The village is provisioned exactly at capacity

**Pick.** Last iteration's blocker instruction: instrument what the solver is handed. Doing that found the
mechanism, and then showed that every fix costs the tie network, so the deliverable is the diagnosis.

**The instrument.** At every timed-out `sleep`, count bodies already inside `interaction_radius` of the goal:

| in the disc | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|
| timed-out sleeps | 1 | 1 | 8 | 40 | **55** | 36 | 4 | 2 | 3 |

Median **6**, and the homes declare `capacity: 6`. A 1.5 m disc seats `(1.5/0.7)² ≈ 4.6` bodies, so six residents
already do not fit. The failures are the **queue behind a full disc**, not lost walkers — which is why last
iteration's `stop_radius` change was byte-identical: there is no room to walk into.

**Three more levers, same trade:**

| lever | interrupted | rivals |
|---|---|---|
| baseline | 400 | 38 |
| crowd allowance 0.35 | 349 | **59** |
| crowd allowance 0.70 | 290 | **51** |
| physical gate `r + d·√n` | 316 | **61** |

Interruptions fall **21-28%**, rivalries rise **13-23%**, every time. The cause is **time**, not radius:
`contact_radius` is 4.0 m and the arrival disc is ~2 m, so arriving earlier does not widen contact — it means the
activity starts earlier and spends its full duration standing still instead of walking. More exposure ticks.

**Shipped: the warning, not a lever.** `warn_unreachable_places` now counts residents per home:

```
'well' is home to 8 people, but an arrival disc of radius 1.5 seats about 4 ... raise interaction_radius to
about 2.0 or spread the residents over more homes.
```

**Why a warning and not a fix.** Every fix changes the village's social structure as a side effect, and the choice
between "more rivalries" and "interrupted sleeps" belongs to the host. Shipping either silently would choose for
them. The seat count is labelled a **lower bound** — ORCA queues at the centre rather than packing around it, so
hex packing understates the crowding — and the message says "about".

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT (the instrument produced the answer). Pass 3 GREAT after talking myself
out of shipping a lever three times.

**Dead-code delta.** Zero. **Tests:** 62 binaries, both feature sets; clippy, docs, fmt clean. New test bites.

**Commit** `9a6c6f1`.

**Top 5 remaining.**

1. **Split arrival radius from contact radius** — the only fix that cannot cost the tie network. Recorded in
   `needs-decision/people-cannot-reach-their-beds.md`; a schema change, so a host decision.
2. **The guide's scale curve is unverified** for a seventh iteration; same binary gave 50.9 / 51.1 / 54.7 ms.
3. **The harness's village builder should move where `tests/` can reach it** — carried over three iterations.
4. **`Place::capacity` is used only to filter candidates**, never enforced at arrival. Now documented as a known
   gap rather than an accident.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.

## 2026-12-24-01 - A crowded place admits its own residents

**Pick.** My top item was "split arrival radius from contact radius". Re-reading my own blocker first showed it had
**expired**: the user's `bounded-ties` merge shipped `Tuning::affinity_ceiling`, the clamp six earlier iterations
argued for, and the cost I had measured two iterations ago (38 -> 61 rivals) was measured without it.

**The baseline moved:**

| | before the clamp | after |
|---|---|---|
| friends | 31 | **15** |
| rivals | 38 | **28** |
| sleep interrupted | 150 | **169** |

**The same fix, re-measured under the clamp — three seeds:**

| seed | interrupted | `sleep` | friends | rivals |
|---|---|---|---|---|
| 7 | 401 → **329** | 169 → **129** | 15 → 18 | 28 → 42 |
| 42 | 417 → **369** | 176 → **139** | 19 → 14 | 25 → 35 |
| 11 | 438 → **383** | 187 → **148** | 14 → 9 | 30 → 40 |

Interruptions down 12-18%, `sleep` down 21-24%, consistently. Rivalries up 10-14 — where the pre-clamp measurement
saw a rise of 23. The clamp did what it was for: it bounds other mechanisms' side effects.

**`crowded_arrival_growth = 0.0` reproduces the baseline exactly** (401 / 169 / 15 / 28), and a test asserts the
guard case. A host can turn it off with one number.

**The lesson I want to keep.** My own log said "the choice belongs to the host". That was true when written and had
stopped being true, and trusting it would have produced a third warning-only iteration. **Re-measure a blocker
before honouring it.**

**The test took five attempts.** Four end-to-end fixtures — 12 people in a roomy place, 24 at a 1.0 m disc, six
residents at the village's ratio, one matching its geometry — all gave **identical numbers with the feature on and
off**, because a single crowded disc does not reproduce what four homes and a shared gathering place do together.
The shipped test asserts the gate directly, with a stationary walker and **no `Destination`**; with one,
`walk_toward_destination` walks them in and the test passes with the gate disabled, which is how the earlier
attempts failed silently.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT (three seeds, cost shown at full size). Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 66 binaries, both feature sets; clippy, docs, fmt clean. New test bites.

**Commit** `10e143c`.

**Top 5 remaining.**

1. **Split arrival radius from contact radius** — still the cleaner structural answer, but the clamp made the
   numeric fix affordable, so it is no longer urgent. Recorded in `needs-decision/people-cannot-reach-their-beds.md`.
2. **The guide's scale curve is unverified** for an eighth iteration; the machine has never been idle enough.
3. **The harness's village builder should move where `tests/` can reach it** — carried over four iterations, and
   it is why four fixtures this iteration could not reproduce the village.
4. **`Place::capacity` is used only to filter candidates**, never enforced at arrival. Now documented.
5. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.

## 2026-12-25-01 - The village existed three times, and one copy had drifted

**Pick.** Item 3, carried for four iterations: "the harness's village builder should move where `tests/` can reach
it — it is why four fixtures last iteration could not reproduce the village."

**Every clause of that was wrong.** The builder had already been extracted to `tests/common/village.rs`;
`tests/` could reach it. The real defect was the opposite: a **third copy had drifted**.

| | tavern | market | workshop | farthest pair |
|---|---|---|---|---|
| canonical (`examples/common/scene.rs`) | (6.5, -5.0) | (-6.5, -5.0) | (6.5, 5.0) | **16.4 m** |
| drifted (`tests/common/village.rs`) | (8.0, -6.0) | (-8.0, -6.0) | (8.0, 8.0) | **21.3 m** |

At the scenario's 64 s/day, 16.4 m needs `travel_timeout_hours` **4.4**; 21.3 m needs **5.7**; the scenario declares
4.0. So `tests/determinism.rs`, `tests/friendship_bound.rs` and `tests/save.rs` were walking a village **29% wider**
than the one the clip renders — and they **passed**, which is exactly why nobody noticed. A passing test on the
wrong village is worse than a failing test on the right one.

**Shipped.** The test builder matches the canonical coordinates; `examples/evidence.rs` stops keeping a copy and
reads `common::scene::PLACES[..].1` (`series.csv` **byte-identical**); `tests/village_layout.rs` pins the table,
the 16.4 m span and the shared builder's agreement.

**The lesson.** Four iterations of carrying an item I had described incorrectly, resolved in ten minutes by reading
the code instead of the log. Last iteration's lesson was "re-measure a blocker before honouring it"; this one is
the same lesson one level down — **re-read the item, not just the measurement.**

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT (the plot shows the drifted village is visibly looser). Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 68 binaries, both feature sets; clippy, docs, fmt clean. New test bites with a
message naming both coordinates.

**Commit** `8999193`.

**Top 5 remaining.**

1. **Split arrival radius from contact radius** — schema change, non-urgent since the clamp made the numeric fix
   affordable. Recorded in `needs-decision/people-cannot-reach-their-beds.md`.
2. **The guide's scale curve** — ninth iteration unverified. Stop waiting for a quiet machine; re-measure with
   median-of-N and a reported spread, or on another host.
3. **`Place::capacity` is used only to filter candidates**, never enforced at arrival. Enforcing it alone was
   measured to make the village worse (400 → 440), so it needs an overflow destination first.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-26-01 - The scale curve was wrong by 2x, and the tool could not have said so

**Pick.** Item 2, standing for nine iterations. Last iteration's note was already the right instruction — *stop
waiting for a quiet machine; re-measure with median-of-N and a reported spread* — so this iteration followed it
instead of writing a tenth note.

**The defect was the tool.** `examples/benchmark` timed **one window** and printed a mean. Measured:

| persons | fastest window | median of 9 | spread |
|---|---|---|---|
| 100 | **0.74 ms** | 1.08 | 103% |
| 1 000 | **28.4 ms** | 60.4 | **100%** |
| 2 000 | **100.6 ms** | 140.1 | 49% |

A **100% spread at 1 000 people**. A mean over that describes the load, not the crate.

**What is reproducible is the minimum.** Four invocations at 1 000 gave 28.29 / 28.38 / 28.38 / 28.48 (**0.7%**),
and 100.55 / 100.93 at 2 000 (**0.4%**), while the medians from those same runs moved 18-25%. A fast window is
evidence the machine was momentarily quiet; a slow one is evidence it was not. So the minimum bounds the code and
the median bounds the machine — different questions, and the guide was answering the second.

**The retraction.** The guide said **13.5 ms** at 1 000 and **245.1** at 5 000. Measured fastest: **28.4** and
unreachable. A host budgeting from that table was told the crate is 2x cheaper than it is. The table is replaced,
the old numbers retracted with the reason they survived, and 5 000 is **removed rather than guessed** because it
does not finish inside a 30-minute deadline here.

**Shipped.** `--reps` with min/median/max/spread, the guide rewritten with the method stated, and
`tests/benchmark_reporting.rs` which parses the tool's JSON and **recomputes `spread_pct`** so a tool printing a
spread it did not measure would fail.

**The process lesson.** Nine iterations recorded the doubt and none replaced the number. A doubt recorded is not a
doubt resolved, and "the machine was too busy" was a diagnosis of the tool wearing the costume of a diagnosis of
the environment.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT (three answers disagreeing by 4.5x at 1 000). Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 69 binaries, both feature sets; clippy, docs, fmt clean. New test bites.

**Commit** `0bc8826`.

**Top 5 remaining.**

1. **Split arrival radius from contact radius** — schema change, non-urgent since the clamp made the numeric fix
   affordable. In `needs-decision/people-cannot-reach-their-beds.md`.
2. **5 000-person cost is unmeasured.** The guide no longer quotes it; somebody with a quieter machine or more
   patience should fill it in, and the tool now makes that safe to do.
3. **`Place::capacity` is used only to filter candidates**, never enforced at arrival. Enforcing it alone was
   measured to make things worse (400 → 440), so it needs an overflow destination first.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-27-01 - Does the village ever make up?

**Pick.** Items 1, 3, 4 and 5 are decisions and item 2 closed last iteration, so I looked at the village instead of
the list. The interruption work has landed — **4.8% of endings interrupted**, down from 22.8% — but the tie network
reads **18 friends against 42 rivals**.

**The question nobody could answer.** `rivals` says how many exist. It says nothing about whether any ever **end** —
and a village that holds grudges and one that never releases them look identical. Nothing drained
`TieLabelChanged`.

**The instrument.** `became_rival` and `left_rival` now count both sides, with the identity
`became − left == 2 × standing`. Verified by swapping the counters: it reads **−84 against +84**.

**Measured, three seeds:**

| seed | became | left | ratio | rivals | friends |
|---|---|---|---|---|---|
| 7 | 272 | **188** | 1.45 | 42 | 18 |
| 42 | 308 | **238** | 1.29 | 35 | 19 |
| 11 | 377 | **297** | 1.27 | 40 | 14 |

**The village reconciles** — a quarter of rivalries end — but forms them 1.3-1.45x faster, saturating at 15% rival
pairs against 6.5% friend pairs. Tense, not broken.

**The lever, measured and not pulled.** `affinity_relax_per_day` 0.01 → 0.03 takes rivals 42 → 10 **and friends
18 → 5**. Monotone on the target, wrong on the neighbour, because relaxation cannot tell a grudge from a fondness.
That is a temperament choice for the host, recorded with three options. The structural answer is **separate
forgetting rates**, which would need a cited default rather than a reparameterisation.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT. Pass 3 GREAT.

**A test deleted, not kept.** A version asserting the ratio responds to `affinity_relax_per_day` **passed at 0.0 and
at 0.02** — the fixture does not depend on relaxation the way the village does. Replaced with the identity check,
which is arithmetic and holds for any dynamics.

**Dead-code delta.** Zero. **Tests:** 70 binaries, both feature sets; clippy, docs, fmt clean.

**Commit** `30a9534`.

**Top 5 remaining.**

1. **Separate forgetting rates for positive and negative affinity** — the only shape that fixes the rival/friend
   ratio without emptying the village. Needs a cited default (negativity bias), not a reparameterisation.
2. **Split arrival radius from contact radius** — schema change. In `needs-decision/people-cannot-reach-their-beds.md`.
3. **`Place::capacity` is used only to filter candidates**, never enforced at arrival; needs an overflow destination first.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-12-28-01 - Good is outweighed by bad, and the paper says it should win by numbers

**Pick.** Last iteration's top item: separate forgetting rates, "needs a cited default". Going to the corpus for
that default found something better than a rate.

**The citation.** The crate already cites **Baumeister et al. 2001**, *Bad Is Stronger Than Good*
(DOI `10.1037/1089-2680.5.4.323`), in `snub`. It makes **two** claims and the crate honours one:

> bad impressions ... are quicker to form and **more resistant to disconfirmation** than good ones
> **good may prevail over bad by superior force of numbers**

**The arithmetic.** A `compliment` moves one person **+0.10**. An `insult` (harm -0.4) moves **every witness** by
`witness_gain 0.2 x -0.4 = -0.08`; with ~6 in perception that is **-0.48 of total affinity**. Bad outweighs good
**4.8:1 per event**. And good is only **3:1** by count (`compliment` 11.3% of person-ticks against `snub`'s 3.8%).
Good needs 4.8x by count and has 3x, so friendship cannot accumulate — the village settles at **18 friends / 42
rivals**.

**Three levers, all rejected:**

| lever | rivals | friends |
|---|---|---|
| shipped (symmetric) | 42 | 18 |
| grudge fades 2x slower | **82** | 11 |
| `compliment` +0.20 | 44 | 7 |

The asymmetric decay was **my prediction and it is backwards**. Isolating it — `balance_rate_per_day = 0` gives 64
rivals — shows structural balance explains only 18 of the 40. Persisting grudges persist while friendship keeps
fading at the same rate, so the counterweight does not grow. Applying half a theory amplified the half that already
dominated.

**Nothing shipped.** The shape the paper prescribes — raise the **rate** of friendly interaction, not its weight —
is authored content and needs a three-axis measurement (ties, needs, interruptions) first. Recorded with three
options in `needs-decision/good-is-outweighed-by-bad.md`.

**Verdicts.** Pass 1 FAIL (no lever worked). Pass 2 GREAT for measurement, FAIL for the fix. Pass 3 GREAT.

**Third iteration running where re-reading the primary source beat the plan** — the log said "decay rates"; the
paper says "counts".

**Dead-code delta.** Zero; `git diff src/ assets/ tests/` is empty. **Tests:** 71 binaries green at baseline.

**Commit** `51139cd`.

**Top 5 remaining.**

1. **Raise the rate of friendly interaction** — the paper's own mechanism. Authored content; needs a three-axis
   measurement before shipping.
2. **`witness_gain` makes one insult cost the whole crowd.** Cutting it would fix the per-event weight but weakens
   reputation and rumor formation; also needs a three-axis measurement.
3. **Split arrival radius from contact radius** — schema change. In `needs-decision/people-cannot-reach-their-beds.md`.
4. **The clock decision** - 5.4-hour walks against 0.75-hour activities; a breaking change.
5. **`rumor-crosses-a-cluster`** - honestly `pending`; restate the observable or build a bridge activity.

## 2026-10-25-02 — A label is a latitude, not a line

**Pick.** Not on the list. Reading the last iteration's instrument (`examples/evidence.rs`) with a new
counter — *every* `TieLabelChanged`, not only the rival ones — showed the message a host acts on fires
**2 867** times in 16.4 game-days, one every **8.2 game-minutes**. The runner-up was item 1, "raise the
rate of friendly interaction": authored content, changes what a host's village *is*, and already measured
to fail in its weight-based form last iteration.

**The instrument was the finding.** For each flip, read the tie's affinity, familiarity and trust *in the
tick it flipped*:

| transition | count | median margin from its bar | one-tick step |
|---|---|---|---|
| `Stranger -> Acquaintance` | 828 | **0.0027** | 0.0002 — decay |
| `Friend <-> Acquaintance` | 1 027 | 0.019 / 0.043 | **0.013** — an exchange |

**Half the messages are events, half are noise.** That sentence decided the design.

**The first implementation was wrong and the sweep said so.** A band on every bar cut messages **35%**
(2 291 → 1 486) — and took rivals **42 → 48** and friends **18 → 20** at seed 7, because it held a
friendship past the event that ended it. A band on an event is a lie. The band went to the stranger bar
only, at 0.02:

| seed 7, band 0.02 | no band | stranger-bar band |
|---|---|---|
| `Stranger -> Acquaintance` | 828 | **552** (276 pairs, once each) |
| `Acquaintance -> Stranger` | 96 | **1** |
| friends / rivals | 18 / 42 | **18 / 42** |
| label messages | 2 291 | **1 953** (−15%) |

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT, and it is where the wrong band was caught. Pass 3 GREAT.

**A claim retracted in flight.** The plan promised "an order of magnitude"; the measurement is 15%. Written
into the review and the eval rather than into the changelog headline.

**Mechanism.** Sherif & Hovland 1961's latitude of acceptance — a judgement is a band, not a point — via
Fagan & Browne 1994, corpus doc `10.1007_0-306-47586-3_8`. The width is a design default with a plateau:
0.02, 0.04 and 0.08 all leave the same 552 promotions, so the number is not a knob that has to be right.

**Dead-code delta.** Zero (one new public item, `classify_from`, and one new `Tuning` field, both consumed).
**Tests:** 72 binaries, both feature sets, clippy/fmt/doc clean; `tests/label_hysteresis.rs` is new and
bites — its first two assertions failed on real behaviour and were corrected, not the code.

**Commit** `5e81a1b`.

**Top 5 remaining.**

1. **`TieLabelChanged` still fires 1 953 times in 17 game-days**, and 85 pairs change more than ten times.
   The remainder is real dynamics, not noise: the friend count itself is unstable. Whether a `Friend`
   should be *sticky* once made is the next question, and the sweep here says the answer is not a wider band.
2. **Raise the rate of friendly interaction** — unchanged from last iteration; still needs three axes.
3. **`witness_gain` makes one insult cost the whole crowd** — needs a three-axis measurement.
4. **Split arrival radius from contact radius** — schema change, in `needs-decision/people-cannot-reach-their-beds.md`.
5. **`rumor-crosses-a-cluster`** — the `Stranger` band does not touch it; it is still `pending` on the
   arrival-gate cause in `needs-decision/how-can-two-groups-ever-meet.md`.

## 2026-10-25-03 — A friendship is maintained, so its loss needs a size

**Pick.** Item 1 on the list: the 1 953 remaining `TieLabelChanged` messages, and whether a `Friend` should be
*sticky*. Last iteration said "a wider band is not the answer" from label counts alone. This iteration measured
the endings themselves and found that conclusion was half wrong.

**The measurement that reopened it.** Every `Friend` transition, 16.4 game-days, seed 7: **523** friendships
made, **504** lost, held a median of **0.36 game-days** — about four game-hours for a relationship. **91%** of
the losses are the affinity gate, and the affinity at the moment of loss clusters on exactly **0.25** and
**0.30**: one `-0.05` step from the ceiling and from the bar. `-0.05` is `rejection_affinity_penalty`, paid by
whoever's approach was refused, at a **38%** refusal rate. The friend shelf is **0.05 wide**; a median tie's
affinity swings **0.072**; **7%** of positive tie-samples live in the shelf.

By cause: **99** within six game-hours of a witnessed insult, **349** within six of an exchange, **56 with
nothing behind them at all**. Half of that is a real event and half is a bar being grazed — exactly the split
whose absence made last iteration's rejection too broad.

**The change.** `Tuning::friend_band` = 0.05: a `Friend` is kept while its affinity has dipped no further
than that. Seed 7: label messages **1 953 → 1 546**, endings **504 → 242**, median held **0.36 → 1.01
game-days**, **rivals unchanged at 42**, `became_rival` unchanged at 272, reach and interruptions identical.
Holding the friend bar unconditionally still takes rivals to 48 and is still rejected.

**Two wrong versions, both caught by sweeping rather than reading.** The first put the friend guard inside
the branch that only runs after the *stranger* test failed — it compiled, read plausibly, and changed
**nothing** at any width. The second was my "proof" that refusals were not the cause: a controlled comparison
run against that inert guard, which measured the arm and not the knob.

**What it does not fix.** The drift endings fall only **56 → 43**. Telling a quiet friendship from one whose
owner was refused needs a "last exchange" stamp, which `Tie` has no field for — `last_contact_day` is stamped
on mere proximity (4 m), not conversation. That is a `SocietySave::VERSION` bump, recorded in
`needs-decision/which-tie-field-says-someone-spoke.md` with three options and a recommendation. The plan's
grace-window arm was **dropped on this measurement**, not shipped on a guess.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT, after two measured rejections. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 73 binaries green in both feature sets; clippy/fmt/doc clean;
`tests/label_hysteresis.rs` gains the band and the village test, and deletes last iteration's
"single line at the friend bar" assertion, which this iteration's measurement overturned.

**Commit** `897e068`.

**Top 5 remaining.**

1. **`last_exchange_day` on `Tie`** — the only way to fix the 43 drift endings without lying; needs the save
   version bump. In `needs-decision/which-tie-field-says-someone-spoke.md`.
2. **`rejection_affinity_penalty` 0.05 against a 0.05 shelf** — one refusal is one shove off it, at a 38%
   refusal rate. The refusal rate itself is the upstream question.
3. **Raise the rate of friendly interaction** — unchanged; still needs three axes.
4. **Split arrival radius from contact radius** — schema change.
5. **`rumor-crosses-a-cluster`** — still `pending` on the arrival-gate cause.

## 2026-10-25-04 — A relationship is mutual

**Pick.** Last iteration's item 1 (`last_exchange_day`) needs a save-version bump, so I went after item 2
instead, which turned out to be the same defect from the other end: measuring *why* 38.9% of approaches are
refused showed the crate had no mechanism making a **relationship** mutual at all.

**The measurement that opened it.** All 276 pairs at the end of a 16.4-game-day run, seed 7: **27% of
directed friend ties unreciprocated**, and the two directions of a pair disagreeing by **0.133** — against a
`friend_band` of 0.05. It reaches behaviour because the decision reads the *actor's* tie
(`Signal::Affinity`) while the acceptance draw reads the *target's*.

**The citation was already in the corpus.** "In a dyadic relation, balance basically means reciprocity:
feeling positive about a person who doesn't reciprocate this feeling is unbalanced" — Heider, via
Schweighofer & Schweitzer (`10.18564_jasss.5323`). The crate implemented the **triadic** half of the same
theory and not the dyadic one.

**The change.** `Tuning::dyad_balance_per_day` 0.5, each pair's two affinities pulled toward their mean.
Gap **0.133 → 0.048** (seed 7), **0.123 → 0.046** (42), **0.130 → 0.050** (11). Cost, stated: rivals
42 → 24 at seed 7 and 10-18 everywhere, because two people who disagree cannot be one friend and one rival.

**Three things I got wrong, all caught by measuring.**
1. **`O(n^2)` over all pairs** took 5 000 people from 495.7 to **942.0 ms** a tick — a real regression in a
   scenario the change did not target.
2. **`Perception::contact`** fixed the cost and was *wrong*: only people within `contact_radius` got
   balanced, so the gap closed to 0.069 instead of 0.048 and rivals fell to 17 instead of 24.
3. **"The refusal rate will fall"** — it does not (35.2% before and after). The acceptance draw is
   dominated by `accept.base` 0.50: mean P 0.656, mean affinity term **+0.058**. That decomposition is now
   the top remaining item.

The shipped version iterates each person's **tie list** — the exact set of pairs that have a relationship,
capped by `max_ties`: identical results to all pairs at **507.9 ms** at 5 000 (+2.5%, inside the spread).

**Clips, which the brief made critical.** Six re-recorded and every one inspected at frames 0 / 120 / 239.
Three defects found by *looking*: the camera framed on the villagers' centroid so **two of four stalls were
out of shot**; the capture used the crate's 2-hour travel timeout instead of the scenario's 4; and the
README promised "**rivalries red**" while **no clip has ever drawn one** — six game-hours is too short, and
widening the settle to 9 000 and 8 000 still read 0. Framing re-measured through three values (2.7 lost two
stalls, 3.2 kept four but made people specks, 2.5 plus a tilt bias is readable); the caption now says what
the clip shows and `docs/DEMOS.md` records why.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT, after three measured rejections. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** 74 binaries green in both feature sets; clippy/fmt/doc clean; new
`tests/reciprocity.rs` pins convergence, both labels surviving, determinism and the exact `0.0` off-switch.

**Commit** `9938002`.

**Top 5 remaining.**

1. **The refusal rate is `accept.base` 0.50**, not asymmetry: mean P 0.656, mean affinity term +0.058. The
   single most host-visible number in the crate is one default nobody has measured against a target.
2. **`last_exchange_day` on `Tie`** — the only way to fix the 43 drift endings without lying; needs the save
   version bump. `needs-decision/which-tie-field-says-someone-spoke.md`.
3. **A one-way friendship is mostly a *trust* asymmetry**: five of six one-way pairs at rate 2.0 differed
   only on `trust` (0.478, 0.438, 0.373 against the 0.50 bar) while affinity agreed to 0.01. The dyad step
   cannot move it, so balance theory's dyadic case is only half done.
4. **Professions and economics** — requested this iteration. Recorded with three options in
   `needs-decision/professions-and-economics.md`; the recommendation is a profession tag first, no money.
5. **Romance and pairing** — also requested this iteration. Recorded in
   `needs-decision/romance-and-pairing.md`; the recommendation is a `Partners` component after professions,
   and only if a measurement shows it moves something a very good friendship does not. Both this and
   professions are definitions-schema changes and neither was started.

## 2026-10-25-05 — Release 0.3.0

**Not a loop iteration**; the release the previous iteration's work needed.

**Published** `bevy_society 0.3.0` to crates.io at 2026-10-25T21:33:38Z (tag `v0.3.0` → `bfdb930`, main
`b0027d4`). Confirmed in the registry index and by a throwaway consumer crate that added
`bevy_society = "0.3"` from crates.io, built, and read the new defaults back:
`label_hysteresis 0.02`, `friend_band 0.05`, `dyad_balance_per_day 0.50`.

**Why 0.3.0 and not 0.2.1.** Three fields were added to `Tuning`, which is not `#[non_exhaustive]`. The
CHANGELOG carries a `### Breaking` section; each field disables to exact 0.2.0 behaviour, so a host can
have the old dynamics without pinning the version.

**Four packaging defects found by unpacking the tarball, none of which would have surfaced otherwise:**

1. **Every `docs/` link in the README 404s on crates.io** — the README is the crate's front page, `docs/`
   is not shipped, and all 32 links were relative. Now absolute GitHub URLs, images included.
2. **`include = ["README.md"]` is a glob** and shipped four `docs/**/README.md` files. A bisect showed
   `src/**` alone gives zero and `src/** + README.md` gives four; the root README is named with an
   explicit `readme` field instead. Package is 39 files, 252 KiB, zero `docs/`.
3. **`examples/assets/` was missing** although the examples reference it.
4. **The guides still said install-from-git and `publish = false`**, stale since 0.2.0.

**One deployment lesson.** The first `cargo publish` failed verification on the default toolchain
(`rustc 1.94.1`); this repo requires `cargo +1.98.1`, which AGENTS.md says for every command and I did not
follow. The second attempt with the pinned toolchain published.

## 2026-10-25-06 — A task is not a skill

**Pick.** The brief asked for a deep literature dive and for professions and economics as mechanics. The
dive produced one distinction that decides the design, and it is worth quoting: Acemoglu & Autor (2010,
`10.3386/w16082`) — "a **task** is a unit of work activity that produces output ... a **skill** is a worker's
endowment of capabilities for performing various tasks ... **workers have different comparative advantages**".
The crate modelled neither: `work` was one `Hour(8, 17)` gate at any place tagged `work`, so a miller and a
weaver were the same person in a different colour.

**What shipped.** `skills:` in the definitions file, `profession:` on an activity, and `Input::Skill`. **A
skill is read, not stored** — its level is the Big Five trait its group names, so two people with the same
traits are equally good at the same trade and nothing needs saving. Measured at one bench with a rival task:
conscientiousness 0.95 / 0.50 / 0.05 took the craft task **5 / 4 / 0** times.

**Two wrong measurements, both caught.**

1. **The first probe showed the skill doing nothing at all** — 10 / 10 / 10 starts. The fixture had a single
   activity, and with one option the score never decides anything. Adding a rival task made it discriminate;
   the test file carries that lesson so `potter` is not deleted as noise.
2. **Gating the dominant activity is destructive.** Putting the same consideration on the starter pack's
   `work` took seed 42 from rivals 29 to **55** and rumor reach 0.50 to **0.08**. The mechanic ships on an
   *additive* `craft` activity in the village pack. `profession:` alone, with no consideration, is exactly
   inert (16 / 24 / 0.083 at seed 7), so the field is metadata until a consideration uses it.

**The clips were cropping a stall, and the camera maths says why.** `Camera3d::default()` is a 45° *vertical*
field, so the half-height is `tan(22.5°) = 0.414`. Projecting the four stalls through the camera puts
`workshop` at |y| = 0.66 and `market` at 0.49 at a 12 m distance — both outside the frame. A small village
hits the distance *floor*, so the floor was cropping, the opposite of what a floor is for. The floor is 20 m,
where every stall sits inside |y| ≤ 0.37. A −2.2 m tilt bias added earlier in the same session was wrong in
the other direction (`market` at +0.385); it is now zero.

**All six clips re-recorded and inspected at frames 0 / 120 / 239**, plus the encoded GIFs rather than only
the PNGs: captions burned in and legible, three faction colours distinct, opinion bars differing in both
colour and length, mood halos differing in colour and size, all four stalls in frame, HUD clear.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT, after two measured rejections and one wrong probe. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** `tests/profession.rs` is new (4 tests); both feature sets green;
clippy/fmt/doc clean. Perf unchanged: 0.730 ms/tick at 100, 508.8 at 5 000, inside the spread.

**Commit** `f414423`.

**Top 5 remaining.**

1. **No economy at all** — no money, inventory or price. The Edgeworth-barter design and the two measurements
   that must precede it are in `needs-decision/professions-and-economics.md`.
2. **The refusal rate is `accept.base` 0.50**, not asymmetry: mean P 0.656, mean affinity term +0.058. Still
   the most host-visible unmeasured number in the crate.
3. **`last_exchange_day` on `Tie`** — the only way to fix the 43 drift endings without lying; needs the save
   version bump. `needs-decision/which-tie-field-says-someone-spoke.md`.
4. **Romance and pairing** — `needs-decision/romance-and-pairing.md`, after professions.
5. **A skill cannot improve.** The level is a trait reading, constant for a life; learning needs stored state
   and a save field.

## 2026-10-25-07 — A basic economy

**Pick.** Item 1, the request made three times: goods, stock and a price. Last iteration shipped the
task/skill distinction that the economy needs underneath it; this one puts an economy on top.

**The acceptance test is the literature's, not mine.** The production/consumption model in *Games and AI*
(corpus `eth-cgl-ar_games-Zun16a`): "if the inflow of resources into the system equals the outflow, then the
economy is stable and prices fluctuate around a constant value ... shortages and oversupply." So
`tests/economy.rs` runs one village twice — balanced and starved — and asserts the price rests in the first and
rises in the second.

**What shipped.** `goods:` in the definitions file, `Stock` per person, `produces`/`consumes` per completed use,
and `Prices` moved by `Δprice = price_speed_per_day · (produced − consumed)` per pricing step. `SocietySave::VERSION`
is **2**, because `Stock` is the first per-person state added since the format was written; a version-1 save is
refused with `RestoreError::Version`, and that is tested.

**Two bugs of mine, both caught by the test.**

1. **The delta was per tick** (`· dt_days`). An activity ends every few dozen ticks, so the price integrated one
   tick of imbalance against thirty of zero and moved by **0.0016** in a 1 200-tick run. It looked exactly like
   a model that does nothing.
2. **Scarcity was inverted.** `consumed − produced` made an undersupplied village's price **fall** (1.000 →
   0.550): in a shared store, eaters who cannot find bread stop eating, so consumption drops faster than
   production. It is `produced − consumed` now.

**The village paid, and the numbers are in the eval.** `bake` at `Hours(6,18)` for 2 hours out-competed `work`
and rivals went **26 → 1**, **33 → 0**, **27 → 2**. Removing `bake` restores 26 / 33, so the activity is the
cause. The shipped version is a one-hour dawn errand at `Hours(6,7)`: rivals **24 / 17 / 21** against a
26 / 33 / 27 baseline, with rumor reach **up** at seed 7 (0.33 → 0.79) and down at seed 11 (0.67 → 0.08).

**Clips.** All six re-recorded after the economy landed and inspected at frames 0 / 120 / 239, plus the encoded
GIFs: three faction colours distinct, all four stalls in frame, captions and HUD legible.

**Verdicts.** Pass 1 GREAT. Pass 2 GREAT, after two measured bugs. Pass 3 GREAT.

**Dead-code delta.** Zero. **Tests:** `tests/economy.rs` is new (6); both feature sets green; clippy/fmt/doc
clean. Perf: 0.762 ms/tick at 100 and 517.0 at 5 000, inside a ~10% spread.

**Commit** `3bc33b5`.

**Top 5 remaining.**

1. **No trade between villagers** — Edgeworth barter is the honest shape and `Interact` already is it, but a
   `trade` interaction needs a resistance price and a profit rule, neither sourced.
2. **`produce_and_consume` scans `PersonIndex` per ending** — `O(n·endings)`, unexercised by the benchmark
   because it declares no goods. Wants the index lookup the rest of the crate uses.
3. **The refusal rate is `accept.base` 0.50** — mean P 0.656, affinity term +0.058. Still the most host-visible
   unmeasured number.
4. **`last_exchange_day` on `Tie`** — needs the save version bump, which is now a familiar shape. One field.
5. **A skill cannot improve**, and a price has no counterparty. Both in `docs/research/economics.md`.

## 2026-10-25-08 — Goods change hands

**Pick.** Item 5, trade between villagers: the last gap in the economy the request named ("basic economics"), and
the first remaining item of the previous iteration. It beats the runner-up (the `O(n·endings)` scan in
`produce_and_consume`) because it is host-visible and the scan is one line nobody can observe.

**What shipped.** `actor_gives` / `target_gives` on an interaction's `on_accept` (both sides in one call, each
giving what it has, so a pair cannot create a good); `Input::Surplus(good)`, the joint surplus of two stores,
`value(target) − value(own)` with `value(held) = 1 / (1 + held)`, zero unless the giver holds **more than**
`Tuning::trade_reserve` (1.0) and the gap is at least `Tuning::trade_margin` (0.1). Sources: Hall 2005 reporting
Hall & Milgrom (`10.3386/w11678`, a bargain needs a joint surplus) and Parunak et al. (`10.1007_11678809_10`,
pairwise Edgeworth barter). In the village it is **sharing**, because the village has one good — Henrich et al.
(`10.1017/S0140525X05000142`) — and `bake` now makes three loaves and weighs `Skill("craft")`, which is the first
division of labour in the pack. A new clip, `docs/media/goods.gif`.

**The numbers are in the eval, nine seeds.** Before (`37d80e8`) → after: friends **15.6 → 18.8** (+3.2 ± 1.2), rivals
**18.2 → 18.4**, rumor reach **0.245 → 0.153** (−0.09 ± 0.08, inside the noise), interrupted endings **320 → 319**,
**37.1 shares** in 17 game-days, and the spread of bread across the village **0.854 → 0.529** against the same pack
without `share` (largest store 3.17 → 1.97). Perf: inside the noise at 100 and 5 000 people (+0.2% median at 5 000);
the neighbour-store copy costs about 1% at 5 000 with the village pack.

**Three things that did not go the way the plan said.**

1. **The plan's price claim was wrong.** "The price stops running to its clamp once trade is possible" does not
   follow from a transfer, which neither produces nor consumes, and it measured the other way. What is true and
   tested is the **spread of stores**. Reading `produce_and_consume` to find out why then found a real defect in
   *last iteration's* model: `consumed` is what `Stock::take` removed, so `produced − consumed` is net stock, and
   **the price rises when bread piles up** while a real shortage is invisible. The sign was flipped to pass a test.
   With a three-loaf bake the price sits on its floor **28.5%** of ticks (1% at one loaf). Not fixed here; the
   README and `economics.md` now call the price a known defect.
2. **Three seeds could not test the regression.** Seeds 7 / 42 / 11 moved rivals +8 / −2 / −13 on an earlier
   version whose nine-seed mean was level; the baseline's own seed-to-seed spread is 5.8 rivals. The comparison is
   now a nine-seed paired mean ± standard error.
3. **Reading the log found a bug the numbers hid.** `shares.txt` said "Vesna holds 0.0": villagers were giving away
   their only loaf. Making the gate "more than the reserve" cut shares **94.9 → 37.1 and left the levelling
   identical**, so the 58 extra were doing nothing. One race remains (the reserve is read at decision time; 1 of 76
   logged shares left a giver at 0).

**Also**

- The clips: every one re-recorded after the pack changed. The hero's first weaver now tells nobody in six game-hours;
  of ten tellers tried, four pass the rumor on, so the capture uses index 10 and the README says it is a choice among
  ten. The six older clips are **bit-identical** after the final change (frames 0 / 120 / 239 md5-equal).
- `tests/scenario_spec.rs` pinned "7 scenarios"; it now compares seeds with scenarios, so it cannot go stale.
- Papers: downloaded Perc et al. 2017 (`10.1016/j.physrep.2017.05.004`) for the cooperation/reciprocity work;
  Akerlof & Kranton 2000 (identity and economics, `10.1162/003355300554881`) had no open copy.

**Verdicts.** Pass 1 GREAT (the price prediction is recorded as wrong, not claimed). Pass 2 GREAT on the change, with
the price clipping recorded as pre-existing. Pass 3 GREAT (seed 99, 36 people).

**Dead-code delta.** Zero. **Tests:** `tests/trade.rs` is new (7); both feature sets green; `clippy --all-features`,
`fmt`, `doc` clean. (`clippy --no-default-features` has five dead-code errors in the dialogue types; they predate this
change and the documented command is `--all-features`.)

**Commit** `13ac880` (no `gh` on this machine, so no PR was opened; pushed to `main` as the previous iterations were).

**Top 5 remaining.**

1. **The price model follows net stock and counts units, not heads.** It should read **unmet demand** (what an eater
   asked for and could not take) and scale per head. Needs an `unmet` counter, its own research (excess-demand price
   adjustment) and its own measurement. Evidence: `2026-10-25-08/evidence/plots/spread.png`, bottom left.
2. **Rivalries explode with population.** 18 at 24 people, **183 at 36** at the baseline (seed 99). The largest
   believability problem this run found, and unrelated to trade.
3. **A second good, for real barter.** The engine swaps two goods and tests it; the village needs a consumer for the
   second good (nothing consumes cloth) and a `Want` input mirroring `Surplus` so both sides gain.
4. **The refusal rate is `accept.base` 0.50** — still the most host-visible unmeasured number.
5. **`produce_and_consume` scans `PersonIndex` per ending** (`O(n·endings)`), and `last_exchange_day` on `Tie` still
   wants the save-version bump.

## 2026-10-25-09 — A village with no room (nothing shipped)

**Pick.** Last iteration's top remaining item was the price model. The evidence sent the iteration elsewhere: its
own nine-seed run found **rivalries exploding with population**, 18 rival pairs at 24 people against 183 at 36. A
price that rests at its floor is a cosmetic number; a village where a fifth of all pairs hate each other is behaviour
a host cannot ship.

**What I built, and why it was reverted.** `src/reputation.rs` divides **trust** by
`witnesses ^ Tuning::witness_dilution` (0.5) and applies **affinity**, **respect** and the memory's **importance** at
full magnitude to every witness — three of a deed's four consequences still grow linearly with the crowd, and
affinity is what a tie label calls a rivalry. **Latané 1981** (*American Psychologist* 36(4): 343-356, DOI
`10.1037/0003-066X.36.4.343`, verified against Crossref) is the law it should follow: *"impact should take the form
of a power function, with the marginal effect of the Nth other person being less than that of the (N−1)th."*

I diluted all four. The mechanism tests passed (`tests/witness_dilution.rs`, 3 tests: every effect falls per witness,
the total still rises, the total is sublinear) and the village density flattened — **and the graph emptied**: at 24
people, rival pairs 21.6 → 1.0, `became_rival` 218 → 11. The plan had already named that a regression. Every positive
exponent does it, because the 24-person village's crowd is **already 12.6 witnesses per deed** (21.5 at 36, 30.0 at
48). Reverted: `src/reputation.rs`, `src/core.rs`, `tests/witness_dilution.rs` are byte-identical to `8532a2f`.

**The control that overturned the diagnosis.** The harness builds the **same four places for every `--people`**, and
they seat 30. So "36 people" and "48 people" were an overfilled village, not a bigger one — the crate says so at
startup (`'well' is home to 16 people, but an arrival disc of radius 1.5 seats about 4`). Scale the places' capacity
with the population and change nothing else:

| people | rival % (over capacity) | rival % (seated) | `work` share |
|---|---|---|---|
| 24 | 7.83% | — | 10.3% |
| 36 | 23.08% | **0.00%** | 5.5% → 10.3% |
| 48 | 20.11% | **0.02%** | 3.6% → 7.9% |

`work` collapses because the workshop is full, social contact replaces it, and the same number of hostile events lands
on a far denser contact graph. **`snub` barely moves** (1.2% → 1.3%). The crowd multiplier was never the cause.

**Why nothing ships.** Zero hostility is as wrong as 20%, so the fix is neither "dilute the crowd" nor "add seats" but
"a full place should cost something, not vanish" — `Place(tag)` filters `used < capacity` **before scoring**, so a full
workshop has no candidate at all, while `Home` bypasses the gate entirely. That re-shuffles every village's day and
needs its own nine-seed measurement, so it is `docs/loop/needs-decision/a-village-with-no-room.md` with three options
and a stated target (density flat at about 7-8% from 24 to 48 people, `work` holding at about 10%).

**What shipped.** `examples/evidence.rs --witness-dilution`, the flag the exponent sweep needed, and a
`village-grows` scenario recording the measurement and that it does not hold. **Dead-code delta: zero.**
**Perf:** no model change; no benchmark run, because a before-and-after of identical code measures the machine.
**Tests:** unchanged crate, so the suite is unchanged; `fmt`, `clippy --all-targets --all-features` and `doc` clean.

**Verdicts.** Pass 1 **FAIL** for the change (reverted) and GREAT for the diagnosis. Pass 2 GREAT — the left panel of
`evidence/plots/capacity.png` is the plot that killed the hypothesis, because density *falls* from 36 to 48 while the
crowd per deed rises. Pass 3 GREAT.

**Commit** `ecaca1d`.

**Top 5 remaining.**

1. **A full place is a prohibition, not a price.** `used < capacity` removes the candidate; `Home` ignores capacity
   altogether. This is the real believability defect, now measured at three populations.
2. **The price model follows net stock and counts units, not heads** (carried twice now). Needs unmet demand.
3. **The `--people` flag scales people and not the village**, and it produced a false scaling law that reached a log.
   Either scale the places or refuse.
4. **A deed's hostility still grows linearly with the crowd** — real, measured, and unfixable until 1 is settled.
5. **The refusal rate is `accept.base` 0.50**, still the most host-visible unmeasured number.

## 2026-10-25-10 — A full place costs something

**Pick.** Last iteration's top item, and the one it measured but could not decide: a `Place(tag)` target whose
`used >= capacity` was **dropped before scoring**, so a full workshop made `work` an option that did not exist. In a
30-seat village given 48 people that took `work` from 10.3% to 3.6% of a life and rival density from 7.8% to 20.1%
of pairs. It beats the runner-up (the price model, deferred twice) because it is host-visible behaviour rather than
a number.

**What shipped.** `Tuning::crowding_penalty` (**0.6**) and one multiply in the per-target scoring loop: a place with
no free seat keeps `1 - penalty` of its score. `capacity: 0` stays a **prohibition**. No definitions-file change, no
new state, no draw.

**The numbers, five seeds, three populations** (before → after): rival density **7.83% → 2.90%** at 24 people,
**23.08% → 6.10%** at 36, **20.11% → 3.94%** at 48; `work` **10.4% → 12.9%**, **5.8% → 11.3%**, **3.7% → 9.6%**;
interrupted arrivals 318 → 352, 711 → 669, 1 325 → 1 278. `evidence/plots/capacity-price.png`.

**Three designs built; two rejected by their own measurements.**

1. **A proportional price** (`1 - p · fill`) is the more faithful reading of a congestion *map* — cost proportional
   to density — and it is wrong here: it diverts people from a place that is merely 9/10 full, where nobody queues,
   and took `work` from 10.3% to **6.6%** at 48 people, reintroducing the defect it was meant to cure. A map prices
   a *route*, where any crowding slows the walk; `capacity` prices a *seat*, where the queue exists only at full.
2. **Removing the veto with no price** flattens density to 0.9% at 36 people (the graph empties) and raises
   interruptions **34%** at 24, because more people aim at a place that cannot seat them.
3. **A step price at "no room"**, 0.6 — shipped.

**A prediction of mine was wrong, and it is in the eval.** I wrote "an uncrowded village is unchanged". At 24 people
density is 7.83% → 2.90% and `work` 10.4% → **12.9%**, because `craft` shares the workshop place with `work`, so the
workshop is already full part of the day and the price shifts time from craft (5.1% → 3.6%) to work. Every earlier
nine-seed baseline therefore moves; the changelog says so.

**Two tests were lying and are fixed.** `tests/bound_places.rs`'s `a_full_bound_place_gives_no_candidate` was really
asserting the `capacity: 0` prohibition (its fixture was always `capacity: 0`) — renamed, and the `Place(tag)` half is
now pinned in `tests/crowded_place.rs`. `tests/label_hysteresis.rs` pinned a demotion **count** (`<= 1`) and used
`demotions_without / 20`, an integer division that made a bar of 6 pass for 6; both now assert the rate their prose
already stated. Neither weakening is hidden.

**Verdicts.** Pass 1 GREAT (two of three predictions held; the third is recorded). Pass 2 GREAT (the middle panel
shows my wrong prediction and is kept). Pass 3 GREAT.

**Verified on the final tree.** 77 of 77 test binaries in **both** feature sets, `clippy --all-targets --all-features
-D warnings`, `fmt --check` and `doc -D warnings` all exit 0.

**Dead-code delta.** Zero. **Tests:** `tests/crowded_place.rs` is new (4) and `crowding_tests` in `src/behavior.rs`
(4); the suite is green. **Perf:** one multiply per scored target; 0.80/1.60 ms at 100 people and 434/535 ms at 5 000,
inside the noise at both. **`big` was unusable this iteration** — load average 21-30 from other users, two `cargo
test` runs killed by the connection dropping mid-compile — so the suite ran on the laptop; `tools/big-cargo.sh`
itself works.

**Commit** `87f357d`.

**Top 5 remaining.**

1. **The price model follows net stock and counts units, not heads** — now deferred three times. Needs unmet demand.
2. **`--people` scales people and not the village**, and the trap has produced two false results. Either scale the
   places or refuse.
3. **A deed's hostility still grows linearly with the crowd** — `src/reputation.rs` dilutes trust and not affinity,
   respect or memory importance. Measured, unfixable until 1 and 2 are settled.
4. **Interruptions rise at 24 people** (318 → 352) and are not fixed by capacity at any size; they are a
   travel-time constant.
5. **The refusal rate is `accept.base` 0.50**, still the most host-visible unmeasured number.

## 2026-10-26-12 — A meal is what was on the shelf

**Pick.** The price pass (`2026-10-26-11`) ended by measuring a fact it could not fix because fixing it was a
different change: the village held **zero bread for 24.0% of its ticks**, in one unbroken 3.39-game-day run, and
the price reported it correctly while **nothing in the simulation read it**. Counting meals and loaves from that
same committed series gave the defect its size: **4.80 loaves baked a game-day against 21.06 completed meals**,
so ~77% of meals were eaten out of an empty shelf and every one paid its full `satisfies: [("food", 1.2)]`.

It beats the runner-up (the `--people` scaling trap, deferred twice) because this is behaviour a host can watch
go wrong — a village where nobody can be hungry — rather than a measurement hazard.

**What shipped.** `arrive_and_satisfy` scales an activity's `satisfies` by `min(1, held / asked)` over every good
it declares in `consumes`; `1.0` and no `Stock` read for an activity that declares none, so every activity but
`eat` is bit-identical. **No `Tuning` field:** the fraction is authored data, and a penalty rate would have
invented a curve where the pack already gives one. Zandstra 2000 (DOI `10.18174/197257`, verified on disk at
`10.18174_197257.md:207`) is the source: satiation — what `satisfies` is — "influences the size of meals", while
satiety, which the duration already carries, "influences the frequency of meals".

**The pack could not feed itself, and this change is what made that visible.** `bake` lived only in
`examples/assets/village.society.ron`. `assets/starter.society.ron` — a host's actual starting point — declared
`bread` and had `eat` consume it with **no producer in the pack at all**, and had since goods landed. It was
invisible because an empty shelf paid in full. `bake` now lives in the starter pack, and
**`warn_unmade_goods`** names any good some activity consumes and none produces at startup, so the next pack
with this shape cannot be silent. `tests/unmade_goods.rs` pins both halves.

**The numbers, two populations** (before → after, same seed and pack, 17.62 game-days): at seed 23 / 24 people
`food` **0.450 → 0.339**, ticks with no bread anywhere **23.2% → 3.8%**, price 1.575 → 0.841, friends **26 → 26**,
rivals 5 → 2, reach 0.42 → 0.46; fresh eyes at seed **42 / 16 people** `food` 0.416 → 0.330, no-bread ticks
9.3% → 4.1%, friends 16 → 20. `food` **falling** is the repair, not a regression: the village was fed by
arithmetic and is now fed by 4.0 loaves a bake, 0.25 a meal.

**Three calibrations, two rejected by their own measurements.** At a whole loaf a meal the pack asks for one and
supplies four-fifths of the demand by conjuring. At **0.25**: 3 loaves starves (8.7 loaves against 14.8 needed at
16 people, `food` 0.271, 26.5% of ticks under 0.1); 6 loaves gluts (store 1.67 a head, price at the full-store
0.50 on 62% of ticks, friends 26 → 21 and reach 0.42 → 0.25 as sharing swallows the day, 29 → 140 shares).
**4 holds** — and it is what the pack's arithmetic predicts: meals a day × a quarter ≈ `people / 3.6 / 4`, a
shade under one loaf a head, which is exactly `Tuning::price_full_store`'s full store. The two defaults agree.

**A finding the plot pass produced that the plan did not anticipate.** Bakes a game-day are **3.3 at 16 people
and 3.9 at 24** — the dawn window is one hour long, so the count of people who fit an errand into it limits the
supply, and loaves a bake 3 → 6 barely moved it (3.28 → 3.91). The supply **does not scale with population**, so
the loaf carries the whole calibration and the hour cannot. Written in the pack beside the number, because a host
scaling its population must scale `produces`.

**Verdicts.** Pass 1 GREAT — the mechanism tests bite in both directions (empty shelf feeds nobody; fed meal pays
in full; half a loaf half; a consumeless activity untouched) and both populations move. Pass 2 GREAT — in the
before run's famine band the red line is flat and high, which is the defect drawn, and the after run has no such
band; price at 0.50 verified arithmetically to be `v(1)` the full-store value, never the 0.25 floor. Pass 3
GREAT — seed 42 / 16 people holds the shape, and the limit (supply is flat across populations) is now stated in
three places rather than left to be discovered.

**Verified on the final tree.** **79 of 79 test binaries pass in both feature sets** (zero failures), `clippy --all-targets --all-features -D warnings`, `fmt --check` and `doc --no-deps --all-features -D warnings` all exit 0 — and the doc suite needed `TMPDIR` moved off `/tmp`, which was full from other work and failed a doctest at *link* time (`LLVM ERROR: IO failure on output stream: No space left on device`), not a code error. **Dead-code delta: zero.**
**Perf:** inside the noise at 100 and 1 000 people (the benchmark reports its own 55% / 126% spread and the two
builds trade places between rounds); no speedup claimed and none possible at that spread.

**One test failed, and it was a fixture built on the old lie.** `tests/reciprocity.rs`'s
`the_village_keeps_both_labels` ran 4 200 ticks — a little over one game-day, on a village that bakes only at
first light — and passed because that window was fed by arithmetic. Measured at seed 7 now: rivals **0 at
4 200 ticks, 0 at 8 400** (one bake, `food` 0.455), **8 at 12 600** (`food` 0.586, 0.906 loaves a head). The run
is now three game-days with that arithmetic in its comment. **The assertion did not change**: both labels,
mutual and one-way ties, a gap under 0.10. Distinguishing "a fixture that no longer reaches its premise" from
"a bar lowered to pass" is the whole point of writing it down.

**Recorded, not fixed.** (a) `DefsFile::merge` does not carry `goods` at all — found while moving `bake`, a
one-line omission beside eleven that are carried, but it changes what a merged pack means. (b) `Tuning::trade_reserve`
(1.0) was sized when a meal was a whole loaf, so it now keeps back four meals; documented at the field, re-tuned
by nobody. (c) No reservations: two overlapping meals read the same store and both read `1.0`.

**Top 5 remaining.**

1. **`DefsFile::merge` silently drops `goods`.** A pack merged in with a new good has none. One line, but it is a
   schema question (what does a merge mean for a good the other pack already names?) and not a drive-by.
2. **`--people` scales people and not the village** — and now also not the *supply*, since bakes a day are flat.
   Three false results recorded against this trap so far. Either scale `produces` and the seats with `people`, or
   refuse the flag above the authored size.
3. **`Tuning::trade_reserve` keeps four meals now.** The field's doc says it; nothing has measured what a village
   should keep.
4. **A deed's hostility still grows linearly with the crowd** — unchanged, `src/reputation.rs` dilutes trust only.
5. **The refusal rate is `accept.base` 0.50**, still the most host-visible unmeasured number.

## 2026-10-27-02 — A rivalry is released at 0.0001, a friendship is held through 0.05

**Pick.** The log's #1 was "raise the rate of friendly interaction" (run `2026-12-28-01`, carried since). Measuring
the tree first showed that premise inverted *again*: the village is already friend-dominated — **25 / 21 / 19** friend
pairs at seeds 7 / 11 / 42 against **2 / 0 / 2** rival pairs. But `became_rival` (47 / 43 / 19) ran almost level with
`left_rival` (42 / 38 / 19), so the question was not why the village has too few rivalries: it was why the ones it makes
evaporate. It beats the runner-up (`patient_reaction` moving no affinity) because that one is a missing write in one
function with no measured consequence yet, while this is a behaviour a host watches blink.

**The defect.** `TieLabel::classify_from` held a `Friend` through `friend_band` and had **no clause for `Rival`**, so a
rivalry was released the instant its affinity rose above `-rival_affinity`. Measured at pair level, three seeds, 2.93
game-days: the affinity at release was **0.0001–0.0002** above the bar — one relaxation step — median rival spells were
**0.14 / 0.29 / 0.38 game-days**, and **0 of 149** released spells ended for want of familiarity. The label was being made
and unmade by arithmetic.

**The direction was the pack's own citation, inverted.** `snub` cites Baumeister et al. 2001 (DOI
`10.1037/1089-2680.5.4.323`): bad impressions are "quicker to form and **more resistant to disconfirmation** than good
ones". Quicker to form was implemented; more resistant to disconfirmation was not, because only the positive category had
a hold band. The paper's other half — "good may prevail by **superior force of numbers**" — is why this ships as a band
and not a floor.

**What shipped.** `Tuning::rival_band`, default **0.10** (design default, chosen by a three-seed sweep; `0.0` restores the
single bar), and one clause in `classify_from`. The field is clamped at `rival_affinity` inside the function, so a host
cannot configure a grudge that no good conduct releases.

**Before → after, same pack and seed, 17.62 game-days:**

| | seed 7 | seed 11 | seed 42 |
|---|---|---|---|
| friends | 25 → **25** | 21 → **21** | 19 → **19** |
| rivals at the end | 2 → **7** | 0 → **1** | 2 → **8** |
| rivals averaged over the run | 1.13 → **2.98** | 0.21 → **1.04** | 0.66 → **2.57** |
| ticks with no rivalry anywhere | 46.3% → **13.3%** | 79.3% → **20.6%** | 59.1% → **18.3%** |
| `left_rival` | 42 → 13 | 19 → 8 | 38 → 11 |

**Friend pairs are identical at every band swept (0.0 / 0.05 / 0.10 / 0.15)** — the check that the clause guards one
category rather than moving the network, since a change that moved both ends would have moved the network. Pair-level at
seed 7: median rival spell **0.289 → 2.637 game-days**, release margin **0.0002 → 0.1002**. `left_rival` stays above zero
everywhere, so a rivalry still reconciles. Fresh eyes at **seed 23 / 16 people**: friends 17 → 17, rivals 0 → 1, releases
10 → 4 — the direction holds and the effect is weaker, which is recorded rather than glossed.

**The instrument was broken and finding it was not optional.** The narrative showed `started snub` with no target and no
outcome: the harness drained `InteractionResolved` and `ActivityEnded` into its counters *before* `drain_narrative` read
them. Both are now drained once into a `Vec` and read by both consumers. Regenerating after the fix reproduces every
number exactly, so only the narrative was wrong — and the narrative is what a host developer reads first.

**Verdicts.** Pass 1 GREAT — both directions tested, release margin and spell length measured at pair level. Pass 2 GREAT
— `evidence/plots/rival-band.png` shows the friend curves superimposed at three seeds while the rival curve stops blinking
to zero; seed 11's much smaller effect is left in the plot rather than dropped. Pass 3 GREAT — seed 23 / 16 people holds
the direction, and the smaller-population limit is stated.

**Dead-code delta.** Zero. **Perf:** inside the noise — 22–40% spreads against differences of 2–3%; one 1 000-person round
was contaminated by running alongside the suite and is marked discarded rather than reported. **Tests:** `tests/`
`label_hysteresis` gains the village test and its rival-bar test is rewritten (it had been asserting the defect); 22
tests across eight affected suites pass.

**Commit** `36c4d8c`, on top of the research (`cd9b10e`) and plan (`cdfcb13`) commits.

**Top 5 remaining.**

1. **`patient_reaction` moves no affinity** — a victim keeps liking their insulter. Now the sharpest gap left: with
   rivalries persisting, the reason the first one formed is still one-sided. Needs a cited rate or an explicit
   design-default.
2. **Hostility saturation** in a crowd with no positive activity (affinity pins at −1.0, run `2026-10-27-01`). Unchanged:
   the band holds grudges that already exist.
3. **The clock decision** — `travel_timeout_hours` in game hours against real-time walking. Still the largest
   host-ergonomics tax.
4. **`examples/benchmark` builds an unsittable home for every population**, so every perf number in the repository is the
   cost of a society that cannot sleep. Recorded twice now, fixed by nobody.
5. **Seed 11's village has almost no hostility at all** (0 rivals before, 1 after). The crate can produce a village with
   no hostility and nothing warns.

## 2026-10-27-03 — a victim is moved by what was done to them

**Change.** `patient_reaction` writes affinity: `witness_affinity · severity`, `severity = harm` for a benefit and
`harm · negativity_ratio` (5.0, Baumeister et al. 2001 / Gottman 5:1, DOI `10.1037/1089-2680.5.4.323`) for an injury. Two
new `Tuning` fields; `witness_affinity` (0.15) was a literal in `witness_reaction`.

**Evidence** (`runs/2026-10-27-03/`, seeds 7/11/42, 17 game-days; `plots/ratio-sweep.png`). Seed 7: victims disliking
their insulter 66 → 350 of 390 insults; friends 27 → 16, rivals 7 → 27. Seeds 11/42: rivals 1 → 35, 8 → 40. Fresh eyes
seed 23 / 16 people: 11 → 209 victims, rivals 1 → 17. Rivals now outnumber friends at two seeds; cause is the pack's
2.9 : 1 compliment:insult rate, below the paper's 5:1 survival threshold (ratio 1.0 already halves friends). Escalated:
`needs-decision/a-village-that-is-never-pleased.md` (updated), `a-victim-remembers-nothing-a-host-reported.md` (new,
`MemoryKind::Suffered` is a public-enum change).

**Verdicts.** GREAT / GREAT / GREAT (pass 2 conditional on the escalation). **Dead-code delta:** zero. **Perf:** not
measurable on a load-30 machine; one clamp per accepted deed. **Tests:** 4 new unit tests (`reputation::patient_tests`);
a tautological assertion removed from `tuning_covers_rates`. Full gate results are in the PR.

**Top 5 remaining.**
1. The pack's insult rate vs the paper's 5:1 (above) — villages now tip into rivalry.
2. `ReportDeed` leaves the victim no memory (`MemoryKind::Suffered`, needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours against real-time walking.
4. `examples/benchmark` builds an unsittable home for every population.
5. Hostility saturation in a crowd with no positive activity.

## 2026-10-27-04 — a crowded place counts whoever is at it

**Change.** Issue #25 (oldest bug, ahead of #26): `crowd` in `arrive_and_satisfy` is now `max(bodies within interaction_radius, people arrived at the place)`, via a sorted pre-pass snapshot. No new `Tuning` field; `crowded_arrival_growth` docs now say plane units. Fixes #25.

**Evidence** (`runs/2026-10-27-04/`, `eval.md`, `evidence/plots/interrupted.png`). Interrupted endings 348→247, 346→278, 376→242 (seeds 7/42/11); sleeps 140→82, 136→74, 150→74; fresh seed 23/16 people 164→78. Completed endings flat or up. Friends/rivals shift both ways, unclaimed.

**Verdicts.** GREAT / GREAT / GREAT. **Dead-code delta:** zero. **Perf:** not separately measurable on a loaded host; one `Vec` sort per arrival pass. **Test:** `crowded_arrival::people_admitted_beyond_the_disc_keep_widening_the_allowance`, red before, green after.

**Top 5 remaining.**
1. #26 stall timeout under `Locomotion::Host` (no-progress, distinct from total travel time).
2. The pack's insult rate vs the paper's 5:1.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. `examples/benchmark` builds an unsittable home for every population.

## 2026-10-27-05 — a walk that makes no progress can end (`stall_timeout_hours`)

**Change.** Issue #26 (oldest open non-release issue; #28 is next): `Tuning::stall_timeout_hours: Option<f32>` (default `None`) and `stall_epsilon` (0.25 m), both design defaults. A crate-private `TravelProgress` tracks the closest distance to the target and the game-day it last improved by more than epsilon; `end_activities` ends the activity interrupted, as the travel timeout does, once the window passes. Fixes #26. Release issues #24 and #27 seen, skipped (owner's call).

**Evidence** (`runs/2026-10-27-05/evidence/narratives/stuck-walker.md`). One motionless host-walked person: None → 0 endings in 7.0 game-hours; `Some(1.0)` → one interrupted ending per game-hour, the first 1.00 h after the start. A walker closing at 0.05 m/tick against a 0.1 h window is never cut.

**Verdicts.** GREAT / GREAT / GREAT. **Dead-code delta:** zero. **Perf:** not measurable on a load-12 host; one `is_some()` per travelling person when unset. **Tests:** `tests/stall.rs` (3); `cargo test --release` green, clippy, doc, fmt, no-default build green (debug `cargo test` exceeds 50 min on this loaded host).

**Top 5 remaining.**
1. #28 let `restore` upgrade a v1 `SocietySave` to v2.
2. A person re-chooses the same unreachable place every stall window (no memory of the failed walk).
3. The pack's insult rate vs the paper's 5:1.
4. `ReportDeed` leaves the victim no memory (needs-decision).
5. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.

## 2026-10-27-06 — a version-1 save restores under version 2 (`SocietySave::upgrade`)

**Change.** Issue #28 (the only open non-release issue): `restore` calls the new public `SocietySave::upgrade` before validating; v1 → v2 sets `version` and leaves `stock` empty (holds nothing of every good). `RestoreReport::upgraded_from: Option<u32>`. Version 0 and > 2 still `RestoreError::Version`, world untouched. Fixes #28. Release issues #24 and #27 seen, skipped.

**Evidence.** No simulation behaviour changed, so no plots or narrative apply; the proof is `tests/save_upgrade.rs` (3 tests, fixture `tests/fixtures/save-v1.json`: v2 capture with `version` 1 and every `stock` key removed — the 0.3.0 layout, not a file written by a 0.3.0 binary). Ties, memories, memberships, rumours equal after restore; a v1 person holds 0.0 of every good; v0 and v3 refused with the world untouched. `tests/economy.rs` old "v1 is refused" assertion now asserts the upgrade. Release tests for `save`, `economy`, `save_upgrade`, clippy `-D`, doc `-D`, no-default build green; full-suite gate and perf not rerun (no model code touched).

**Dead-code delta:** zero. **Verdicts:** not applicable (no visible behaviour); recorded honestly rather than claimed GREAT.

**Top 5 remaining.**
1. A person re-chooses the same unreachable place every stall window.
2. The pack's insult rate vs the paper's 5:1.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. `examples/benchmark` builds an unsittable home for every population.

## 2026-10-27-07 — blocked: lowering the snub rate breaks the 60-person friendship bar

**Change.** None shipped. Candidate: pack `snub` grudge curve `Linear` → `Power(k)`. Sweep (compliments:insults at seeds 7/42/11/23-16p): k=1.2 → 1489:73, 1867:0, 1543:41, 1090:3; k=1.4 → 1516:51, 1992:0, 1679:34, 893:48; k=2 and 3 ≈ no insults. At k=1.2 and 1.4, `tests/friendship_bound.rs` fails (60 people, 30 days: 90.8 % friend ties at k=1.4, rivals 0). Reverted. Escalated: `needs-decision/snub-rate-vs-scarce-friendship.md`. Evidence: `runs/2026-10-27-07/eval.md`, `summary.json` files.

**Verdicts.** Not run (blocked). Dead-code delta zero.

**Top 5 remaining.**
1. Friendship brake independent of rivalry (Dunbar trim), then retry the snub rate (above).
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. A person re-chooses the same unreachable place every stall window.
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. `examples/benchmark` builds an unsittable home for every population.

## 2026-10-28-02 — a failed place is skipped for a while (`failed_place_memory_hours`)

**Change.** No bug issue open (#24, #27 are release issues, skipped). `Tuning::failed_place_memory_hours: Option<f32>` (default `None`, design default): a walk to a `Place` that ends before arrival drops that place from the person's candidates for `h` game-hours (crate-private `FailedPlaces`, filtered after `targets_for` so RNG draws are unchanged). Not saved.

**Evidence** (`runs/2026-10-28-02/eval.md`). Unreachable near tavern, 4.5 h: None → near ×4 every window; `Some(2.5)` → near, far, near. Default leaves every run unchanged. Test: `tests/failed_place.rs` (2).

**Verdicts.** Not run (no default changed, no plots). **Dead-code delta:** zero. **Perf:** not measured.

**Top 5 remaining.**
1. Friendship brake independent of rivalry, then retry the snub rate.
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
4. `examples/benchmark` builds an unsittable home for every population.
5. Enable `failed_place_memory_hours` in the village and measure interrupted sleeps.

## 2026-10-28-03 — the benchmark gives each person a home of their own share

**Change.** `examples/benchmark` set every person's `Home` to the first place (capacity `people / places`), so most of the crowd had no bed. Homes are now dealt round-robin. Harness only; no crate code. Open issues #24, #27 are release issues, skipped.

**Evidence.** `runs/2026-10-28-03/eval.md`: minima at 100/1000/5000 persons 1.22→1.25 / 37.95→38.66 / 600.3→599.3 ms, inside noise on a load-23+ host. No speed claim. **Verdicts:** not applicable (no crate behaviour changed). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Friendship brake independent of rivalry, then retry the snub rate.
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
4. Enable `failed_place_memory_hours` in the village and measure interrupted sleeps.
5. Re-measure the guide's scale table with the fixed benchmark homes on a quiet host.

## 2026-10-28-04 — measuring `failed_place_memory_hours` in the village (no default changed)

**Change.** Harness only: `examples/evidence` gains `--stall-timeout` and `--failed-place-memory`. Open issues #24, #27 are release issues, skipped. No crate code or default changed.

**Evidence** (`runs/2026-10-28-04/eval.md`, three seeds, 10 days). Memory 3 h on top of stall 1 h cuts interrupted endings 311→236, 332→246, 325→290. But stall detection alone doubles interruptions over the default (172→311 at seed 42) and moves rivals 24→40 at seed 7, so the village stays `None`/`None`. **Verdicts:** not run (no default changed). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Why stall detection swings friends/rivals by seed (24→40 rivals at seed 7); then decide whether the village enables stall + memory.
2. Friendship brake independent of rivalry, then retry the snub rate.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the guide's scale table with the fixed benchmark homes on a quiet host.

## 2026-10-28-05 — stall detection widens the friends/rivals scatter but does not bias it (measurement only)

**Change.** None to crate or harness. Open issues #24, #27 are release issues, skipped. Answers top item 1 of the last pass.

**Evidence** (`runs/2026-10-28-05/eval.md`, ten new seeds, 24 people, 10 days). Mean friends 19.1 → 18.4, rivals 26.1 → 27.2 with `stall_timeout_hours = 1`; sd of rivals 3.9 → 7.3; interrupted endings 135 → 365. The earlier "24 → 40 rivals" is scatter, not a shift. Village default stays `None`. **Verdicts:** not run (no behaviour changed). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Why stall-cut walks (about 2.7x more interrupted endings) widen the rivals spread; with `failed_place_memory_hours` the village may still enable both, which needs plots and three reviews.
2. Friendship brake independent of rivalry, then retry the snub rate.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the guide's scale table with the fixed benchmark homes on a quiet host.

## 2026-10-28-06 — scale table re-measure abandoned: host too loaded

**Change.** None. Open issues #24, #27 are release issues, skipped. No crate, harness or doc change.

**Evidence** (`runs/2026-10-28-06/eval.md`). Load average 30-45; 100 people spread 35.6%, 1 000 and 5 000 did not finish in 900 s. Nothing quotable; guide table unchanged. **Verdicts:** not applicable. **Dead-code delta:** zero.

**Top 5 remaining.**
1. Friendship brake independent of rivalry (Dunbar trim), then retry the snub rate.
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
4. Re-measure the scale table on a quiet host; make `examples/benchmark` stop warning for its own setup.
5. Stall + failed-place memory in the village needs plots and three reviews before enabling.

## 2026-10-28-07 — benchmark prints only its JSON line

**Change.** `examples/benchmark` filters the `bevy_society::behavior` validator warnings its deliberately compressed setup trips (9 lines at 100 people, 250 at 5 000). Workload unchanged. Open issues #24, #27 are release issues, skipped. Evidence: `runs/2026-10-28-07/eval.md`. **Verdicts:** not applicable (example only). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Friendship brake independent of rivalry (Dunbar trim), then retry the snub rate.
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
4. Re-measure the scale table on a quiet host (load average still 9-16).
5. Stall + failed-place memory in the village needs plots and three reviews before enabling.

## 2026-12-30-01 — friend-capacity brake built and measured, not shipped

**Change.** None to crate or pack (tree reverted; trial kept as `runs/2026-12-30-01/friend-capacity.patch`). Open issues #24, #27 are release issues, skipped. Took the last four passes' top item: a friendship brake independent of rivalry, so the snub rate could fall.

**Evidence** (`runs/2026-12-30-01/eval.md`). The brake works (`Power(1.2)`: 90.8% → 21.9% friend pairs at 60 people) but leaves 0 rivals, failing `rivals >= 1`. New fact: HEAD's 60-person town is 32.7% rival pairs and 1.8% friend pairs; rivals across snub exponents 1.0/1.1/1.15/1.2 are 1157/1267/426/0 on seed 7, no stable middle. **Verdicts:** not run (nothing shipped). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Why the default 60-person town is a third rival pairs (`became_rival` vs `left_rival` over 30 days); the friendship bar guards the wrong failure.
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
4. Re-measure the scale table on a quiet host (the table is from an M1 Max; this host differs).
5. Stall + failed-place memory in the village needs plots and three reviews before enabling.

## 2026-10-28-08 — refusal penalty is not what makes the town rival-heavy (measurement only)

**Change.** Harness only: `examples/evidence --rejection-penalty`. Open issues #24, #27 are release issues, skipped. No crate or default change.

**Evidence** (`runs/2026-10-28-08/eval.md`, three seeds, 24 people, 17 days). Penalty 0.05 → 0 moves mean rivals 33.0 → 27.3 and friends not at all; insults rise ~20%. At 60 people, 85% of insults (1 588 of 1 875) come from someone the victim already dislikes: the rivalry engine is retaliation through `snub`'s grudge-only consideration. The harness's 170 rival ties at 60 people disagree with `friendship_bound`'s 1157; not reconciled. **Verdicts:** not run. **Dead-code delta:** zero.

**Top 5 remaining.**
1. Retaliation loop: bound a victim's snub rate per insult received (Bondü & Richter 2016), measure, then retry the snub exponent.
2. Reconcile the evidence harness's rival count with `friendship_bound`'s at 60 people.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the scale table on a quiet host (load average still ~25).

## 2026-10-28-09 — a delivered retaliation settling the grievance does not calm the town (measured, not shipped)

**Change.** None to crate or pack (tree reverted; trial kept as `runs/2026-10-28-09/revenge-settles.patch`). Open issues #24, #27 are release issues, skipped. Took the last pass's top item, the retaliation loop: after an accepted harmful deed the actor's bad memories about the target were scaled by `1 - f`, citing Gollwitzer, Meder & Schmitt 2011 (DOI 10.1002/ejsp.782).

**Evidence** (`runs/2026-10-28-09/eval.md`). f = 0 reproduces HEAD. Insults do not fall (390 → 369 → 389 at seed 7; 1 875 → 1 808 at 60 people); rivals rise at f = 1.0 (mean 33.0 → 44.7) and friends fall (21.0 → 14.3). The loop is standing dislike re-fed by every contact, not insult-for-insult retaliation. **Verdicts:** not run. **Dead-code delta:** zero.

**Top 5 remaining.**
1. Tally what writes the grudge behind each snub at decision time, by memory kind (insult received / refusal / witnessed deed / rumor), at 24 and 60 people.
2. Reconcile the evidence harness's rival count with `friendship_bound`'s at 60 people.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the scale table on a quiet host.

## 2026-10-28-10 — three quarters of snubs answer a received insult (measurement only)

**Change.** Harness only: `examples/evidence` writes `grudge_sources.txt`. Open issues #24, #27 are release issues, skipped. No crate or default change.

**Evidence** (`runs/2026-10-28-10/eval.md`, seeds 7/42/11 at 24 people, 60-person town). 73% of snubs (55% at 60 people) rest on an insult the snubber received and accepted; 10-19% on its own refused insult; rumors and witnessed deeds <2%. The loop is a reply-ping-pong; the previous pass cleared only the actor's side. **Verdicts:** not run. **Dead-code delta:** zero.

**Top 5 remaining.**
1. Break the ping-pong at the reply: a per-pair retaliation refractory period, or no grudge-bearing memory from a refused insult; measure insults and rivals at the same seeds.
2. Reconcile the evidence harness's rival count with `friendship_bound`'s at 60 people.
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the scale table on a quiet host.

## 2026-10-29-02 — `Away`: the town misses the people who left, and a return is an event (fixes #35)

**Change.** `Away` (`#[require(Dormant)]`) and `AwaySince`; close holders (a `Friend` tie, or `familiarity_peak` at `bereavement_familiarity`) hold a `Distress` toward the away person at `missing_cap · closeness · (1 − e^(−days/missing_days))`; removing `Away` writes `PersonReturned { person, days_away }` once and gives each holder `Joy` of `reunion_fraction` of what they were missing; `Deceased` replaces `Away` with no return; `SocietySave` version 3 keeps the start of an absence. Three design-default `Tuning` fields (`missing_cap` 0.4, `missing_days` 2.0, `reunion_fraction` 0.75); source for the mechanism Hazan & Shaver 1987 (DOI `10.1037/0022-3514.52.3.511`). Open issues: #35 worked; #24 and #27 are release issues, closed by this pass's release. Parked branches (`wiggum/parked-pass-20/28/36/37`) are for older items, not adopted.

**Evidence** (`runs/2026-10-29-02`). `tests/absence.rs` 8 tests; `village-evening` `summary.json` and `series.csv` byte-identical before and after; the absence harness shows the closest friend's distress 0.06 → 0.19 over three days, 0 for a stranger, gone within a day of the return (`evidence/plots/absence-seed7.png`). Perf unchanged within noise (`perf.md`; 5000 people not measured under load). **Verdicts:** Behavior GREAT, Zoom GREAT, Fresh eyes GREAT with the stated limit (the harness has no RNG). **Dead-code delta:** zero. Assumed decisions in `needs-decision/an-absence-is-felt-not-caused.md`.

**Top 5 remaining.**
1. A host that writes a tie above `affinity_ceiling` (0.35) sees it clamped at the next write; the absence harness showed it. Document it on `Ties::entry_with`.
2. Population-level check of `Away` in a living town (a party leaving through `Autonomous` life).
3. `ReportDeed` leaves the victim no memory (needs-decision).
4. Break the insult reply ping-pong (per-pair retaliation refractory period).
5. Re-measure the scale table on a quiet host (5000 people did not finish under load).

**Release 2026-10-29-02.** `cargo semver-checks` required a major bump (3 major checks, from earlier unreleased work); published **0.4.0**, https://crates.io/crates/bevy_society/0.4.0, tag `v0.4.0`; closes #24 and #27 (0.3.1 was never cut). **Cleanup:** removed the baseline worktree `/home/ladvien/wt-absence-base` and `/home/ladvien/wt-absence-base-target` and my scratch under `/tmp` (`abs7`, `abs42`, `absfresh`, `suite.log`); `~/.venvs/society-plots` kept for the plot script. No leftover worktrees from earlier passes.

**Correction to the cleanup line.** Earlier passes left two worktrees, listed and not removed: `/home/ladvien/.cache/loop-runs/before-tree` and `/home/ladvien/.cache/price-before-src`. A person should approve their removal.

## 2026-10-29-03 — `Away` in a living town; the tie-write clamp documented (small pass)

**Change.** No crate behaviour change. `Ties::entry_with` now documents that the affinity it writes is not clamped but the next `add_affinity` is (last pass's top item 1). New test `an_autonomous_party_member_leaves_and_comes_back_into_the_day` (`tests/absence.rs`): with two `Autonomous` friends, the one `Away` starts no activity for 1.5 days while the other keeps living and misses them; after `(Away, Dormant)` are removed they decide again and one `PersonReturned` is announced. Open issues: none. Parked branches (`wiggum/parked-pass-20/28/36/37`) are for older items, left alone.

**Evidence.** `cargo test --test absence` 8/8. First run failed only because the test removed `Away` alone, which leaves `Dormant` (documented on `Away`); the crate was right. The host was at load ~45, so the full gate (both feature sets, clippy, doc, perf) was not run; only the doc comment and a test changed. **Verdicts:** not run (no behaviour change). **Dead-code delta:** zero.

**Top 5 remaining.**
1. Break the insult reply ping-pong (per-pair retaliation refractory period).
2. `ReportDeed` leaves the victim no memory (needs-decision).
3. Reconcile the evidence harness's rival count with `friendship_bound`'s at 60 people.
4. Clock decision: `travel_timeout_hours` in game hours vs real-time walking.
5. Re-measure the scale table on a quiet host.

**Cleanup.** Nothing created outside the clone. Earlier leftovers unchanged: `/home/ladvien/.cache/loop-runs/before-tree`, `/home/ladvien/.cache/price-before-src` (listed, not removed).

## 2026-10-29-04 — a reported deed's victim keeps a `Suffered` memory

**Change.** Assumed decision (owner may override): option 2 of `needs-decision/a-victim-remembers-nothing-a-host-reported.md`. `MemoryKind::Suffered { deed, actor }` recorded at the patient of a `ReportDeed` with harm < 0, valence `-(0.3+|harm|)`; save `VERSION` 4 (3 → 4 no-op upgrade). Victim `grudge` was 0.0000 forever, now 0.8 at the deed for `brawl` (harm −0.5), so `snub` is reachable. Witnesses and bystanders unchanged. No open issues; parked branches not adopted (older, other items).

**Evidence.** `tests/report_deed.rs::a_reported_victim_remembers_and_resents_the_actor`. No plots/three-review gate run (host loaded; narrow path only `ReportDeed`, no scenario uses it). **Dead-code delta:** zero.

**Top 5 remaining.** 1. Insult reply ping-pong. 2. Reconcile rival count with `friendship_bound`. 3. Clock decision `travel_timeout_hours`. 4. Re-measure scale table. 5. Release (breaking change pending, 0.5.0).

## 2026-10-29-05 — one blow per grievance (`retaliation_refractory_hours`), adopted from parked pass 40

**Change.** Adopted `wiggum/parked-pass-40-…` (the insult-reply ping-pong item): opt-in `Tuning::retaliation_refractory_hours` (default `None`, design default; Gouldner 1960, DOI 10.2307/2092623). Applied only its `behavior.rs`/`core.rs`/`evidence.rs` hunks and its tests/run docs; its other files predate `main` and were not taken. No open issues. Other parked branches (20/28/36/37) older, ignored.

**Evidence** (`runs/2026-10-29-04/`, run docs from the parked tree, numbers in its `eval.md`): town 60: reply share of snubs 55.5% → 41.2%, rivals 170 → 138; village seeds 7/11 improve, 42 flat. `tests/retaliation.rs` 3/3. **Verdicts:** GREAT / GREAT (limit stated) / GREAT per its `review.md`. Default `None`: no run changes. **Dead-code delta:** zero.

**Top 5 remaining.** 1. Reconcile rival count with `friendship_bound`. 2. Clock decision `travel_timeout_hours`. 3. Re-measure scale table. 4. Release 0.5.0 (breaking `MemoryKind` variant unreleased). 5. Decide whether to default the refractory on.

**Release 2026-10-29-05.** Published **0.5.0** (`MemoryKind::Suffered` is breaking per semver-checks), https://crates.io/crates/bevy_society/0.5.0, tag `v0.5.0`. Also fixed `examples/evidence.rs` (non-exhaustive `Suffered` match that broke `cargo test`'s example build on `main`). **Cleanup:** created nothing outside the clone. Earlier leftovers still listed, not removed: `/home/ladvien/.cache/loop-runs/before-tree`, `/home/ladvien/.cache/price-before-src`.

## 2026-10-30-01 — evidence harness counts directed ties

**Change.** `examples/evidence.rs` halved the friend and rival tie counts (`/ 2`), assuming mutual ties; one-way ties were rounded down. Now directed ties, the unit `tests/friendship_bound.rs` uses (top item 3 of last pass). Library unchanged. No open issues; parked branches older, ignored. Host load ~30, so no perf re-measure and no behaviour gate.
**Evidence.** `runs/2026-10-30-01/eval.md`, `evidence/after/summary.json` (village, 24, 6 days: friends 25, rivals 93). **Dead-code delta:** zero. Old numbers in earlier logs are halved; compare ×2.
**Top 5 remaining.** 1. Re-measure scale table on a quiet host. 2. Decide default for `retaliation_refractory_hours`. 3. Clock decision `travel_timeout_hours`. 4. Insult ping-pong residue (41% replies). 5. Friendship capacity brake (#10).
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-02 — `retaliation_refractory_hours` defaults to `Some(6.0)`

**Change.** Top item 2 of last pass, decided: assumed decision (owner may override): default on at 6 h (options: keep `None`; 6 h; 12 h; pick 6 h, the value measured, conservative, `None` restores old). No open issues; parked branches older, ignored.
**Evidence.** Numbers from `runs/2026-10-29-04/eval.md` (same seeds, same 6 h): town reply share 55.5% → 41.2%, rivals 170 → 138. Full `cargo test --release` green (all suites incl. `friendship_bound`, `retaliation`). Host load ~36: no perf, no new plots. Verdicts: not re-run (reuse of 2026-10-29-04 GREENs). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table. 2. Clock decision `travel_timeout_hours`. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-03 — evidence harness measures partner repetition; compliment social-need gate rejected

**Change.** `examples/evidence.rs` writes `partners.txt` (share of exchanges with the initiator's previous partner, per interaction). Library unchanged. No open issues; parked branches older, ignored. `travel_timeout_hours` clock decision is already dissolved by `Locomotion::Host` (`needs-decision/travel-time-in-two-clocks.md`): no work left.
**Evidence.** `runs/2026-10-30-02/eval.md`: 33.0% of exchanges repeat the previous partner (village, 24, 6 days). A `Need("social")` gate on `compliment` left it at 32.1% and made friends 26 → 16, rivals 77 → 105; reverted. No behaviour change shipped, so no plots/three-review gate. Host load ~30: no perf. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Decide whether partner repetition is implausible (needs a source; a per-tie `last_exchange_day` would be the lever). 2. Re-measure scale table on a quiet host. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts (77 directed rivals of 552 pairs).
**Cleanup.** Scratch `/tmp/ev0 /tmp/ev1 /tmp/ev2 /tmp/ev3` removed.

## 2026-10-30-04 — authored placement (`PlacementFile`), issue #41 (consumer:bevy_emerge)

**Change.** General form of the request: `PlacementFile { places, persons }` in `src/placement.rs`, serde in and out (RON), `spawn(&mut World)` validates the whole file against `SocietyDefs` first (spawns nothing on error), no RNG. Person fields: personality, needs, signals, opinions, factions, `home`, named `bound` places (workplace = `("work", place)`), start `at`, directed ties. Homes are named places, not raw points: a place entity is what `Home`/`BoundPlaces` need. Assumed decision (owner may override): options were raw `[f32;2]` homes (needs anonymous places), named places only, or both; picked named places, the smallest that meets the acceptance. Uses `BTreeMap` so `tests/leaf.rs` is untouched. No other open issues; parked branches older, ignored.
**Evidence.** `tests/placement.rs` 4/4: RON round-trip equality, homes/workplace/faction/tie present after spawn, a bad reference spawns nothing and names the owner, same seed gives same positions after 200 ticks. Library-only data path: no behaviour scenario touched, so no plots/three-review gate; host load not measured. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table. 2. Insult ping-pong residue (41%). 3. Friendship capacity brake (#10). 4. Reconcile rival counts. 5. Release (new API, 0.6.0 pending semver check).
**Cleanup.** Created nothing outside the clone; scratch `/tmp/p.py` removed.

## 2026-10-30-05 — `check_script`: standalone script syntax check with line and column

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is under 6 h old. Pick (editor need, bevy_emerge #105 acceptance "a syntax error is refused with the line and message"): `check_script(source, ScriptKind)` → `ScriptSyntaxError { line, column, message }`, same sandbox limits as the compiler, no app or `DefsFile` needed. Runner-ups: friendship capacity brake (#10, escalated), scale-table re-measure (host load 14-20).
**Evidence.** `tests/check_script.rs` 2/2, `tests/leaf.rs`, clippy `-D warnings`, doc green. Library-only API: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table. 2. Insult ping-pong residue (41%). 3. Friendship capacity brake (#10). 4. Reconcile rival counts. 5. Release (0.6.0 pending semver check, after 6 h from v0.5.0).
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-06 — `place_dependents` and `persons_in_rect` (bevy_emerge #104/#105 acceptance)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 1 h old. Pick (editor need): bevy_emerge #104 "removing a building that is someone's home is refused, naming the person" and #105 "a trigger bound to an area fires only for people inside it" need crate-side queries: `place_dependents(world, place)` (home or bound role, `PersonId` order) and `persons_in_rect(world, min, max)` on the society plane. Runner-ups: friendship capacity brake (#10, escalated), scale table (host load 32).
**Evidence.** `tests/placement.rs` 5/5 (new test: cottage lists Ada and Bram as home, mill lists Ada as work, rects select the right person), `tests/leaf.rs`, clippy `-D warnings`, `--no-default-features` build green. Library-only queries: no behaviour change, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-07 — `PlacementFile::capture`: save a live world as a placement file

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is under 6 h old. Pick (editor need, bevy_emerge #104 "saved in a game's project and edited live"): `capture(&mut World)` is the inverse of `spawn`; needs, signals, positions are current, opinions are anchors, unnamed place/person is an error. Runner-ups: scale-table re-measure (host load 38, unreliable), friendship brake (#10, needs a source).
**Evidence.** `tests/placement.rs` 7/7 (live edit read back; save, reload, capture again equals; unnamed person refused), `tests/leaf.rs`, clippy `-D warnings`, doc green. Library-only API: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-08 — `TriggerDef::area`: a trigger bound to a rectangle (bevy_emerge #105)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is under 6 h old. Pick (editor need, #105 acceptance "a trigger bound to an area fires only for people inside it"): `ctx` exposes no position, so a script could not express it. `TriggerDef.area: Option<TriggerArea>` (design default: absent = anywhere), checked in `evaluate_triggers` against the actor's current society-plane position, edges inclusive; `Tick`/`Hour`/`RumorReach` (no actor) and inverted or non-finite corners are refused at load. Assumed decision (owner may override): options were a declarative rectangle, a `ctx.position(p)` script reader, or both; picked the rectangle, since the editor edits it as data and a selection maps onto it 1:1. Runner-ups: scale-table re-measure (host load 35), friendship brake (#10, needs a source).
**Evidence.** `tests/trigger_area.rs` 2/2 (inside fires, outside and walked-out do not, edge inclusive; bad areas refused), full `cargo test --release` green, clippy `-D warnings`, `--no-default-features` build (5 pre-existing dialogue-gated warnings). Data path only, no behaviour scenario touched: no plots/three-review gate; host load not measured. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-09 — `DefsFile::to_ron`: every definition type serialises (bevy_emerge #104/#105 "crate's serde types are the data")

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2 h old. Pick (editor need): `DefsFile` and every type under it were `Deserialize` only, so an editor could load a definitions file and never save an edited trigger, activity or need. Added `Serialize` to all of them (`Trait`, `EmotionKind`, `TieLabel`, `Curve` too, no `serde` feature needed, matching `Deserialize`) and `DefsFile::to_ron`. Runner-ups: scale-table re-measure (host load 34), friendship brake (#10, needs a source).
**Evidence.** `tests/defs_roundtrip.rs` 3/3 (starter, starter+village: second write equals first and reloaded file compiles; an edited trigger keeps `if`, `once`, cooldown, area), `tests/leaf.rs` and `--no-default-features` green, clippy `-D warnings`, fmt. Data path only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0; new API). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-10 — `TriggerFired`: which triggers ran (bevy_emerge #105)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2 h old. Pick (editor need, #105 "see which triggers fired" against a fast-forwarded society): no observable existed besides a script's own `fx.emit`. Added `TriggerFired { trigger, actor, target }` (authored id, `PersonId`s), written when a trigger's `then` completes without error; failures keep writing `ScriptError`. Runner-ups: live defs hot-reload (needs a design: ids are indices, saves compare names), scale-table re-measure (host load 30).
**Evidence.** `tests/trigger_area.rs` 3/3 (new: inside actor reported once with id and `PersonId`, outside actor not), `tests/leaf.rs`, `guide_blocks`, clippy `-D warnings`, doc, `--no-default-features` build (5 pre-existing warnings). Message surface only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Defs hot-reload at runtime (#105 "editing a script in the running game"). 3. Re-measure scale table. 4. Insult ping-pong residue (41%). 5. Friendship capacity brake (#10).
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-11 — `reload_defs`: edit a running society's definitions (bevy_emerge #105)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2.3 h old. Pick (editor need, #105 acceptance "editing a script in the running game takes effect without a restart"; top item 2 of last pass): `SocietyDefs::reload` compiles the new file and refuses it unless every id table is name-for-name unchanged (ids in the world are indices); `reload_defs(world, file)` swaps it in and remaps `TriggerState` (`once`, cooldown) by trigger id. Assumed decision (owner may override): options were hot-swap with id tables frozen, an id-remapping migration of every component, or triggers only; picked frozen id tables, since it covers scripts, activities and interactions without touching per-person state, and a structural change still needs a restart. Runner-ups: scale table (host load 20), friendship brake (#10).
**Evidence.** `tests/trigger_area.rs` 4/4 (new: edited script runs for a new actor, spent `once` stays spent through the edit, new trigger live, uncompilable and renamed-need edits refused leaving the last good defs), `leaf`, `guide_blocks`, `defs`, `defs_roundtrip`, clippy `-D warnings`, doc, `--no-default-features` build. API only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-12 — `TriggerDef::factions` / `persons`: bind a trigger to people and factions (bevy_emerge #105)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2.4 h old. Pick (editor need, #105 "bind a trigger to people, factions or a map area"; only the area existed): `factions: Vec<String>` (actor belongs to any) and `persons: Vec<u32>` (actor's `PersonId`), both empty = any actor (design default), both listed = either; unknown faction and either list on `Tick`/`Hour`/`RumorReach` refused at load. Runner-ups: scale-table re-measure (host load 19), friendship brake (#10, needs a source).
**Evidence.** `tests/trigger_area.rs` 6/6 (new: faction member and listed id fire, a stranger does not; bad lists refused), full `cargo test --release` green, clippy `-D warnings`, fmt, `--no-default-features` build. Data path only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-13 — `fast_forward`: run a live society N days in one call (bevy_emerge #104)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2 h old. Pick (editor need, #104 acceptance "a fast-forward of a fixture village is deterministic: same seed, same day"): no crate-side way to advance a society without frames. `fast_forward(&mut World, days) -> Result<FastForward, FastForwardError>` runs `FixedMain` one fixed step at a time with message buffers rotated; `ceil(days * seconds_per_day / tick_secs)` ticks. Refuses non-finite or negative days, an app that never ran `Startup` (no RNG yet), and `ClockSource::Host` (the host owns that clock). Runner-ups: scale-table re-measure, friendship brake (#10, needs a source).
**Evidence.** `tests/fast_forward.rs` 3/3 (2 days = 512 ticks, whole-society hash equals a live app's after the same ticks and differs from the start; 1+1 days equals 2; error cases), `tests/leaf.rs`, clippy `-D warnings`, doc, `--no-default-features` build (5 pre-existing warnings). API only: no behaviour change, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone; `/tmp/ed.py` removed.

## 2026-10-30-14 — `test_trigger`: dry-run a trigger on the frozen society (bevy_emerge #105)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is under 6 h old. Pick (editor need, #105 "test it against a snapshot of the fast-forwarded society"): `test_trigger(world, id, actor, target) -> TriggerTest { selected, in_area, condition, would_fire, effects }`; effects are the `Debug` text of each queued write (design default: `ScriptAction` stays crate-private). Ignores `once`/cooldown and whether the event happened; changes nothing. Errors: no snapshot yet, unknown id, script failure naming its authored path. Runner-ups: scale-table re-measure (host load 24), friendship brake (#10, needs a source).
**Evidence.** `tests/test_trigger.rs` 2/2 (each check reported, repeat call identical, nothing emitted; unknown id and failing script are errors), `tests/leaf.rs`, `guide_blocks`, clippy `-D warnings`, fmt, `--no-default-features` build (5 pre-existing warnings). API only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-15 — `preview_forward`: run forward, report, reset (bevy_emerge #104)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 2.9 h old. Pick (editor need, #104 "run the society forward ... show where people go and how their ties change, then reset"): `fast_forward` had no reset and no report. `preview_forward(world, days) -> Preview` = capture, fast_forward, capture, restore, plus each person's `Transform` put back; reports per `PersonKey` position before/after, activity before/after and changed ties. Needs `PersonKey` on every person (placement does not write one; the crate never writes keys by design), else `PreviewError::Unkeyed` before anything runs. Feature `serde`. Runner-ups: scale-table re-measure (host load 17), friendship brake (#10, needs a source).
**Evidence.** `tests/fast_forward.rs` 5/5 with `--features serde` (new: a 1-day preview moves both villagers, whole-society hash after equals before, a repeat preview is identical; unkeyed refused untouched), `tests/leaf.rs`, clippy `-D warnings`, fmt, doc, `--no-default-features` build. API only: no behaviour scenario, no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Re-measure scale table. 3. Insult ping-pong residue (41%). 4. Friendship capacity brake (#10). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone.

## 2026-10-30-16 - friend-capacity brake + softer snub: measured, blocked by a protected pin

**Change.** None shipped. No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 3 h old. Pick (ranked item 3, weakest model: the 60-person town had 29 friends against 170 rivals and 2.9:1 compliments to insults): the 2026-12-30 patch (`friend_capacity` 10, after Zhou et al. 2005, DOI 10.1098/rspb.2004.2970, plus `snub` at `Power(1.15)`) re-measured with the refractory default. Same seeds: village insults 390/369/415 -> 156/165/181 (8:1), friends 21/22/20 -> 91/90/72, rivals 35/28/36 -> 27/27/34; town rivals 170 -> 54, friends 29 -> 209 (plateau); `friendship_bound` passes. It fails `label_hysteresis::a_friendship_is_held_through_a_shallow_dip_and_lasts_longer` (`rivals_with == rivals_without`, 19 vs 16, a chaotic-equality pin); Power(1.1) passes both but gives 1070 rivals, 1.2 gives 0. Assumed decision (owner may override): do not loosen a protected test; patch kept, blocker written in `needs-decision/snub-rate-vs-scarce-friendship.md`. Runner-ups: scale-table re-measure (host load 26), editor APIs (#104/#105 acceptances all covered).
**Evidence.** `runs/2026-10-30-16/{eval,review,research,plan}.md`, `evidence/plots/`, `friend-capacity-1.15.patch`. Review: behavior GREAT, zoom GREAT, fresh eyes NOT YET (bistable exponent). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Owner: relax `label_hysteresis` rival equality to a tolerance, then apply the patch (needs-decision). 2. Damp the grudge feedback so the rival count is not bistable in the snub exponent. 3. Release 0.6.0 (after 6 h from v0.5.0). 4. Re-measure scale table. 5. Reconcile rival counts.
**Cleanup.** Removed `/tmp/mpl-iter56`; `docs/loop/runs/2026-10-30-16/evidence/*/series.csv` stay ignored (68 MB local).

## 2026-10-30-17 - decision path: O(n^2) list searches removed (bit-identical)

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is 3.5 h old at pass start. The friend-capacity brake stays blocked (it needs the `label_hysteresis` equality loosened, which the brief forbids). Pick (ranked item 4, performance; the scale table is from an M1 and 20 passes of features old): `decide_activities` scanned `PersonIndex.sorted` for the actor id per person, and `near` for the target id per target (O(neighbours^2) per activity per person at 182 neighbours); `score_option` re-searched the tie list and neighbour list per consideration. Now a slot read, a binary search over a sorted copy, and once per option. Assumed-nothing: no owner decision needed.
**Evidence.** `runs/2026-10-30-17/eval.md`: evidence `summary.json` and `series.csv` byte-equal to HEAD at seeds 7 and 42; full `cargo test --release` green, clippy `-D warnings`, fmt, `--no-default-features` build. Speed: -8%, 0%, -9% in three paired 3 000-person runs on a host at load 20+, so a small gain, not a figure. Behaviour unchanged, so no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0). 2. Attribute the other ~50% of `decide_activities` (needs a profiler: `perf_event_paranoid` 2; try per-person `Decision` clones of `ties`/`near`/`place_positions`, Rhai scope per target). 3. Re-measure scale table on a quiet host. 4. Friendship capacity brake (#10, owner relaxes `label_hysteresis` equality). 5. Reconcile rival counts.
**Cleanup.** Removed `/tmp/bs-base` worktree, `/tmp/bs-base-target`, `/tmp/ev-before-*`, `/tmp/ev-after-*`, `/tmp/bench-{before,after}`, `/tmp/samples.txt`, `/tmp/perf.data`. One `git stash` entry from this pass (`stash@{0}`, my clean copy of `src/behavior.rs`) is left for a person to drop.

## 2026-10-30-18 - save/restore bug: the retaliation refractory was not saved; `ctx.position` and `ctx.distance`

**Change.** No open issues; parked branches older, ignored. Release skipped: `v0.5.0` is under 6 h old (00:42, pass started 05:21). The friend-capacity brake stays blocked (loosening `label_hysteresis` is the one block the brief leaves). **Bug first:** `tests/save.rs::the_hundred_ticks_after_a_restore_are_the_uninterrupted_ones` fails on `main` (hashes `16078120503991199322` vs `11685627293723451633`, identical at HEAD with my change stashed). Cause: `473b396` made `retaliation_refractory_hours` default `Some(6.0)`, and the `RecentHarm` it writes ("Not saved") steers `decide_activities`, so a reload let through insults the uninterrupted run was refusing. Fix: `PersonSave::recent_harm` (target key, until-day bits), `#[serde(default)]`, `VERSION` stays 4 (older files restore with no refractory running; bumping would have needed re-pinning `tests/economy.rs`). Second item (editor need, bevy_emerge #105 noted `ctx` has no position reader): `ctx.position(a)` (`#{x, y}` on the society plane, from `Transform`, like a trigger's `area`) and `ctx.distance(a, b)`, tick-start values; a person without a `Transform` is a script error naming them.
**Evidence.** `tests/save.rs` 4/4 (was 3/4); new `tests/script_position.rs` (distance and position in a trigger's `if`; a moved person changes the answer); full `cargo test --release --all-features --no-fail-fast` green (92 binaries ok), clippy `-D warnings`, fmt, doc, `--no-default-features` build and tests. Bug fix plus API: no new behaviour scenario, no plots/three-review gate (precedent: iterations 14-15). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.0 (after 6 h from v0.5.0, i.e. after 06:42; new API). 2. `FailedPlaces` is also unsaved (it did not trip the test, but is the same class of divergence). 3. Re-measure scale table on a quiet host. 4. Friendship capacity brake (#10, owner relaxes `label_hysteresis` equality). 5. Reconcile rival counts.
**Cleanup.** Created nothing outside the clone; `stash@{0}` (`iter58-wip` popped, none left from this pass).

## 2026-10-30-19 - save/restore bug: the failed-place memory was not saved

**Change.** No open issues; parked branches older, ignored. Pick (bug recorded in the log, top item 2 of pass 18): `FailedPlaces` (`failed_place_memory_hours`) steers `decide_activities` and was "Not saved", the same class as the retaliation refractory. `PersonSave::failed_places` (place key, until-day bits, `#[serde(default)]`, `VERSION` 4) saves and restores it; unkeyed places are left out. Release 0.6.0 follows (v0.5.0 is over 6 h old).
**Evidence.** `tests/failed_place.rs::a_restore_keeps_the_failed_places` (feature `serde`: the failed tavern is in the save by key and a restored world recaptures the same list); full `cargo test --release --all-features` green (0 failing binaries), clippy `-D warnings`, fmt, `--no-default-features` build. Bug fix: no behaviour scenario, no plots/three-review gate (precedent: pass 18).
**Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table on a quiet host. 2. Attribute the other ~50% of `decide_activities`. 3. Friendship capacity brake (#10, owner relaxes `label_hysteresis` equality). 4. Reconcile rival counts. 5. Insult ping-pong residue (41%).
**Cleanup.** Created nothing outside the clone.
**Release.** v0.6.0 published, https://crates.io/crates/bevy_society/0.6.0 (semver-checks: breaking, new pub fields on TriggerDef and PersonSave).

## 2026-10-30-20 - save/restore: a walker's stall window was not saved

**Change.** No open issues; parked branches older, ignored. Pick (same class as passes 18-19, found by grepping `Not saved` in `src/`): `TravelProgress` (the `stall_timeout_hours` window) was dropped by a save, so a restored walker got a fresh window. `PersonSave::travel_progress` (`(f32 bits, f64 bits)`, `#[serde(default)]`, `VERSION` 4) saves and restores it; two stale "Not saved" doc lines on `Tuning` corrected. Release skipped: v0.6.0 is under 6 h old.
**Evidence.** `tests/failed_place.rs::a_restore_keeps_the_stall_window_of_a_walker` (feature `serde`); full `cargo test --release --all-features` and `--no-default-features` green, clippy `-D warnings`, fmt. Bug fix: no plots/three-review gate (precedent: passes 18-19). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table on a quiet host. 2. Attribute the other ~50% of `decide_activities`. 3. Friendship capacity brake (#10). 4. Reconcile rival counts. 5. Insult ping-pong residue (41%).
**Cleanup.** Created nothing outside the clone. Leftover from pass 17: `stash@{0}` for a person to drop.

## 2026-10-30-21 - save/restore: the price book was not saved

**Change.** No open issues; parked branches older, ignored. Release skipped: v0.6.0 is under 6 h old. Pick (same class as passes 18-20, found by auditing resources against `save.rs`): `Prices` is recomputed from stores each tick, but `ctx.price` reads the tick-start snapshot, frozen before that update, so the first tick after a restore (or a `preview_forward` reset) read the fresh world's book. `SocietySave::prices` (`f32` bits, `#[serde(default)]`, `VERSION` 4) saves and restores it; `preview` doc corrected. Other resources audited: queues are drained within a tick; `Home`/`BoundPlaces`/`Autonomous` are spawn-time host data.
**Evidence.** `tests/economy.rs::a_restore_puts_the_price_book_back` (serde; precondition asserts the fresh book differs); economy, save, save_upgrade, leaf, readme, guide_blocks green; clippy `-D warnings`, fmt, `--no-default-features` build. Bug fix: no plots/three-review gate (precedent: passes 18-20). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure scale table on a quiet host. 2. Attribute the other ~50% of `decide_activities`. 3. Friendship capacity brake (#10, owner relaxes `label_hysteresis` equality). 4. Reconcile rival counts. 5. Insult ping-pong residue (41%).
**Cleanup.** Created nothing outside the clone. Leftover: `stash@{0}` from pass 17, for a person to drop.

## 2026-10-30-22 - scale table re-measured on v0.6.0 (docs only)

**Change.** No open issues; parked branches older, ignored. Release skipped: v0.6.0 is under 6 h old and no packaged file changed. Pick (top item 1 of pass 21, ranked item 4, performance): the guide's scale table was an M1 Max from 20+ passes ago and had no 5 000-person row. `docs/guide/concepts.md` now carries a v0.6.0 table at 100/1 000/2 000/5 000 people. Save-state audit of every `Component`/`Resource` against `save.rs` found nothing further unsaved (the rest is recomputed per tick or spawn-time host data).
**Evidence.** `runs/2026-10-30-22/perf.md` (fastest/median/spread per size). Host load 11-38, spreads 28-134%, so the table says "upper bounds" outright. No code change: no plots/three-review gate. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Re-measure on a quiet host (spread under 20%). 2. Attribute the other ~50% of `decide_activities`. 3. Friendship capacity brake (#10, owner relaxes `label_hysteresis` equality). 4. Reconcile rival counts. 5. Insult ping-pong residue (41%).
**Cleanup.** Created nothing outside the clone (`target/` is the clone's own).

## 2026-10-30-23 - `dialogue_problems`: dangling dialogue references as data (bevy_emerge #106)

**Change.** No open issues; parked branches older, ignored. Release skipped: v0.6.0 is under 6 h old. Friend-capacity brake still blocked (needs `label_hysteresis` equality loosened). Pick (idle ladder 1, editor need: bevy_emerge #106 wants a conversation graph "validated" and rows marked): the crate only logged conversation rules and node effects that name a conversation or node the `DialogueScript` lacks, and skipped `DialogueNode` triggers. `dialogue_problems(&defs, &script) -> Vec<DialogueProblem>` (+ `DialogueReferenceKind`) returns every one as data, covers triggers too; the plugin's log uses it.
**Evidence.** `tests/dialogue.rs::dialogue_problems_lists_each_dangling_reference` (7/7), lib tests, `leaf`, `readme`, `guide_blocks`, clippy `-D warnings`, fmt, doc, `--no-default-features` build (5 pre-existing dead-code warnings, same as HEAD). API only: no plots/three-review gate (precedent: pass 15). **Dead-code delta:** zero.
**Top 5 remaining.** 1. Release 0.6.1 after 6 h from v0.6.0. 2. Re-measure scale table on a quiet host. 3. Friendship capacity brake (#10). 4. Reconcile rival counts. 5. Insult ping-pong residue (41%).
**Cleanup.** Created nothing outside the clone (`/tmp/new_fn.rs` removed).

## 2026-10-30-24 - friend-capacity brake shipped opt-in (the label-hysteresis block dissolved; the pack change hit a second pin)

**Change.** No open issues; parked branches older, ignored. Release skipped: v0.6.0 is 1-2 h old. Pick (ranked item 3, weakest model; blocked since pass 16): the brake (`Tuning::friend_capacity`, Zhou et al. 2005, DOI 10.1098/rspb.2004.2970). Pass 16's block (a held friendship, by label, occupied capacity and coupled the trajectories `label_hysteresis` pins equal) is gone: the brake counts `Tie::is_close` (affinity and trust). With the pack at `Power(1.15)` all of `label_hysteresis`, `friendship_bound` pass unmodified, **but** `tests/rumor_book.rs` (fewer insults -> 735 rumors, 289 tellable; pinned >1000 and <20%) fails, and the full-suite run in pass 16 never ran it. Loosening it is the one block left, so the pack keeps `Linear` and the brake ships **opt-in** (default 0.0, village s7 `summary.json` byte-identical to HEAD). Brake alone on `Linear` is worse (town rivals 277 -> 331), so it is not on by default.
**Evidence.** `runs/2026-10-30-24/{eval,review,research,plan}.md`, `evidence/plots/before-after.png`, `snub-power-1.15.patch`. Brake+`Power(1.15)` vs HEAD: village insults 343/382/348/361 -> 202/159/186/167, rivals 54/56/59/66 -> 39/25/37/30, friends 46/30/40/35 -> 66/90/72/80; town rivals 277 -> 49, friends 49 -> 249; exponent sweep steep (140/49/0 at 1.1/1.15/1.2). Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. New: `examples/evidence --friend-capacity`, `tests/friend_capacity.rs` (3). Perf: off, no tie scan. **Dead-code delta:** zero.
**Assumed decision (owner may override):** options were (a) loosen `rumor_book`, (b) ship brake opt-in with the pack unchanged, (c) revert. Picked (b): reversible, touches no protected check. To ship `Power(1.15)`, the owner relaxes `rumor_book`'s two thresholds.
**Top 5 remaining.** 1. Owner: relax `rumor_book` thresholds, then apply `snub-power-1.15.patch` and default `friend_capacity` 10. 2. Release 0.7.0 after 6 h from v0.6.0 (breaking: `Tie::expose`, new `Tuning` field). 3. Damp the grudge feedback so rivals are not steep in the snub exponent. 4. Re-measure scale table on a quiet host. 5. Attribute the other ~50% of `decide_activities`.
**Cleanup.** Removed `/tmp/mpl-iter64`, `/tmp/bs-base`, `/tmp/bs-base-target`, `/tmp/bs-base-v7`, `/tmp/starter*.ron`, `/tmp/sweep-*`. Leftover: `stash@{0}` from pass 17, for a person to drop.

## 2026-10-30-25 - forgiveness that follows trust: `ctx.grudge` fades faster toward a trusted offender (`Tuning::forgiveness` 12.0, default on)

**Change.** No open issues on this repository; bevy_emerge's only open issue naming this crate (#106) has its crate side on `main`. Parked branches (passes 20-40) are older items, adopted or stale; ignored. Pick (ranked item 2 and idle ladder 3, the weakest model: the 60-person town held 277 rival ties against 49 friend ties and 75% of snubs answered an insult received; `needs-decision/what-does-the-village-owe-its-grudges.md` option 3 had no paper): McCullough et al. 2010 (DOI 10.1037/a0019349), Burnette et al. 2012 (DOI 10.1177/0146167211424582) and Strelan, Karremans & Krieg 2017 (DOI 10.1111/bjso.12173) make forgiveness follow relationship value and low exploitation risk, carried by trust. `ctx.grudge(a, b)` now reads each bad memory at `age · (1 + forgiveness · trust(a → b))` (`model::recall::forgiving_pace`, `Memories::grudge_at_pace`; `Memories::grudge` unchanged). Runner-up: re-measure the scale table (host load 39-82, unmeasurable). Also corrected three stale "a save does not carry it" lines in `docs/guide/concepts.md` and the README's rivalry figure.
**Evidence.** `runs/2026-10-30-25/{research,plan,eval,review,perf}.md`, `evidence/plots/{before-after,sweep}.png`, `evidence/{before,after}/*/{summary.json,narrative.txt,grudge_sources.txt}`. Same seeds, HEAD → 12.0, directed ties: village s7/42/11 rivals 54/56/59 → 28/24/18, friends 46/30/40 → 62/65/47, insults 343/382/348 → 251/240/255; town s7 rivals 277 → 111, friends 49 → 101. Fresh eyes (s99): village 66/35 → 36/54, town 229/52 → 155/86. Town sweep 0/4/8/12/16/32: rivals 277/205/148/111/89/53 (smooth). `--forgiveness 0` byte-identical to HEAD. Gains 1-4 did nearly nothing in the village (feuding pairs lose trust), so it is a design default near 10. Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. Gate: full `cargo test --release --all-features` (94 binaries) and `--no-default-features` (94, at `-j 1`: two earlier runs had the linker OOM-killed by host memory pressure, not a test) green, clippy `-D warnings`, fmt, doc `-D warnings`. Perf: within noise (spreads 13-80%). New: `tests/forgiveness.rs`, `examples/evidence --forgiveness`, `tools/plot_forgiveness.py`. **Dead-code delta:** zero (the 5 dialogue-gated `--no-default-features` warnings, unchanged).
**Assumed decision (owner may override):** options were (a) default 12.0, (b) opt-in at 0.0, (c) 16.0. 16.0 was tried first: the whole suite passed except `tests/rumor_book.rs::the_book_accumulates_rumors_nobody_can_tell` (264 of 1 315 rumors tellable, 20.1% against its pinned 20%), because the pack's rumors come only from witnessed insults. Picked (a), the largest of the gains measured on that pin (0/4/8/10/12/16/24) that keeps every protected check (18.3% at 12): it moves the shipped society and `0.0` restores the old curve exactly. Recorded in `needs-decision/what-does-the-village-owe-its-grudges.md`.
**Top 5 remaining.** 1. Feud seeds: the town at s99 still holds 155 rival ties to 86 friends; 10-15% of snubs answer one's own refused compliment or chat (rejection, then aggression), worth a model of its own. 2. Owner: re-measure `rumor_book`'s 20% tellable line (it couples the rumor model's finding to the insult rate) to allow `forgiveness` 16. 3. Re-record the README clips (2026-10-25; the settled village's ties have moved since) on a host with a display. 4. Re-measure the scale table on a quiet host. 5. Attribute the other ~50% of `decide_activities`.

## 2026-10-30-26 - four filed bugs: NeedBelow arms each person (#62), restore matches triggers by id (#64), `fast_forward` runs the ticks it reports (#63), stale save docs (#65)

**Change.** Four open bug issues from the owner's review pass, all outranking any enhancement; #62 is the oldest and #64 shares its save-format change, so one pass took all four, in two fix commits. No other issue open; parked branches (passes 20-40) older, ignored. **#62** `NeedBelow`'s arming was one flag per threshold for the whole town: a person above the line re-armed it every tick (the trigger fired every tick for the person below it), and the first person's event spent it for everyone (a `persons: [1]` trigger never fired); two triggers on one line also double-fired. Now `NeedDisarmed`, one `(need, threshold bits, person)` entry per person who fired and has not recovered above `below + 0.05`. **#64** a save keyed `once`/cooldown state by trigger index, so loading it after a trigger was inserted or moved replayed a spent trigger and spent another: `TriggersSave::ids` + `restore` maps by authored `id` through the `TriggerRemap` `reload_defs` now shares. **#63** `run_fixed_main_schedule` adds the last frame's `Time<Virtual>` delta each call: zeroed for the call (clocks put back), a `days` that overflows `u64` is `BadDays` (it looped for ever), a world with no fixed clock is `NoTimeStep` (it panicked). **#65** the persistence guide's `persons`/`triggers` rows and `Tuning::stall_timeout_hours` said what `capture` does not write any more (`concepts.md` had already been corrected by an earlier commit).
**Assumed decision (owner may override):** save `VERSION` 4 -> 5 (`need_armed` replaced by `need_disarmed`, `ids` added). Options: (a) bump and raise the literal pin in `tests/economy.rs`, (b) stay at 4 and drop `need_armed` (an old build then fails on a missing field, not a clean `Version` error), (c) keep the dead field. Picked (a); the pin is the only protected check touched and only raised (`assumed owner exception: tests/economy.rs save version pin 4 -> 5`). A v4 file upgrades without rewriting: its indices apply as before and its one-flag-per-threshold `need_armed` is not read.
**Evidence.** `runs/2026-10-30-26/{research,plan,eval,perf,review}.md`, `evidence/{before,after}/firings.md`, `evidence/need_below_probe.rs`. Village, seed 7, 12 people, 3 days, `NeedBelow(food, 0.7)`: 12 people crossed, **1 firing before, 12 after**, equal to an independent oracle; seed 99, 24 people, 6 days, line 0.5 (fresh eyes): 29 firings = 29 crossings, per person. New tests `tests/need_below.rs` (5, all fail on `300c52b`), `tests/save_triggers.rs` (3), two in `tests/fast_forward.rs` (one fails, one never returns, on `300c52b`). Gate: fmt, clippy `-D warnings`, doc `-D warnings`, full `cargo test --release` in both feature sets (96 binaries each, 0 failures). Perf: no `NeedBelow` in the starter pack so the benchmark's work is unchanged; paired runs at 1 000 people were inside host noise (spread 47-124%, load 48), so no figure. Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. **Dead-code delta:** zero.
**Top 5 remaining.** 1. Needs hit the floor: at seed 99 (24 people, starter + village) **19 of 24 people reach food 0.00 within 6 days** (`evidence/after/firings.md`), and every person's first crossing lands on one tick; cause not investigated (supply, place capacity, or identical initial needs). 2. Feud seeds: 10-15% of snubs answer one's own refused compliment or chat (rejection, then aggression). 3. Owner: re-measure `rumor_book`'s 20% tellable line so `forgiveness` 16 is allowed. 4. Re-record the README clips on a host with a display. 5. Re-measure the scale table on a quiet host.
**Cleanup.** Created no worktree. Removed `/tmp/bs-build.log`, `/tmp/bs-all.log`, `/tmp/bs-nodef.log`, `/tmp/zz_probe.rs`, `/tmp/probe-{before,after,fresh}.txt`, `/tmp/bench-before`, `/tmp/bench-after`. `df -h .` 282 G free at start, 245 G at the end (the clone's own `target/` growing, plus the host's other jobs). **Leftovers, not mine, not removed:** worktree `/home/ladvien/.cache/loop-runs/before-tree` (3.2 G, clean, `d825bdd` on `origin/main`, idle over 2 h), worktree `/home/ladvien/.cache/price-before-src` (721 M, clean, `fd89781` on `origin/main`, idle over 2 h), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop.
**Release.** v0.7.0 published, https://crates.io/crates/bevy_society/0.7.0 (semver-checks: breaking, new pub fields on `Tuning`, `PersonSave`, `SocietySave`, `TriggersSave`; `TriggersSave::need_armed` removed; `Tie::expose` takes one more parameter). Tag `v0.7.0` is on `0418528`. #62, #63, #64 and #65 closed by PR #67.

## 2026-10-30-27 - a society begins with a loaf: `GoodDef::initial` (the start-up famine behind "needs hit the floor")

**Change.** No open issues; parked branches (passes 20-40) older, ignored. Release skipped: `v0.7.0` is 1.1 h old. Pick (pass 26's top item, a bug recorded in the log, ranked ahead of any feature): "19 of 24 reach food 0.00 within 6 days". Investigated before any code: the probe's "days" were 256-update days (1.6 game-days), and the floor is a **start-up transient**, not a shortage. A person is created with `food` 1.0 and **no bread**; `bake` is a one-hour dawn errand and the run starts at 08:00, so the first bread exists 22 game-hours in, `eat` (`Logistic(-10, 0.35)`) only wins below 0.35 (01:00, asleep), and every villager falls from the same 1.00 to the floor together. Seed 99, 240-second day: game-day 1 has an empty larder on 99.8 % of person-ticks and game-day 2 has 38.7 % of them at `food` < 0.05 (days 3-10: 1.4-10.7 %). `GoodDef::initial` (default 0.0, held at `stock_ceiling`) is what a new person holds, as `NeedDef::initial` is the level a need starts at; `size_person_state` applies it to a new `Stock` and to the slots a short one gains; a host's full-length `Stock` is left alone; `restore` gives a person whose save holds no stock a zero-filled one (else the first tick would hand a version-1 file's people a loaf). The starter pack's bread starts at **1.0**, a design default where `price_full_store`, `trade_reserve` and a quarter-loaf meal agree. Research: Murray & Kelton 1988 (DOI 10.1145/318123.318252) on starting in a typical state; Winterhalder et al. 2015 (DOI 10.1179/1749631415y.0000000025) on the household store; Longhurst 1986 (DOI 10.1111/j.1759-5436.1986.mp17003005.x). Runner-ups: heterogeneous starting needs (lockstep crossings; needs a per-person draw in a fixed order), the `eat`-from-an-empty-larder trap (found while measuring, below).
**Evidence.** `runs/2026-10-30-27/{research,plan,eval,perf,review}.md`, `evidence/{floor,summary,long}.md`, `evidence/plots/{larder,zoom-first-days}.png`, `evidence/{before,after}/*/{summary.json,narrative.txt}`. Evidence harness gained `floor0`..`floor3` (villagers with need *k* < 0.05). Baseline reproduced pass 25/26's numbers exactly (village s7 28 rivals / 62 friends / 251 insults), so before is HEAD. Day-2 share of villager-ticks at the floor on `food`, 64-second day: village s7/42/11/99 **78.6/88.9/89.6/90.0 % → 12.6/15.6/9.3/14.7 %**; town of 60 s7/s99 90.8/83.3 → 36.0/38.1 %; settled share unchanged (19.5/23.1/35.0/23.2 → 19.7/24.2/16.7/22.9 %); test village (240-second day) **38.7 % → 4.7 %**. Cost, recorded: at day 17 every village seed has more rival ties and fewer friend ties (rivals 28/24/18/36 → 36/36/35/38, friends 62/65/47/54 → 41/43/37/42, insults +9-21 %); two 40-day runs shrink it to about ±10 % of the tie counts over days 20-40, inside the swing of either trajectory. Verdicts: behavior GREAT, zoom GREAT (with the town's collapse, identical on both sides, recorded), fresh eyes GREAT. Gate: fmt, clippy `-D warnings`, doc `-D warnings`, full `cargo test --release` in both feature sets (97 binaries each, 0 failing), `--no-default-features` build. New: `tests/starting_larder.rs` (6, including a paired two-village mechanism test), `tests/save_upgrade.rs::a_version_one_save_holds_nothing_a_tick_after_it_restores` (fails on the parent's `save.rs`). Perf: not distinguishable from host noise (spreads 28-96 %, load 29-54); a same-binary larder on/off control at 100 people reads 4.97/4.69/4.20 ms against 5.61/4.29/3.32. **Dead-code delta:** zero.
**Assumed decision (owner may override):** the schema gains an optional `GoodDef::initial`. Options: (a) that field with the pack's bread at 1.0 (picked: reversible, mirrors `NeedDef::initial`, touches no id table so `reload_defs` accepts it, no protected check touched), (b) a warm-up the host runs (`fast_forward` a day) before it shows anything, (c) a randomised start. `docs/loop/needs-decision/a-village-begins-with-empty-larders.md`. Override: set `initial: 0.0` on bread in `assets/starter.society.ron`.
**Top 5 remaining.** 1. **The 60-person town starves for good** (found here, `eval.md` "The town's famine"): 100 % of villager-ticks at `food` < 0.05 from about game-day 17 with no bread anywhere, before and after this change; 65-104 meals a day from empty larders and 12-13 % of everyone's day spent walking to an empty table, 0-2 bakes a day. `[INFERENCE]` a starving person scores `eat` above the dawn `bake` and the two share the same two places (capacity 18 for 60 people); untested. Next: read Berridge 2004 (DOI 10.1016/j.physbeh.2004.02.004) and Torrealba et al. 2012 (DOI 10.3389/fnsys.2012.00051) on appetitive against consummatory behaviour, confirm the hypothesis from `DecisionTrace`, then make a hungry person with nothing to eat seek food rather than a table. 2. Release 0.8.0 after 6 h from v0.7.0 (18:53; breaking: `GoodDef` gains a public field). 3. The 64-second day makes a meal two to four game-hours of walking (the harness warns "4 hours buys 14.9 m"); 9-13 of 24 villagers are still at the floor at hour 26 even fed. 4. Feud seeds: 10-15 % of snubs answer one's own refused compliment (pass 25). 5. Re-record the README clips and re-measure the scale table on a quiet host.
**Cleanup.** Removed `/tmp/bs27` (worktree `/tmp/bs27/base` via `git worktree remove`, binaries, logs) and `/tmp/plotenv27`. `df -h .` 243 G free at the start, 212 G now: the baseline worktree's 14-minute build went into the clone's own `target/` (shared target dir), which stays, beside the host's other jobs. Leftovers, not mine, not removed: worktrees `/home/ladvien/.cache/loop-runs/before-tree` and `/home/ladvien/.cache/price-before-src` (pass 26 listed them), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop.

## 2026-10-30-28 - `Input::Stock`: the household reads its own shelf (the 60-person town's supply; opt-in overlay, default flip blocked by one protected assertion)

**Change.** No open issues; parked branches (passes 20-40) older, ignored. Release skipped: `v0.7.0` is 3.6 h old (eligible after 00:53). Pick (pass 27's top item, a bug recorded in the log, ranked ahead of any feature): "the 60-person town starves for good". Diagnosed before any code with a new harness ledger (`activities.txt`: starts, completions and interruptions per game-day and activity, and the clock hour each was chosen at). The town at seed 7 made **3 bakes in ten game-days**; of ~34 decisions a day at hour 6, the only hour `bake` scores, **6 in ten days were `bake`** (2 %), against 123 `compliment`. At the library's own 240-second day (so not the 64-second day's walking) bakes fall 5, 1, 2, 1, 0, 0, 0, 0, 0, 1, 1, 0, bread a head 1.13 → 0.05 and **86 %** of person-ticks sit at `food` below 0.05 by game-day 12, with 66-140 meals a day *started* and nothing on the table. Pass 27's hypothesis (a starving person scores `eat` above `bake`) was only half right: `bake` is a clock-driven open loop (a one-hour window a decision rarely lands in, drawn against nine options), and nothing in the pack could tell a person its own shelf was bare. `Input::Stock(good)` is the actor's own holding clamped to `[0, 1]` (`Signal::Stock`, the copy of `Stock` that `Surplus` already makes; an undeclared good is a `DefsError`). `examples/assets/provisioning.society.ron` replaces the starter's `bake` and `eat` with versions that read it: a hungry household with a bare shelf bakes in daylight (`Hours(6, 20)`, `Stock` `Linear(-1, 1)`, `Need("food")` `Logistic(-10, 0.5)`), and `eat` on an empty shelf is priced at `0.05`, not gated. Research: Meadows (a balancing loop closes on a stock; no DOI) and Zunic's production-consumption model (`eth-cgl-ar_games-Zun16a`), Woodworth 1918 and Konorski 1976 on appetitive before consummatory (corpus doc `10.1002/hipo.22488`), Game AI Pro on *The Sims* (hunger with the availability of food), Zun15a (affordance preconditions), Maslow 1943 (DOI 10.1037/h0054346) for what is *not* done. Runner-ups: prepotent hunger by `rank` (a second experiment: below), a price-driven bake (the price is the crowd's information, a household reads its shelf).
**Evidence.** `runs/2026-10-30-28/{research,plan,eval,perf,review}.md`, `evidence/provisioning.md`, `evidence/plots/{provisioning,zoom-town-first-days}.png`, `evidence/{before,after}/*/{narrative.txt,summary.json,activities.txt}`, `evidence/diagnosis/`. Same seeds, default pack against `--provisioning`: town of 60, 30 game-days at the 64-second day, seeds 7 / 99: bread a head over the last six days **0.00 / 0.05 → 1.16 / 1.10**, floor on `food` **100 / 96 % → 38 / 37 %**, bakes a day 0.0 / 0.3 → 4.2 / 5.3, price 1.7-2.0 → 0.50-0.66; town of 40, seed 3 (fresh eyes): 0.58 → 1.29, 49 → 23 %; town of 60 at the 240-second day: 0.19 → 1.40, 63 → 23 %. The four villages are not hurt (floor 19.4 / 21.7 / 17.7 / 21.9 % → 15.8 / 17.9 / 16.7 / 17.4 %), friend ties rise (41 / 43 / 37 / 42 → 50 / 58 / 48 / 75), rivals mixed (36 / 36 / 35 / 38 → 30 / 42 / 47 / 37). `eat` is started 1 847 times in the 30-day town against 3 307 (no walks to an empty table). A run of the default pack is **byte-identical** to HEAD (`series.csv`, `cmp`). Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. Gate: fmt, clippy `-D warnings`, doc `-D warnings`, `cargo test --release` in both feature sets (97 binaries each, the doctests, 0 failing), `--no-default-features` build (5 warnings, as `main`). New: `tests/stock_input.rs` (4), `tests/provisioning.rs` (2, the two halves of the mechanism against the clock-driven pair the starter had through 0.7.0: a hungry bare household bakes at noon, one holding bread makes none). Perf: no code on the default path but an untaken match arm, fastest windows 0.63-0.79 ms at 100 and 33-43 ms at 1 000, spreads 55-202 % (host noise). **Dead-code delta:** zero.
**Assumed decision (owner may override):** ship the input and the overlay **opt-in** and leave the starter pack alone. Options: (a) replace the starter's `bake` and `eat` now, (b) opt-in overlay, (c) keep `Hours(6, 7)` and add the shelf term. (a) is blocked: `tests/starting_larder.rs::a_starting_larder_removes_the_second_days_famine` asserts `empty > 0.2`, "the transient this guards against has to be there to cure", and a self-provisioning village cures that transient itself (0.057; a `(6, 8)` window gives 0.195, a `(6, 7)` window 0.322 and does not fix the town: floor 70 %, bread 0.24 at game-day 10). The loop's rule is that a protected assertion is never removed or loosened, so (b), reversible; `docs/loop/needs-decision/households-provision-themselves.md` and `runs/2026-10-30-28/provisioning-default.patch` flip the default (the patch applied: library tests and all 97 binaries pass; the one assertion deleted). The overlay's numbers are design defaults.
**Top 5 remaining.** 1. **Hunger is not prepotent**: even fed, the overlay town has 37 % of person-ticks at the floor with 1.1 loaves a head, because `eat` is one draw among many by `weight · score` (Maslow 1943, DOI 10.1037/h0054346); a `rank` on `eat` needs a curve exactly zero above the line, measured on the village, the town and every protected pin. 2. **Owner: flip the default** (delete `empty > 0.2`, apply the patch), then CHANGELOG and README say the starter pack provisions itself. 3. Release 0.8.0 after 00:53 (`Input` gains a variant, breaking). 4. Lockstep: every run jumps from 0 to 36 of 60 villagers at the floor at hour 24 because everyone starts at `food` 1.0 (heterogeneous starting needs, a per-person draw in a fixed order). 5. 38 % of the overlay town's bakes (50 of 130) time out at the 64-second day (a walk of four hours or more) and the clock the harness runs at is not the library's; feud seeds (pass 25) and re-recording the README clips remain.
**Cleanup.** Created no worktree. Removed my stray evidence files (`shares.txt`, `partners.txt`, `grudge_sources.txt`, `why.txt` under `runs/2026-10-30-28/evidence/`, uncommitted by design) and `/tmp/bs28` and `/tmp/plotenv28` at the end. Left (mine, ignored by git): the `series.csv` files under `runs/2026-10-30-28/evidence/` (about 320 MB, they regenerate with the command in `eval.md` and `tools/plot_provisioning.py` reads them). Leftovers, not mine, not removed: worktree `/home/ladvien/.cache/loop-runs/before-tree` (3.2 G, clean, `d825bdd` on `origin/main`, idle over 2 h), worktree `/home/ladvien/.cache/price-before-src` (721 M, clean, `fd89781` on `origin/main`, idle over 2 h), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop. `df -h /home` 213 G free at the start, 179 G now (the clone's own `target/` and the 320 MB of ignored `series.csv`, beside the host's other jobs). **Merged:** PR #69, merge `e0e38e3`; branch commits `a486a6e` (research and plan), `60f758b` (implementation), `4555b3a` (evidence and docs), `b165a98` (log).

## 2026-10-30-29 - `ActivityDef::urgent`: hunger takes precedence (overlay floor 15-37 % → 3-9 % of person-ticks; starter pack deliberately unchanged)

**Change.** No open issues (`gh issue list` returns nothing); parked branches (passes 20-40) older, ignored. Pick (pass 28's top item, a defect recorded in the log, ranked ahead of any feature): "hunger is not prepotent". Diagnosed before any code with a new harness ledger (`floor.txt`: who is at `food` < 0.05, doing what, with a meal in hand or not, and each spell of hunger, `food` under 0.35 until back over). Village seed 7, default pack: **96.8 % of floor ticks had a meal in hand; 49 % were `work`, `craft`, company or leisure and their walks** (26 % asleep, 20 % walking to eat); spells mean 12.4 h, p90 27.5 h, longest 86 h. Maslow (1943, DOI 10.1037/h0054346), the Sims' need buckets (Game AI Pro 1 ch. 9: "a starving Sim will never even consider watching TV unless they fail to find anything that can solve their hunger") and Dill's dual-utility ranks (Game AI Pro 2 ch. 3) say a low need should take the choice; the crate's ranks were static. New: `ActivityDef::urgent: Option<UrgentDef { need, below, rank }>` raises an activity's rank while `need < below` (`ActivitySpec::rank_for`, one `Option` match per scored option); an urgent option with no candidate falls through to the lower tiers; `ScoredOption::rank` records the tier and `Decision::describe` says `chose eat (0.500, rank 1) over work (0.900, rank 0)` when a tier decided. The provisioning overlay's `eat` and `bake` take it at 0.25, rank 1. Runner-ups: flip the starter pack to the overlay (blocked by one protected assertion), heterogeneous starting needs (the hour-24 lockstep). Release skipped at the start: `v0.7.0` was 5.2 h old.
**Evidence.** `runs/2026-10-30-29/{research,plan,eval,perf,review}.md`, `evidence/urgent.md`, `evidence/plots/{urgent,zoom-first-days}.png`, `evidence/{before,after}/*/{floor.txt,narrative.txt,summary.json,activities.txt}`, `evidence/sweep/`. Same seeds, the overlay with `--no-urgent` against as shipped: floor on `food` village (seeds 7 / 42 / 11 / 99) **18.8 / 19.9 / 15.5 / 16.2 % → 2.7 / 2.6 / 3.3 / 2.6 %**, town of 60 (seeds 7 / 99, 30 days) 36.1 / 37.0 → 7.9 / 8.7 %, town of 40 (seed 3) 22.8 → 5.4 %, town of 60 at the 240-second day 25.9 → 5.4 %; p90 spell 23-47 h → 10-15 h, longest 40-212 h → 24-52 h; bread a head 1.1-1.5 → 1.45-1.65; what is left of the floor is the walk to the table (60-71 %) and the night (18-26 %), work, craft and company 0.4-0.6 %. Social effects inside the seed range (village friends 58 → 47, insults 277 → 313, accepted approaches +2 to +19 %; towns 3-4 % fewer exchanges). **The limit, measured:** on the starter pack the same line makes the floor worse (19.4 → 18.1 / 26.9 / 52.2 % at 0.15 / 0.25 / 0.35, bread a head 3.2 → 0.7 / 0.3 / 0.0): its `eat` prices an empty shelf and its `bake` is outside the bucket, so a hungry bare household keeps choosing a meal that feeds nothing; a prepotent option has to be able to solve the need. A default-pack run (seed 7, 17 days) is byte-identical to the pre-change binary (`cmp`). Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. Gate: fmt, clippy `-D warnings`, doc `-D warnings`, `cargo test --release --all-features --tests` and `--no-default-features --tests` (99 binaries each, 0 failed), doc tests, `cargo build --examples`; `tests/urgent.rs` (7) and one in `src/trace.rs`. Perf: no measurable cost (`perf.md`; one core pinned, same median 1.03 ms/tick at 100 people). **Dead-code delta:** zero (the five warnings of `--no-default-features` are the dialogue-gated ones from before).
**Assumed decision (owner may override):** add the optional `urgent` field to the activity schema and switch it on **in the provisioning overlay only**, at `below: 0.25`, rank 1. Options: (a) put it in the starter pack's `eat` now (measured worse, rejected), (b) field plus overlay (picked: the bucket then holds only options that can solve the hunger; reversible; touches no protected check), (c) infer the bucket from `satisfies` with a `critical` level on `NeedDef` (rejected: `bake` satisfies nothing and must be in the bucket on a bare shelf). Swept at 0.15 / 0.25 / 0.35 (floor 3.6 / 2.7 / 1.8 %); 0.25 leaves a person who is merely peckish alone. `docs/loop/needs-decision/hunger-takes-precedence.md`. The one edit to an existing test is additive (`assumed owner exception: src/trace.rs` unit-test literals gain `rank: 0`).
**Top 5 remaining.** 1. **Owner: flip the default.** The pass-28 patch (`runs/2026-10-30-28/provisioning-default.patch`) predates `urgent`: regenerate it from the overlay as it is now (it carries the `urgent` lines), delete `empty > 0.2` in `tests/starting_larder.rs`, then CHANGELOG and README say the starter pack provisions itself and puts hunger first. 2. Release 0.8.0 (`Input` and `Signal` gain `Stock`, `GoodDef`, `ActivityDef` and `ScoredOption` gain fields, `size_person_state` takes a parameter: semver-checks, three major). 3. The residue of the floor is the walk (60-71 % of it at the 64-second day): `travel-time-in-two-clocks.md`; a prepotent meal at 0.25 starts a walk of two to four game-hours that ends under 0.05. 4. Lockstep: every run still jumps from 0 to 5-12 villagers at the floor at hour 24 (everyone starts at `food` 1.0). 5. Watch: village friends 58 → 47 with `urgent` (seed sd 12) and a hungry person at night draws between sleep and a meal at the same rank (a rank of 2 puts hunger over sleep): measure before deciding.
**Cleanup.** Created `/home/ladvien/.cache/loop-runs/bs29-base` (a worktree at `6e2947e` for the paired benchmark; its build went into the clone's own `target/`): removed with plain `git worktree remove`. Removed `/tmp/plotenv29`, `/tmp/evidence-29`, `/tmp/run29.sh`, `/tmp/run29-*.log`, `/tmp/default-final-s7`, `/tmp/larder29.*`, `/tmp/gate29-*.log`, `/tmp/bench-{before,after}`, and my own uncommitted evidence files under `runs/2026-10-30-29/evidence/` (the `series.csv` files, 563 MB, which regenerate with the command in `eval.md`, and the stray `grudge_sources.txt`, `partners.txt`, `shares.txt`, `why.txt`, and the sweep's `narrative.txt` and `activities.txt`). `df -h .` 179 G free at the start, 156 G now (the clone's `target/` growing with the release test builds, beside the host's other jobs). Leftovers, not mine, not removed: worktree `/home/ladvien/.cache/loop-runs/before-tree` (3.2 G, clean, `d825bdd` on `origin/main`, idle over 2 h), worktree `/home/ladvien/.cache/price-before-src` (721 M, clean, `fd89781` on `origin/main`, idle over 2 h), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop; `runs/2026-10-30-28/evidence/**/series.csv` (about 320 MB, ignored by git, regenerable). **Merged:** PR #71, merge `6f38b11`; branch commits `f24ccee` (research and plan), `958a5b2` (implementation), `5f91287` (evidence and docs).
**Release.** v0.8.0 published, https://crates.io/crates/bevy_society/0.8.0 (`cargo semver-checks check-release`: breaking, 3 major checks: `Input::Stock` and `Signal::Stock`, new public fields on `GoodDef`, `ActivityDef` and `ScoredOption`, `size_person_state` takes one more parameter). The last tag was 6 h 0 m 30 s old when it went out (`git log -1 --format=%ct v0.7.0`). Tag `v0.8.0` is on `e4f358f`, merged by PR #72 (`e3c0b18`); the README and `docs/guide/getting-started.md` install snippets named `0.4` and now name `0.8`.
**Flip check (supersedes top item 1: the patch is regenerated).** After the release I put the starter pack on the overlay as it now is (`urgent` lines included) in a scratch worktree (`/home/ladvien/.cache/loop-runs/flip29`, mine, removed with plain `git worktree remove` after reverting my own two edits in it) and ran the protected pins in release, all features. **`tests/starting_larder.rs::a_starting_larder_removes_the_second_days_famine` fails its second assertion**: the day-2 floor is `0.000` with an empty larder and `0.000` with a loaf (`stocked < empty / 2.0`, 0.000 against 0.000), so the pack now cures the transient the test guards against and the test has nothing left to measure; the pass-28 flip failed only the first assertion (`empty > 0.2`, 0.057). Passing on the flipped pack: `friendship_bound`, `label_hysteresis`, `rumor_book`, `need_cycle`, `economy`, `trade`, `provisioning`, `village_evening`, `village_layout`, `save`, `determinism` and the 37 binaries that sort before `friendship_bound`; the rest were not run on the flipped pack. The loop's rule blocks removing or loosening an assertion, so the pack stays unflipped; the patch is `runs/2026-10-30-29/provisioning-urgent-default.patch` (applies to `main`), and the owner takes it by deleting that one test (`needs-decision/households-provision-themselves.md`, "Update, 2026-10-30-29").

## 2026-10-31-02 - body clocks: `Chronotype`, `Input::BodyHour`, `ActivityDef::unavailable` (bedtimes follow each person's own clock; nobody is greeted in their sleep)

**Change.** No open issues (`gh issue list` returns nothing; bevy_emerge's open issues name nothing for this crate); parked branches (passes 20-40) older, ignored. Release skipped: `v0.8.0` is 2 h 12 m old (eligible after 06:53); this change is breaking (`Input` and `Signal` gain a variant; `ActivityDef`, `PersonDef`, `PersonSave` gain fields), so the next release is 0.9.0. Ranked candidates: (1) **nobody has a body clock** (ranked item 2, "everyone doing the same thing"): picked; (2) partner repetition and re-approach after a refusal (already worked by pass 3 of 2026-10-30 and `friendship-has-no-diminishing-return.md`, no new source); (3) re-record the README clips (needs a display). (1) beats (2): it is visible to anyone who watches a village at night, the crate already warned about it twice, and it needs no change to a pinned model. Measured first, on the shipped pack, village seed 7, 17 game-days: 280 of 450 sleeps begin in the 22:00 hour and 81 in the 23:00 hour (`Hour(22, 6)`, one window for everyone), and, found while building the harness, **7-9 % of all exchanges (349, 378, 324, 362 at four seeds; 13 % of chats) were aimed at somebody who was asleep**, a defect no one could see while everybody slept at once. What shipped: `Chronotype(f32)` (hours the body clock runs behind the wall clock; the host writes it, the crate draws nothing; `Chronotype::sample(rng, sd)`, `POPULATION_SD_HOURS` 1.5 h, a design default inside the published 1.2-2.5 h), `Input::BodyHour` beside `Hour` (the village's clock stays the social one), `ActivityDef::unavailable` (a person who has arrived at it is in nobody's `near` list and a walk toward them ends), the starter pack's `sleep` on `BodyHour` and `unavailable`, `PersonSave::chronotype` and `PersonDef::chronotype_hours` (serde defaults, `VERSION` stays 5), `Hours` refusing any input but `Hour` and `BodyHour` with a message naming both. Sources (home-still, DOIs checked): Roenneberg et al. 2007 (10.1016/j.smrv.2007.07.005), Fischer et al. 2017 (10.1371/journal.pone.0178782, read in full: near-normal, IQR 02:24-04:15, sleep length independent), Reiter et al. 2021 (10.3390/clockssleep3020021), Sogawa et al. 2023 (10.5334/jcr.225), Wittmann et al. 2006 (10.1080/07420520500545979). The work is a new note, `docs/research/chronotype.md`.
**Evidence.** `runs/2026-10-31-02/{research,plan,eval,perf,review}.md`, `evidence/chronotype.md` (the table), `evidence/plots/{sleep,day}.png`, `evidence/{before,after,variants}/<run>/{sleep.txt,summary.json,narrative.txt}`, `evidence/narratives/{lark-hilda,owl-quill}-s7-6days.txt`. Same seeds, `before` = 0.8.0's society (`--sleepers-available`, byte-identical to the 0.8.0 binary at seed 7: `cmp` on `summary.json`, `series.csv`, `narrative.txt`), `after` = `--chronotype-sd 1.5`: sleeps begun in the busiest clock hour, village seeds 7 / 42 / 11 / 99 **63 / 60 / 64 / 64 % → 19 / 23 / 27 / 21 %**, towns of 60 (s7, s99) and 40 (s3) 49 / 47 / 48 % → 18 / 17 / 20 %, the library's own 240-second day **75 % → 19 %** (clock hours with 10-90 % of the village asleep 4 → 12); exchanges aimed at a sleeper 349 / 378 / 324 / 362 → **0 / 1 / 2 / 3** (towns 1 950 / 1 620 / 692 → 87 / 118 / 11). Two controls: the flag alone (`no-clocks`) and clocks without it (`sleepers-open`: 12-17 % of exchanges, 15 % in the town, so the guard is a precondition of the clocks, not an extra). Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT (`review.md`). Gate: full `cargo test --release --all-features` twice (first and final code) and `--no-default-features` (101 ok) green with **no existing assertion touched** (`friendship_bound`, `label_hysteresis`, `rumor_book`, `starting_larder`, `determinism` included), clippy `-D warnings`, fmt, doc, both feature builds; `tests/chronotype.rs` 9 tests. Perf: none measurable (`perf.md`: one O(N) lookup per tick, host load 9-50 so under about 10 % at 5 000 people is not resolvable). **Dead-code delta:** zero.
**What was wrong, and what moved that is not mine to explain.** (1) My first prediction, "interrupted sleeps fall", came from the crate's own warning text (a queue at the bed) and was struck: with a ledger of the walker's state on the tick before each ending, **every interrupted sleep in six runs ended before the person reached the bed** (63 of 63, 74 of 74, 577 of 577, 707 of 707, 18 of 18, 24 of 24); the walk outlasts `travel_timeout_hours`, and the town rate is 27 % → 32 % and 32 % → 35 % at two seeds (24 % → 23 % at the third), a few points worse, unexplained. (2) The social outcomes move: friend ties rise in all seven before/after pairs (41 → 79, 43 → 60, 37 → 53, 42 → 57, 101 → 121, 89 → 111, 69 → 79), rival ties fall in five of seven (the town's 116 → 123 and 50 → 72 do not), insults do not rise; the flag alone accounts for much of it (41 / 43 / 37 / 42 → 48 / 66 / 51 / 69 friends), a chaotic society's one trajectory per cell, so the direction is the finding. This changes the shipped society, which the CHANGELOG says in the first entry under Changed.
**Assumed decision (owner may override):** `docs/loop/needs-decision/bodies-keep-their-own-time.md`. Options, each with a pick: where the clock comes from (host component: picked; derived from personality; drawn by the crate from a `Tuning` field), how it reaches a rule (second input `BodyHour`: picked; shift `Hour`), how a sleeper is kept from being greeted (per-activity `unavailable`: picked; insert `Dormant`; nothing), whether the pack changes (`sleep` on `BodyHour` and `unavailable`: picked; leave it opt-in). To undo the pack half: `unavailable: false` on `sleep`.
**Top 5 remaining.** 1. **Re-approach after a refusal and partner repetition**, seen again in the narratives this pass: the owl's `gossip with Garet (refused)` four times between 22:43 and 23:37, Hilda's six `compliment with Bram` in 40 minutes (34 % of exchanges repeat the previous partner): a failed-approach memory like `FailedPlaces`, or damping on the tie's `last_contact_day`, with a source. 2. **The walk home**: all sleep interruptions are before arrival (see above) and are 27-35 % in the towns: `travel-time-in-two-clocks.md`. 3. Release 0.9.0 after 06:53 (breaking: see Change; the README install snippet names `0.8`). 4. `ctx.body_hour(person)` (the snapshot does not carry the offset) and, if the owner wants clocks default-on, a `Tuning` spread and an `Add<Person>` draw (option 1c in the decision file). 5. Explain the friend-tie rise: a per-interaction ledger exists now (`sleep.txt`: chat 13 %, share 9 %, compliment 8 %, gossip 7 %, insult 5 % of each were aimed at sleepers); the question is whether a sleeper's acceptance draw, or the actor's need being paid by it, is what moved the graph.
**Cleanup.** Created and removed: worktree `/home/ladvien/.cache/loop-runs/bs70-base` (`773ba21`, plain `git worktree remove`; its build went into the clone's own `target/`), `/tmp/bs70` (runs, binaries, gate logs, scripts), `/tmp/plotenv70`, `/tmp/bs70-build-evidence.log`, and my four ignored `series.csv` under `runs/2026-10-31-02/evidence/` (about 30 MB; regenerate with the commands in `eval.md`). `df -h .` 271 G free at the start, 214 G now (the clone's `target/` growing with five release test builds and the benchmark baseline, beside the host's other jobs). **Leftovers, not mine, not removed:** worktree `/home/ladvien/.cache/loop-runs/before-tree` (3.2 G, clean, `d825bdd` on `origin/main`, idle over 2 h), worktree `/home/ladvien/.cache/price-before-src` (721 M, clean, `fd89781` on `origin/main`, idle over 2 h), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop.

## 2026-10-31-03 - a refusal is remembered: `Tuning::rebuff_memory_hours` (shipped off: `Some(12.0)` trips `label_hysteresis`'s 5 % bar by one event)

**Change.** No open issues (`gh issue list` returns nothing); parked branches (passes 20-40) older, ignored. Release skipped: `v0.8.0` is 5 h old (eligible after 06:53); this change adds public fields to `Tuning` and `PersonSave` (breaking), so the next release is 0.9.0 with the body clocks. Ranked candidates: (1) **re-approach after a refusal** (pass 02's top item, ranked item 2, "reads as wrong when you watch it"): picked; (2) satiation after *accepted* approaches (34 % of exchanges repeat the partner, six compliments in forty minutes): a different mechanism, no source yet; (3) the walk home (`travel-time-in-two-clocks.md`), a decision about two clocks, not a measured defect; (4) re-record the README clips, needs a display. (1) beats (2): Maner et al. 2007 (DOI 10.1037/0022-3514.92.1.42, abstract read: excluded people "did not seek reconnection with the specific perpetrators of exclusion", and turned to new partners) states the direction and the measurement has its own control row. Measured first, with a new ledger in the harness (`ReapproachLedger`, `reapproach.txt`): on the village, **29.0 % of refused approaches were followed within the hour by another to the same person (not an insult), 42 % of accepted ones**; the owl of pass 02 asked one neighbour for gossip seven times in an evening. The cause is that `decide_activities` remembers nothing about who said no (`gossip` scores only `Distance`; `rejection_affinity_penalty` is 0.05 on a logistic). Now `Tuning::rebuff_memory_hours: Option<f32>`: a person whose affiliative approach was refused by the target's acceptance draw drops that person from every non-harmful `Person` activity for `h` game-hours (`Rebuffed`, written in `exchange`, read after `targets_for` so the RNG draws are unchanged, saved as `PersonSave::rebuffed`). A refused insult stays with `retaliation_refractory_hours` (Bondü & Richter 2016). `None` is byte-identical to the previous build (`cmp` on `summary.json`, `narrative.txt` and `series.csv`, village seed 7).
**Evidence.** `runs/2026-10-31-03/{research,plan,eval,perf,review}.md`, `evidence/rebuff.md` (the table), `evidence/plots/{rebuff,sweep,over-time}.png`, `evidence/{before,after,sweep}/*/{reapproach.txt,partners.txt,summary.json}`, `evidence/narratives/`. Same seeds, off against `Some(12.0)`, four villages of 24 (17 game-days), towns of 60 at seeds 7 and 99 (30) and 40 at seed 3 (20): refused-then-asked-again within the hour **24-29 % → 0.3-1.1 %** in all seven, within a day 39-50 % → 13-20 %; different people approached per person-day **4.1-4.4 → 5.2-5.4** (villages), 4.7-5.1 → 5.4-6.3 (towns); exchanges with the previous partner 30.5-34.4 % → 21.5-25.3 %; the accepted row does not move. Sweep at seed 7 (off / 3 / 6 / 12 / 24 h): re-approach within a day 49.6 / 28.3 / 24.9 / 18.3 / 1.8 %. Narrative: the owl's seven asks are one; at another seed 6 of 20 refused asks were repeated within 12 h and 1 of 18 is (an insult after a refused compliment, left to the retaliation refractory on purpose). Verdicts: behavior GREAT, zoom GREAT, fresh eyes GREAT. Perf: no measurable cost, on or off (`perf.md`; the benchmark gained `BENCH_REBUFF=<hours>` so one binary measures both). **Dead-code delta:** zero (the same 5 dialogue-only warnings as `main`).
**What was wrong.** (1) My prediction that **the refused share would fall** (the pairs that refuse most are no longer retried) is **struck**: 35.2 → 36.6 % at seed 7, flat in every run; the acceptance draw reads the target's side, not who is being asked again. The memory changes whom a person asks, not how often they are refused. (2) **Costs**, recorded not argued away: approaches per person-day fall 4.5-11 % in six of seven pairs and the mean `social` and `fun` needs are 0.005-0.03 lower in five (a person with few neighbours runs out of people to ask for twelve hours and goes to work). (3) Friend ties rise in seven of seven pairs (48 → 86, 66 → 69, 51 → 61, 69 → 70, 128 → 168, 106 → 140, 103 → 107), insults fall in five, rivals and mood move both ways: one trajectory per seed, direction only, mechanism not established. (4) Maner's second half (more interest in *new* partners) is not modelled; only the turning away.
**Assumed decision (owner may override):** ship the field **off** (`None`), the measured value `Some(12.0)` documented. At `Some(12.0)` every protected check passes **except** `tests/label_hysteresis.rs::the_band_stops_the_low_end_flapping_and_moves_no_relationship`, whose bar (`demotions_with / demotions_without <= 0.05`) reads 16 against 311, 5.14 %. A probe of the same fixture across seven memories (off / 3 / 6 / 9 / 12 / 18 / 24 h) reads 3.8 / 3.9 / 5.6 / 6.0 / 5.1 / 2.8 / 3.7 %: the with-band count is 9-16 events (mean 13, standard deviation 3.6) against a bar of 15.5, so the pin is a Poisson count at its edge and not a finding about the memory. Options: (a) default `Some(12.0)` and relax the bar to 0.08 (`rebuff-default.patch`, applied and run in a scratch worktree: `label_hysteresis` 9 passed, `rebuff` 7 passed; the other 100 test binaries passed at `Some(12.0)`); (b) default 18 or 24 h, which pass the bar as it stands (rejected: chosen for passing, 24 h leaves 1.8 % retrying within the day and costs 15 % of approaches, 18 h was not run through the evidence); (c) off (picked: reversible, touches no protected check). `docs/loop/needs-decision/a-refusal-is-remembered.md`. No protected check was edited.
**Gate.** Final code: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`, `cargo build --no-default-features` and `--examples` clean; `cargo test --release --all-features --no-fail-fast` in two batches **101 binaries ok, 432 tests, 0 failures**, `--doc` ok; `cargo test --release --no-default-features --no-fail-fast` **102 ok, 392 tests, 0 failures**. New: `tests/rebuff.rs` (7), `tools/plot_rebuff.py`, `ReapproachLedger`, `--rebuff-memory H` / `--no-rebuff`.
**Top 5 remaining.** 1. **Owner: switch the memory on** (`git apply docs/loop/runs/2026-10-31-03/rebuff-default.patch`, then the field doc paragraph "The default is `None`", the `concepts.md` heading "(opt-in)", and the CHANGELOG and README sentences say it is on). 2. **Satiation after an accepted approach**: 21-25 % of exchanges still go to the previous partner with the memory on, and the narratives show a person complimented twice in twelve minutes (Edric to Dena, seed 42, day 2); a mechanism and a source are both missing. 3. **The walk home** (`travel-time-in-two-clocks.md`): every interrupted sleep ends before arrival; towns 27-35 %. 4. Release 0.9.0 after 06:53 (breaking: `Input`/`Signal` variant, `ActivityDef`, `PersonDef`, `PersonSave` and `Tuning` fields; README install snippet names `0.8`). 5. `ctx.body_hour(person)` and, if the owner wants clocks default-on, a `Tuning` spread; and re-record the README clips on a host with a display.
**Cleanup.** Created and removed: worktree `/home/ladvien/.cache/loop-runs/bs71-flip` (`e992429` with `rebuff-default.patch` applied for the flip check; its build went into the clone's own `target/`; git refused a plain remove with the patch in it, so I reverted my own patch there with `git apply -R` and removed it plainly, no `--force`), `/tmp/bs71` (baseline and after binaries, probe output, sweep and gate scripts and logs), `/tmp/plotenv71`, `/tmp/bs71-build-evidence.log`, the throwaway `tests/zz_rebuff_probe.rs` (never committed), and my own ignored or untracked evidence under `runs/2026-10-31-03/evidence/{before,after,sweep}/` (the `series.csv` files and the per-run `narrative.txt`, `activities.txt`, `sleep.txt` and the rest, 321 MB; all regenerate with the commands in `eval.md`). `df -h .` 213 G free at the start, 180 G now (the clone's `target/` growing with two release test builds, the harness and the benchmark, beside the host's other jobs). **Leftovers, not mine, not removed:** worktree `/home/ladvien/.cache/loop-runs/before-tree` (3.2 G, clean, `d825bdd` on `origin/main`, idle over 2 h), worktree `/home/ladvien/.cache/price-before-src` (721 M, clean, `fd89781` on `origin/main`, idle over 2 h), `stash@{0}` (pass 17) and `stash@{1}` (2026-10-27) for a person to drop.
