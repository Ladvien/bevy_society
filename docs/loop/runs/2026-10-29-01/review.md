# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the change do what it claims?

Claim: the crate now tells a host when a definitions/settings pair makes an activity degenerate,
instead of accepting it silently.

**Evidence.** Running the cluster scenario and reading its log:

```
WARN bevy_society::behavior: bevy_society: 1 activities are shorter than two society ticks at
seconds_per_day 4 (one tick is 0.094 game-hours), so their people will re-decide every tick:
snub (0.1h = 1.1 ticks)
```

The count, the tick length, the activity's own duration and its tick count are all correct and
independently checkable. Two tests pin the invariant from both sides: at a 4-second day the starter
pack *must* have short activities, at the default 240-second day none may be shorter than a tick.

**Critique.** The tests assert the *invariant*, not the *warning*: no log capture is made, so
deleting the `warn!` call would leave both tests green. The behavioural evidence for the warning
itself is the manual run above, which is not a test.

That is a real weakness and it is recorded rather than papered over — the crate has no log-capture
harness, and adding one to assert a single line would be more machinery than the line is worth.

**Verdict: GREAT** for the accessor and the invariant (both tested), with the assertion gap named.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique.** The threshold is two ticks. Is it right at the boundaries?

- One tick: the failure mode — activity ends on the tick it starts, person re-decides immediately.
  Warned, correctly.
- Two ticks: warned. A person completing a two-tick activity alternates decisions every other tick,
  which is already a poor society.
- Four ticks (`snub` at the default 240-second day): not warned. That is the value the starter pack
  was authored for, and it is not degenerate.
- The reserved `idle` fallback is one hour and would trip the check at a short day; it is skipped by
  index, because it is *meant* to be the shortest thing in the pack.

No off-by-one: `< 2.0` excludes exactly 2.0, and the tests bracket 4.0 s and 240 s, which are 1.1
and 4.2 ticks for `snub`.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: another scenario and population

**Critique.** Checked across all three scenarios at their own settings. `village-evening` at
`seconds_per_day = 4.0` warns about `snub`; the cluster at 4.0 warns identically; `need-cycle` at
1.0 warns about more activities (a tick is 0.375 game-hours there, so `chat` at 0.3 h is **0.8
ticks** — the worst case, and it is caught).

Critically, **the default 240-second-day app warns about nothing**, which is the property that
matters for a host who takes the crate's defaults.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — warning verified by running it; the missing log-capture assertion is named |
| Zoom | GREAT — threshold bracketed from both sides, `idle` correctly exempt |
| Fresh eyes | GREAT — fires at 1.0 s, 4.0 s; silent at the default |

## The negative result, recorded

The log's #1 was attempted first and **could not be fixed by tuning**:

| attempt | chat person-ticks | outcome |
|---|---|---|
| baseline | 128 | — |
| gossip `satisfies` 1.0 → 0.3/h | 106 | worse |
| gossip interaction 0.15 → 0.05 | 153 | +20%, still starved |

Both reverted. `chat` and `gossip` serve one need with no differentiation, and a per-instance supply
small enough to balance a 1.0/day decay across 17 gossip instances makes gossiping worthless. That is
a schema question, and it should be written up in `needs-decision/` next iteration rather than tuned.

## Next iteration's candidates

1. **`chat` vs `gossip` sharing one need** — write the `needs-decision/` note: a second need, or a
   gossip variant gated on having something to tell. This is now the oldest unresolved finding.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open, implicated in
   three iterations, and now provably the same class as the duration warning just added.
3. `Signal::Field` / `Gradient` — Moussaïd 2011 needs re-converting.
4. A log-capture harness, so warnings can be asserted rather than read by hand.
5. `scale-1000` perf.
