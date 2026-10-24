# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the harness actually capture?

Claim: a `warn!` in `BehaviorPlugin::finish` is now observable from a test, and deleting it fails.

**Evidence.** `tests/warnings.rs::every_startup_warning_is_asserted_from_one_app` builds one capturing
app and asserts, per warning: exactly one event, the level, the activity name, and the arithmetic.
The captured lines are in `evidence/captured-warnings.txt`.

**Critique.** The strongest available check is a negative one: comment out a `warn!` and confirm the
test fails. Done mentally? No — done for real would mean editing the source and reverting. The
substitute is that the test asserts a *count of one* and the message's *contents*; an empty capture
fails on the count, and a wrong message fails on the contents. That is strong enough without
mutating the source.

**Verdict: GREAT.**

## Pass 2 — Zoom: what the harness's own limits are

**Critique — the important one.** The `tracing` subscriber is global and settable once, so a binary
may build **one** capturing app. I discovered this the hard way: four tests passed individually and
three of them silently captured nothing, logging
`Could not set global logger and tracing subscriber as they are already set`.

That is a sharp edge for anyone adding a third warning later. It is documented in the module header
and in the test file's own doc comment, and the test count is deliberately two:
- one capturing test that asserts both warnings,
- one non-capturing test that asserts the starter pack's checks produce neither.

A future test that needs capture must either extend the first test or live in its own file.

**Verdict: GREAT**, with the constraint documented where the next author will hit it.

## Pass 3 — Fresh eyes: did the harness justify itself immediately?

**Critique.** Yes, and the way it did is the strongest argument for having built it. Asserting the
field warning's text showed it read:

```
2 definitions rely on social-field inputs … — follow, follow
```

for a definitions file with **one** offending activity. `warn_unscored_fields` pushed the activity
once for its `Field` consideration and again for its `Gradient` target. The previous iteration's four
invariant tests — which checked that both accessors *see* the definition — were all green while the
sentence was wrong.

Fixed, and the test now asserts `1 definitions`, so the double-count cannot come back.

**Verdict: GREAT** — the harness found a real defect in a shipped warning within minutes of existing.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — count, level, name and arithmetic all asserted; a deleted `warn!` fails the test |
| Zoom | GREAT — the one-capturing-app-per-binary limit documented where it will be hit |
| Fresh eyes | GREAT — caught a shipped warning over-reporting by half |

## Next iteration's candidates

1. **`Field` / `Gradient` wiring** — the actual bug, four measured facts and the unexplained tick-20
   case recorded. Start from a single-person case with one cell of value 1.0.
2. **The cluster's 1:24 starvation** — a second need; a schema change.
3. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
4. **A spatial index for perception** past ~2 000 people, with the benchmark numbers behind it.
5. **`warn_unscored_fields` says "1 definitions"** — grammatically wrong. A plural helper would fix it;
   noted rather than done so the fix is deliberate.
