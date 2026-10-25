# Research: seven levers on the sleep jam, none of them work

## Why this

The previous iteration established that **142 of 150** sleep failures survive a travel budget that covers the whole
village, and left that as the top item. This iteration set out to fix it and **failed**, which is the finding.

## What was already established

| fact | measurement |
|---|---|
| all 150 sleep failures are `timed_out`, not gone/suspended/lost | instrumented every exit path |
| they stall just outside the arrival radius | **min 1.52 m, p25 1.83, median 2.06, p75 2.33** against `interaction_radius` 1.5 |
| they have a `Destination` | 150 of 150 |
| they have a `SocialMover` | 150 of 150, and **84 still have velocity > 0.01 m/s** — pressing, not idle |

So: a crowd of people pressing toward a target they cannot reach. Every lever below was chosen to break that
press, and each was measured at seed 7 over 17 game-days.

## The seven levers

| lever | interrupted | sleep | friends | rivals | comparable? |
|---|---|---|---|---|---|
| baseline (shipped) | 400 | 150 | 31 | 38 | — |
| scatter destinations, drawn from RNG | 456 | 173 | 25 | 42 | **no** — consumes the stream, so the whole run diverges |
| scatter destinations, deterministic (golden angle by index) | **484** | **174** | 25 | 36 | yes |
| steering `stop_radius` 0.75 → 1.5 m | **400** | **150** | 31 | 38 | yes — *byte-identical* |
| `interaction_radius` 1.5 → 2.2 | 378 | 160 | 24 | **90** | yes |
| `interaction_radius` 1.5 → 2.6 | 364 | 136 | 19 | **82** | yes |
| `Home` honours `Place::capacity` | **440** | **155** | 22 | **69** | yes |

## What each failure teaches

**Scattering the goal makes it worse, both ways.** The hypothesis was that a crowd cannot share one coordinate, so
ORCA holds the back of the queue outside the radius. Spreading the goals should let everyone in. Measured, it
raises interruptions to 484 and leaves sleep at 174. So the jam is **not** about sharing a goal point.

**`stop_radius` is a no-op, and that is the most informative result.** Changing steering's stop distance from half
the interaction radius to the whole of it produced a **byte-identical** simulation — 400 / 150 / 31 / 38. Steering
runs every fixed step and is compiled in (`steering` is a default feature). A change to how far walkers push toward
their goal that changes *nothing* means the walkers' wishes are already zero, or ORCA is refusing all motion
regardless of the wish. Either way, the lever is downstream of the problem.

**`interaction_radius` is shared with contact.** Enlarging it does reduce sleep slightly (150 → 136 at 2.6 m), but
it also enlarges `contact_radius`-style proximity, so rivals go **38 → 90**. That is the affinity feedback loop
that six earlier iterations on the friendship clamp already mapped: anything that changes contact changes the tie
network. Not a lever, a coupling.

**Honouring `Home` capacity makes it worse.** Refusing a full home sends people elsewhere instead of home, which
raised interruptions to 440 and rivals to 69. The inconsistency with `Place(tag)` is real but fixing it in
isolation damages the village.

## Conclusion: this is a blocker, not a bug to patch

Seven levers, each measured, none reduces the jam, and three of them damage another model. The loop's rule is
"after 5 failed rounds, log the blocker and revert" — this is round seven.

What the evidence points at, and what it would take:

- The stall is **mechanical**: pressing walkers at ~2 m with zero or near-zero velocity means ORCA's pairwise
  constraints are being satisfied by mutual blocking, not by anyone reaching a goal.
- `stop_radius` not mattering says the wish may already be `None`/zero for these movers, which would mean the
  problem is upstream — in what `Destination` these people actually hold at that moment, not in how they chase it.
- Separating that needs the same treatment as the person-target bug: instrument what the *solver* is handed
  (member count, neighbours, `pref_vel`) for the stuck walkers, rather than changing constants and re-running.

Nothing changed in the crate this iteration. The measurement is the deliverable.

## Sources

No paper. This is a mechanical question about a crowd solver, and the numbers are the crate's own.
