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
