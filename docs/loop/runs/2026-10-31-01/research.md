# Research: what a thousand people actually cost

The log's #5 was `scale-1000`, listed as `pending` in `scenarios.yaml` since the loop's first
iteration. The brief ranks performance as candidate #4, and `docs/guide/concepts.md` made a claim
nothing had ever checked:

> Perception is brute force … up to roughly a thousand people, which at a 0.25 s tick is a few
> hundred thousand distance checks per tick — **fine**, and simple enough to reason about.

"Fine" is not a number. This iteration measured it.

## The harness

`examples/benchmark` spawns N autonomous persons in a golden-angle spiral with tagged places scaled
to the population, settles them so they are not all idle (an idle person skips the decision and
exchange systems, which would flatter the number), then times N society ticks and reports
`ms_per_tick`.

Two decisions that make the number honest:

- **Per society tick, not per update.** A society tick happens once per `tick_secs`, so a host's
  frame cost is `ms_per_tick / (tick_secs · fps)`. Reporting per update would understate or overstate
  depending on settings.
- **Settled first.** An unsettled crowd has everyone idle; `end_activities` and `decide_activities`
  then do almost nothing.

## The measurement

Release build, M1 Max, starter pack, `tick_secs = 0.25`, one society tick per fixed step:

| persons | ms per tick | pairs per tick | amortised per 60 fps frame |
|---|---|---|---|
| 100 | 0.55 | 4 950 | 0.03 ms (0.2%) |
| 1 000 | **14.1** | 499 500 | 0.83 ms (4.9%) |
| 5 000 | **351.6** | 12 497 500 | 23.4 ms (**140%**) |

**Reproducible.** Two independent 400-tick runs at 1 000 persons: 14.168 and 14.167 ms/tick — a
0.01% spread. This is not a noisy number.

**Quadratic, as claimed.** `ms_per_million_pairs` converges to 28.1–28.4 at 1 000 and 5 000, which
is what a cost proportional to pair count looks like. The 100-person run reads 110 because at
4 950 pairs the fixed per-system overhead dominates.

## What it means for a host

- **100 people: free.** 0.2% of a frame.
- **1 000 people: usable but not free.** 5% of a 60 fps budget, and that amortisation assumes one
  society tick every fifteen frames. Raise `tick_secs`, or lower the frame rate, and it grows
  proportionally.
- **5 000 people: unplayable at 60 fps.** 140% of a frame. This is where a spatial index stops
  being an optimisation and becomes the only way to run it.

So the guide's "fine up to about a thousand" is **correct at 1 000 and wrong at 5 000**, and the
guide now carries the table instead of the adjective.

## What was not done, and why

No optimisation. The cost is perception, which is brute force *by design* — the SPEC chose it and
the guide says so. Making it fast means a spatial index, which changes `Perception`'s shape and is a
structural decision for `needs-decision/`, not something to slip into a measurement iteration.

## The corpus

Not searched: no model was being changed, and this is a measurement of the crate's own code. A paper
search for "quadratic perception spatial index" would return broad-phase collision literature that
does not change what the number *is*.

## Verdict

The `scale-1000` scenario moves from `pending` to `measured` with real numbers, the guide's claim is
replaced by a table, and the point at which the design stops working (between 1 000 and 5 000) is now
documented rather than assumed.
