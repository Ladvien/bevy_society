# Review

## Pass 1 — Behavior: does the mechanism work?

**GREAT** for the mechanism, and it is the one thing that did work.

| | before | after |
|---|---|---|
| two co-located people, 20 000 ticks | **30 exchanges** | **89** |
| cooldown | — | first gap **1 920 ticks at 3 h**, held |

Verified tick by tick: `last_spoke_day` holds at 0.6252 across ticks 63–1920 and advances at 1921. The gate is
exactly right, and the request goes through the same `Interact` stream a host writes.

## Pass 2 — Zoom: read the village, not the pair

**FAIL**, and that is the verdict that matters.

| cooldown | friends | rivals |
|---|---|---|
| none (shipped) | **22** | **33** |
| 3 h | **267** | **0** |
| 12 h | 182 | 3 |
| 48 h | 72 | 12 |

267 friend pairs in a village of 24 is **97% of all possible pairs**. Rivals 0 — a village with no
disagreements at all. The contact pass adds **13 742** requests against a baseline of roughly **7 500**
interactions: it nearly triples all social contact, because in a village everyone is always in contact.

The sweep is monotonic and there is no point on it that both bridges the cluster and preserves a village.

## Pass 3 — Fresh eyes: what the numbers mean

**GREAT for the reading, FAIL for the design.** Two people who share a tavern are aware of each other; they do
not become friends by standing in the same room. The crate has one verb — `Interact` — that does both, so
turning on "meeting" necessarily turns on "befriending", and a village cannot survive that.

The same overshoot appeared two iterations ago when a `help` activity was added (rivals 44 → 0) and was
reverted for the same reason. That is now **two independent mechanisms** producing a clique from a
plausible-looking local rule, which says the village's tie formation is far more sensitive to contact volume
than its rates suggest.

## Verdict

**Reverted.** Pass 1 GREAT, Pass 2 FAIL, Pass 3 FAIL — and the iteration's value is the measurement, not a
change. Shipping any tested cooldown would have traded a scenario that fails honestly for a model that fails
silently, which is the opposite of what this crate is for.

Recorded in `needs-decision/how-can-two-groups-ever-meet.md` with three options.
