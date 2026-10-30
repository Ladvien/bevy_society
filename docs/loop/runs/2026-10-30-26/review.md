# 2026-10-30-26 — three reviews

Evidence cited is committed: `evidence/before/firings.md`, `evidence/after/firings.md`,
`evidence/need_below_probe.rs`, `eval.md`.

## 1. Behavior — GREAT

*Critique.* The documented contract is "fires when a person's need first falls below the line, re-arms
once back above `below + 0.05`". Before, the village at seed 7 had 12 people cross the line and exactly
1 firing (person 0's; the town's one flag was spent on it and nobody else got an event: issue case 2).
After, 12 firings, one per person, equal to an independent oracle that arms each person on their own
(`before/firings.md`, `after/firings.md`). The every-tick case (issue case 1) is covered by
`tests/need_below.rs::need_below_fires_once_per_person_per_crossing`: 10 firings in 10 ticks before, 1
after. What I looked for and did not find: a person firing before the line is crossed, a person who fired
and never could again (the re-arm test and the 5-day run show people firing twice), two triggers on one
line double-firing. The restore half is a state-mapping fix with no model in it; its tests assert the
documented promise (spent `once` state stays with its trigger; a person who fired stays disarmed).

## 2. Zoom — GREAT

*Critique.* Read per person, not just in total (`after/firings.md`, seed 99): fired equals the oracle for
all 24 people, including the five who crossed twice (their food minimum is 0.04 to 0.10 and the maximum
1.00, so they ate and came back above 0.55 between crossings). 2 of 1 536 ticks had a firing, so nothing
fires every tick; all firings for a day land on one tick, which is the society's own synchrony (everyone
starts at 1.0 and decays at the same rate), not the trigger. Spike at a tick boundary: none beyond that.
Two things I noticed and did not chase because they are not this change: **19 of 24 people reach food
0.00 within 6 days at seed 99** (a clip at the bottom of the range; either a supply or contention
question), and the synchrony itself. Both go to the log's remaining items.

## 3. Fresh eyes — GREAT

*Critique.* Regenerated from scratch with a different seed, population, line and length (seed 99, 24
people, 0.5, 6 days) against the same oracle: 29 firings = 29 crossings, per person equal. The
explanation a host developer needs fits in a sentence: *a `NeedBelow` trigger fires once for each person
each time their need falls below the line, and again only after it has recovered past `below + 0.05`;
nobody else's level matters.* That is also what `docs/guide/scripting.md` now says. The save side reads as
one rule too: *the save names its triggers by `id`, so editing the defs does not move anyone's spent
state.* One honest limit, written in `SocietySave::upgrade` and the CHANGELOG: a version-4 file has no
ids and no per-person arming, so it applies its indices as before and a person already below a line
fires once more after the restore.
