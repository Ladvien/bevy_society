# Eval

## Change

`warn_unreachable_places`: the crate now measures its own layout at `Startup` and says when `travel_timeout_hours`
cannot cover a walk between two places that exist.

## The warning, verbatim, on the project's own village

```
a travel_timeout_hours of 4 at seconds_per_day 64 buys 14.9 metres of walking, but the farthest pair of
places is 'market' -> 'workshop' at 16.4 m. Every walk between them times out before arrival, so the
activities that need one end interrupted and the needs they satisfy are never met. Raise
travel_timeout_hours to at least 4.4, or move the places closer.
```

Before this, `warn_short_travel_reach` stayed silent — 14.9 m is above its 10 m floor — and its own doc comment
explained why: *"above it the crate cannot tell, because it does not know the layout."* A system can know it.

## What it bought

| timeout | interrupted | `sleep` interrupted | warning |
|---|---|---|---|
| 4 h | 400 | 150 | fires |
| 5 h | **312** | 142 | silent |

**The honest accounting:** this explains **8 of 150** sleep failures. The other **142** survive a budget that
covers the whole village (18.6 m against a 16.4 m worst case), so they are not reachability — they are walkers
who never complete a straight line. That defect is recorded, not claimed.

## Why the change is still worth shipping

A host running this crate with a layout their own game chose now gets a **specific** diagnosis: which two places,
how far apart, and the exact number to set. Silence would have told them nothing, and the generic 10 m floor says
nothing about a 16.4 m village.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **60** binaries |
| clippy / docs / fmt | 0 errors |
| bite check | disabling the system fails the test |

## Coverage

Real this time. Unlike the last two iterations, the test **fails when the feature is removed** — the failure
prints the old warning as the only output, which is exactly the gap the new one fills.
