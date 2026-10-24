# Review

## Pass 1 — Behavior: is the repair what it claims?

**GREAT** for the repair, **FAIL** for shipping it.

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | -0.054 |
| clamp, gate unchanged | 0 | 10 | -0.078 |
| clamp + gate 0.25 | 0 | 10 | **-0.045** |
| clamp + gate 0.25 + bar 0.30 + trust 0.5 | **16** | **10** | **-0.045** |

The mood cost is exactly explained: `compliment` scores **0.057** at the clamp ceiling against **0.673**
unclamped, because its gate sits at `Logistic(8.0, 0.7)` and the ceiling is 0.35. Moving the gate to
`Logistic(16.0, 0.25)` recovers mood **past** baseline.

## Pass 2 — Zoom: read the full suite, not the scenario you aimed at

**FAIL**, and this is the pass that matters. `village-evening` looked good on every axis — 16 friends (down from
22), 10 rivals (down from 33), mood -0.045 (better than -0.054). The full suite then failed
`opinion_convergence`:

| | before | with the clamp |
|---|---|---|
| opinion spread over 12 000 ticks | 0.5106 → **0.24** | 0.5106 → **0.3813** |

The scenario asserts a fall of at least a third; the clamp halves it. **Deffuant influence is weighted by tie
strength**, so bounding affinity bounds opinion convergence.

## Pass 3 — Fresh eyes: what does this tell the next attempt?

**GREAT**, and it is a different kind of finding from the previous six.

Six attempts changed *rates*, *gates* or the *accumulation shape*, and all failed to bound the village. This one
changed the **range** — and it worked, while revealing that the range is **load-bearing in a second model**.

> `affinity` sets how likeable people are, how fast they become friends, **and** how fast opinions spread. A
> scenario's tick count is silently a statement about that rate.

That is a coupling across three models. Any clamp must be paired with a decision about the opinion scenario, and
that decision is now recorded with the numbers rather than discovered later.

## Verdict

Pass 1 **GREAT**, Pass 2 **FAIL**, Pass 3 **GREAT**. **Reverted** — 53 binaries clean, village at 22/33.

The iteration's value: the clamp's actual cost is identified and quantified, the mood repair is proven, and the
next attempt knows the affinity range is shared before it touches it.
