# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what it claims?

Claim: an activity whose consideration is a script is scored by that script.

**Before.** `tests/dbg_s.rs` printed `Some(0)` — idle — for an activity whose only consideration was
`Script("1.0")` through `Linear(1.0, 0.0)`. Certain 1.0, and not chosen.

**After.** `a_script_consideration_is_evaluated` asserts the person is not idle, and
`a_script_consideration_can_score_zero` asserts the mirror: a script returning 0.0 does not win. Both
pass.

**Critique.** Two cases are not enough to call a scoring path proven: they show the extremes
(always-win, always-lose) but not the middle, and they do not show the script receiving the *right*
`actor`. That last omission is the one that matters — a consideration evaluated against the wrong
person would pass both tests.

**Verdict: NOT YET.** Fix: one more test that binds `actor` to the decision-maker.

## Pass 1, second attempt

Added a third test: an activity whose script reads `ctx.need(actor, "rest") > 0.5` wins only for a
person whose need is high, and the same definition does not win for a person whose need is low. That
fails if `actor` is unbound or bound to the wrong id.

**Verdict: GREAT.** The scoring path is exercised end to end, both extremes and the actor binding.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique.** Scoring is multiplicative: one consideration returning 0 zeroes the whole option, which
is correct for "this activity is impossible now" but surprising for a soft preference. The
compensation term `s + (1-s)(1-1/n)s` keeps a 0 at 0 — compensation cannot rescue a hard zero.

That asymmetry is worth stating in the docs rather than changing: a definition author needs to know
that `0` means "never", and a small value like `0.01` means "almost never but allowed".

**Verdict: GREAT** for behaviour, with the doc note required.
Recorded here and in the code comment; `docs/guide/authoring.md` says a failed consideration scores 0
but does not say a *successful* 0 is also final. Next iteration's polish.

## Pass 3 — Fresh eyes: another seed and population

**Critique.** `cargo test` runs the whole suite at 16 binaries; `tests/determinism.rs` re-runs the F1
village under seeds 7 and 8 and both hold. The quickstart, run fresh, reports `59 social exchanges,
30 accepted`.

But the 480-frame capture, run as part of this iteration, reported:

```
{"frames": 480, "rumor_reach": 0.040, "friends": 0, "rivals": 0, "barks": 0}
```

Zero friendships and one villager in twenty-five knowing the rumor is **not believable**, and it is the
one number a visitor sees first. This does not invalidate the scoring fix — the capture's village and
its timing are a separate concern — but it is the largest remaining gap in what the crate *shows*.

**Verdict: NOT YET** for the crate as a whole, **GREAT** for this iteration's change. The capture is
the next iteration's first candidate, and it outranks the remaining dead accessors.

## Summary

| Pass | Verdict | Why |
|---|---|---|
| Behaviour | GREAT (second attempt) | extremes and actor binding all proven by test |
| Zoom | GREAT, one doc note owed | a successful 0 is as final as a failure; documented in code, not yet in the guide |
| Fresh eyes | NOT YET for the crate | the hero capture produces no friendships and almost no rumor spread |

## Next iteration's candidates, in order

1. **The capture**: `friends 0`, `barks 0`, `rumor_reach 0.04` at 480 frames. Either the village's day
   is too short for ties to form, or `bark_chance` and the interaction weights need tuning — but the
   thresholds in `docs/DEMOS.md` say it must be ≥3 friendships, ≥1 rivalry, ≥6 barks, ≥0.5 reach.
2. `Field` scoring in `signal_value`, the mirror of the bug fixed here: a definition that uses a field
   input still scores 0 on every decision.
3. The four remaining dead accessors (`TriggerSpec::name`, `ActivitySpec::barks`,
   `ActivitySpec::target`, the snapshot's `Names` lookups, `TieView::target`).
4. `docs/guide/authoring.md`: say that a consideration scoring 0 is final.
5. Performance at 1,000 persons (`scale-1000` in `scenarios.yaml`, still `pending`).
