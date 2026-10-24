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
