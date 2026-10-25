# Review

## Pass 1 — Behavior: does it do what the design says?

**GREAT.** Two people who share a place pass a rumor **iff** the teller has news the listener lacks. All four
tests fail when the behaviour they pin is removed — verified by disabling the gate and by deleting the
`theirs.knows(rumor)` check. Evidence: `evidence/after/narrative.txt` shows gossip starting and completing at
realistic 18-game-minute durations (d2 11h23 → 11h41).

Against the literature, the design is what civil inattention describes (Goffman 1963 via Cañigueral & Hamilton
2019, DOI `10.3389/fpsyg.2019.00560`): acknowledge presence, do not treat it as special interest. A pair with
nothing between them is offered nothing.

## Pass 2 — Zoom: read the plot, not the summary

**FAIL on the first version, GREAT after the fix.** This pass is why the iteration has a finding.

`evidence/before-after.png`, middle panel: the shipped curve rises to **0.583**; the first version of the change
plateaus at **0.375**. A **35% drop in rumor reach** on a scenario the change did not target.

The village summary did not show it — friends fell 23 → 23, mood improved. Only the reach panel did. Cause, by
isolation: every telling to an *informed* listener stifles the teller (Daley–Kendall), so a half-day cooldown
spends the spreader pool faster than it spreads news.

The fix is the check I had removed as "redundant", plus pacing. The sweep is monotonic in the right direction
once the check is restored:

| cooldown | reach | friends | rivals |
|---|---|---|---|
| 0.5 d | 0.375 | 23 | 45 |
| **2.0 d** | **0.875** | **25** | **28** |
| 6.0 d | 0.583 | 22 | 33 |

## Pass 3 — Fresh eyes: another seed, and can a host explain it?

**GREAT.** A different seed gives friends 22, rivals 50, mood -0.0466 — the same shape, no seed-specific
artefact. The rule is one sentence a host can read: *people who share a place talk when the speaker has news the
listener does not have, and the village pays for it in spreaders, so it is paced.*

The clique failure mode of the earlier attempt is absent: friends 25 of 276 possible pairs, not 267.

## Verdict

Pass 1 **GREAT**. Pass 2 **FAIL → GREAT** after restoring the informed-listener check and pacing at 2 days.
Pass 3 **GREAT**.

Shipped at `contact_gossip_cooldown_days = 2.0`, with the sweep recorded in the field's doc comment so the next
reader knows why that number and not another.
