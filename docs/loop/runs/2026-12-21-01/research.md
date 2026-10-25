# Research: the crate can now tell whether its own layout is walkable

## Why this

My log's top item was "reproduction coverage is slipping — the harness's village builder should move where
`tests/` can reach it". Before doing that refactor I re-read the failure breakdown, which had changed under it:

| activity | interrupted | share |
|---|---|---|
| **sleep** | **150** | **37.5%** |
| work | 138 | 34.5% |
| eat | 70 | 17.5% |
| relax | 23 | 5.8% |

Last iteration's person-target fix had cleared the noise, and `sleep` — the `Home` activity — was now genuinely
first. My previous log entry said capacity was "no longer first"; it is first again.

## What the sleep failures actually are

Instrumenting every exit path for `sleep`:

```
150 WHY sleep pre-arrival timed_out=true gone=false
```

All **150**, no exceptions: the walk to bed ran out of `travel_timeout_hours`. Not capacity, not dormancy, not a
lost target.

## The arithmetic

`warn_short_travel_reach` computes reachable distance as `speed · seconds_per_tick · ticks_available`, which
reduces to `speed · seconds_per_day · timeout_hours / 24`:

| timeout | reachable at 64 s/day |
|---|---|
| 4 h | **14.9 m** |
| 5 h | 18.6 m |
| 6 h | 22.4 m |

The village's places span `(-6.5, -5)` to `(6.5, 5)`: the farthest pair is **16.4 m**. A 4-hour timeout therefore
forbids the `market → workshop` walk by **1.5 m**.

## Why the existing warning did not catch it

`warn_short_travel_reach` fires only below `MIN_WALK_METRES = 10.0`, and says why in its own doc comment:

> above it the crate cannot tell, because it does not know the layout

At `finish()` that is true — there is no world yet. **But a system can see the layout.** Every `Place` is an
entity with a `Transform`, so at `Startup` the crate knows every pairwise distance exactly. That is the gap this
iteration closes: not a new model, but the crate using information it already had.

## What the warning says now

```
a travel_timeout_hours of 4 at seconds_per_day 64 buys 14.9 metres of walking, but the farthest pair of
places is 'market' -> 'workshop' at 16.4 m. Every walk between them times out before arrival, so the
activities that need one end interrupted and the needs they satisfy are never met. Raise travel_timeout_hours
to at least 4.4, or move the places closer.
```

It names the pair and the number. A host can act on it without reading this crate's source.

## What this does NOT fix, measured rather than assumed

Raising the harness to 5 h silences the warning and moves interruptions **400 → 312**, but `sleep` only
**150 → 142**. **Eight of the 150 are the reachability problem; 142 are something else.** They time out even with
an 18.6 m budget, so they are not walking a straight line to their destination — blocked by a crowd, or
re-targeted mid-walk.

That residual is a different defect and is **not** claimed here. It is recorded as the next item, with the
measurement that separates it from this one.

## Sources

No paper. This is arithmetic over the crate's own settings and the world's own transforms.
