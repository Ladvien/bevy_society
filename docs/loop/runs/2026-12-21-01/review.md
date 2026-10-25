# Review

## Pass 1 — Behavior: does the warning say something true and actionable?

**GREAT.** On the project's own village it names the pair and the number:

```
'market' -> 'workshop' at 16.4 m ... Raise travel_timeout_hours to at least 4.4
```

and both halves are asserted: it fires at 30 m with a 4 h timeout, and stays quiet at 5 m. Disabling the system
fails the test and prints the *old* warning as the only output — the exact gap being filled.

Evidence: `evidence/reachability.png` (left panel).

## Pass 2 — Zoom: the plot shows the fix is partial, and I did not hide it

**GREAT**, and this is the pass that could have gone wrong. The right panel is the one that matters:

| timeout | interrupted | `sleep` interrupted |
|---|---|---|
| 4 h | 400 | **150** |
| 5 h | 312 | **142** |

Raising the timeout to cover the whole village removes **8 of 150** sleep failures. So reachability is **5% of
the sleep problem**, not the cause of it. I had a clean story ("the timeout is too short") and the measurement cut
it down to a fifth of its size.

The other 142 time out with an 18.6 m budget against a 16.4 m worst case, so they are not walking a straight
line — blocked by a crowd, or re-targeted mid-walk. That is a different defect, it is next iteration's item, and
it is **not** claimed here.

## Pass 3 — Fresh eyes: can a host act on it?

**GREAT.** One sentence: *if your timeout cannot cover the distance between your two farthest places, the crate
tells you which two and what number to set.*

That is the whole value: before, a host with a 16.4 m village and a 4 h timeout got silence, because 14.9 m is
above the 10 m generic floor. `warn_short_travel_reach`'s own doc comment admitted the limit — *"above it the
crate cannot tell, because it does not know the layout"* — and a `Startup` system does know it. Using information
the crate already had is the cheapest kind of improvement available.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT** — the partial result is stated at full size rather than rounded up. Pass 3
**GREAT**.

Test coverage is real this time: the test fails when the feature is removed, ending a two-iteration run of
shipping changes I could not test.

## Not shipped, deliberately

- The residual 142 sleep timeouts — measured and recorded as the next item.
- The harness-builder refactor — still worth doing, but it helps future iterations rather than this one.
- The guide's scale curve — unverified for a fifth iteration.
