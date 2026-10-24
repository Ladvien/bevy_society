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
