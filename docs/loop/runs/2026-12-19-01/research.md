# Research: a crowd walking to one place must not deadlock

## Why this

Item 2 is "behavior that reads as wrong when you watch it". Reading `village-evening`'s narrative as a stranger
showed it plainly: **19 of 27 sleeps were `(interrupted)`**, and one villager's `sleep` started at 01:25 and
"finished (completed)" at 10:40 only because `play` began. People were not sleeping.

## The measurement

Counting `ActivityEnded` over 17 game-days at the scenario's own settings:

| mover | interrupted | completed |
|---|---|---|
| the core walker, **no steering at all** | **4** | 305 |
| ORCA steering | **40** | 242 |

Ten times more failures with the steering feature on than with it off. The crate ships steering, so this is the
configuration a host gets.

Crowd size scales it: **6 people → 6** interrupted, **12 → 19**, **24 → 40**.

## The cause

Every sleep targets `Home`, and the village gives all 24 people the same one. At evening the whole population
walks to a single coordinate. `steering.rs` already names the failure in its own words:

> ORCA has no reason to prefer left over right at an exactly head-on meeting, so it stalls.

`NUDGE` is the symmetry kick that breaks it, as a fraction of a mover's wish. At the shipped `0.05` it was too
weak. Walkers stalled **2.0–4.0 m** from their target and sat there until the 4-hour `travel_timeout_hours`
expired — measured directly by printing the remaining distance at every timeout.

## The fix, measured

| nudge | interrupted |
|---|---|
| 0.05 (shipped) | 40 |
| 0.15 | 32 |
| 0.20 | 29 |
| 0.25 | 24 |
| **0.30** | **19** |
| 0.60 | 27 |

Interior optimum. At **0.30** the village reads **26 friends / 33 rivals** against the shipped **22 / 33** — the
kick costs the society nothing and finds more friends, because people actually reach each other.

Reproducible: two runs give 19 interruptions and 26/33 both times.

## What is still wrong

`NUDGE = 0.30` halves the failures; it does not reach the core walker's 4. The residual is geometric: 24 bodies
of 0.35 m radius cannot all occupy a 1.5 m arrival circle, so the outer shell legitimately cannot get in. That is
a **capacity** problem, not a steering one, and the honest fix is the one `ActivityTarget::Home` lacks — unlike
`ActivityTarget::Place(tag)`, it never checks `Place::capacity`. Both `capacity: 24` and body radius 0.2 m were
each measured to bring interruptions down (27 and 19 respectively) but neither eliminates them, and neither is a
change to make silently: the first alters what the village means, the second changes a body's physical size.

The travel timeout remains the third factor, and it is `travel-time-in-two-clocks.md`, still a decision.

## Sources

None for the constant — `NUDGE` is a numerical device for breaking symmetric deadlocks, not a social parameter,
and the crate says so in the comment. The interpersonal-space framing that makes a crowded arrival *legitimate*
rather than a bug is Vagnoni et al., "Listening to a conversation with aggressive content expands the
interpersonal space" (`W2795247901`), which defines personal space as "the emotionally-tinged zone around the
human body that people experience as 'their space' and which others cannot intrude without causing discomfort".
