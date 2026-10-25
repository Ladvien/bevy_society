# Review

## Pass 1 — Behavior: did any lever do what it claimed?

**FAIL on all seven, and that is the iteration's result.** The table is in `eval.md`; the summary is that no lever
reduced the sleep jam, and the two that touched `interaction_radius` paid for a 14-failure improvement with 52 new
rivalries.

`stop_radius` is the one to read twice: changing how far walkers push toward their goal produced a **byte-identical
simulation**. That is not a null result to shrug at — it rules out the entire "tune the walker" family of fixes and
says the constraint is elsewhere.

## Pass 2 — Zoom: read the negative results, not just the positive ones

**GREAT as measurement, FAIL as a fix.** `evidence/levers.png` left panel: every bar at or above the baseline
except the two radius levers, which buy 22-36 fewer interruptions with the right panel showing 90 and 82 rivals
against 38.

Two things the close read caught that a summary would have hidden:

1. **The first scatter test was invalid.** It drew from `SocietyRng`, so 456 versus 400 compared two different
   simulations. Re-run deterministically it gave 484 — still worse, but the number I nearly recorded was not a
   measurement of anything. The purple bar marks it.
2. **`stop_radius` is byte-identical, not merely similar.** 400 / 150 / 31 / 38 in both. Identical output from a
   changed constant means the constant is not in the causal path.

## Pass 3 — Fresh eyes: what should the next attempt do?

**GREAT, and it changed my next step.** I had been treating this as a geometry problem — crowd cannot share a
point, so spread the point. Scattering made it worse in both variants, which kills that story.

What survives: 150 of 150 have a `SocialMover`, 84 of 150 still carry velocity > 0.01 m/s, and yet they never close
from 2.06 m to 1.5 m over a four-hour budget. With `stop_radius` irrelevant, the question is no longer "where are
they trying to go" but **"what is the solver being handed"** — `pref_vel`, neighbour count, member radius. That is
the same instrument-then-measure move that cracked the person-target bug last iteration, and it is where the next
attempt should start.

## Verdict

Pass 1 **FAIL**. Pass 2 **GREAT** (measurement), FAIL (fix). Pass 3 **GREAT**.

**Nothing shipped.** Seven levers is past the loop's five-round limit, so the blocker is logged and the tree
reverted rather than a seventh constant nudged into the code because the iteration was supposed to produce
something.

## What a host gets from this iteration

Not a fix — a ruled-out list. The next person to look at a village where people cannot reach their beds will not
spend an iteration on `stop_radius`, on scattering goals, or on `interaction_radius`, and will know that the cost
of touching proximity is measured in rivalries.
