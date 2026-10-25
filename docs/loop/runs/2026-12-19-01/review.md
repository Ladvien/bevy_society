# Review

## Pass 1 — Behavior: does the constant do what its comment claims?

**GREAT.** `steering.rs` says "ORCA has no reason to prefer left over right at an exactly head-on meeting, so it
stalls; `Crowd` nudges each member to a side". At `0.05` the nudge did not do that job: 40 interrupted activities
against **4** when steering is switched off entirely. At `0.30` it does, at 19.

Three independent confirmations that steering is the cause, not a coincidence:
1. steering off -> 4, steering on -> 40, same village and seed;
2. crowd size scales it: 6 -> 6, 12 -> 19, 24 -> 40;
3. at every timeout the walker was printed **2.0-4.0 m** from its target with `Destination` still set.

## Pass 2 — Zoom: read the curve, not the headline

**GREAT.** The sweep is an **interior optimum**, not a monotone slide toward a rail:

| nudge | 0.05 | 0.15 | 0.20 | 0.25 | **0.30** | 0.60 |
|---|---|---|---|---|---|---|
| interrupted | 40 | 32 | 29 | 24 | **19** | 27 |

That shape is the argument: too little nudge and the stall survives, too much and the side-step pushes walkers
off their line. A constant with no interior optimum would have been tuning; this one has a right answer.

Evidence: `evidence/deadlock.png` (left panel: the curve; right: the three-way comparison).

## Pass 3 — Fresh eyes: is the residual a bug or geometry?

**GREAT**, and this pass changed what I would have claimed. I tried three tests to pin the behaviour — a ring of
twelve, a ring of twenty-four, and a synthetic village sharing one home. **All three passed at the old nudge as
well**, so none of them was measuring the change, and the synthetic village's interrupted share was *identical*
(7% both ways).

Rather than ship tests that report protection they do not give, they were deleted. What the failed test-building
taught me is the honest conclusion: the residual 19 is **geometric, not kinetic**. Twenty-four bodies of 0.35 m
radius cannot all fit inside a 1.5 m arrival circle. The floor of 4 is unreachable without changing what the
village *means* — `ActivityTarget::Home` ignores `Place::capacity`, so 24 people are sent to a place declaring
room for 6. That was measured too (capacity 24 -> 27 interruptions, radius 0.2 -> 19) but neither is a change to
make silently.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**, and the least comfortable of the three: the change ships
**without a test**, because every test I could write passed either way. The constant carries its measurement
table and the reproduction command instead.

## Not shipped, deliberately

- `Home` honouring `Place::capacity` — changes what the village means; a decision.
- body radius 0.35 -> 0.20 — changes a physical property for a numerical reason.
- the travel timeout — `travel-time-in-two-clocks.md`, still open.
