# Review

## Pass 1 — Behavior: does the gate admit the crowd it claims to?

**GREAT.** Eight bodies packed inside a 1.5 m disc, one stationary walker at 2.2 m: refused with
`crowded_arrival_growth = 0.0`, admitted at `0.7`. Disabling the gate fails that test and leaves the empty-disc
guard passing, so the assertion is measuring the feature.

## Pass 2 — Zoom: three seeds, and the cost shown at full size

**GREAT.** `evidence/three-seeds.png`: interruptions and `sleep` failures fall on **every** seed (12-18% and
21-24%), and the third panel shows the rivalries rising by 10-14 rather than hiding them in prose.

The important read is the comparison with two iterations ago. The identical change, before the user's
`bounded-ties` merge, took rivalries **38 → 61**. Under `affinity_ceiling` it takes **28 → 42**, against a baseline
that has already halved friendships. The clamp did what it was for: it bounds other mechanisms' side effects.

**My own blocker had expired and I nearly didn't notice.** If I had trusted my log — "the choice belongs to the
host" — this would have been a third warning-only iteration. Re-measuring before building is the reason it is not.

## Pass 3 — Fresh eyes: is the test honest, and can a host explain it?

**GREAT for the feature, and a hard lesson on the test.** Four end-to-end fixtures were built first and **all
produced identical numbers with the feature on and off**: a 12-person roomy place, 24 people at a 1.0 m disc, six
residents at the village's ratio, and one matching its geometry exactly. The village's crowding needs four homes
and a shared gathering place interacting; a single crowded disc does not reproduce it.

Rather than ship a fifth fixture that passed either way, the test moved **down to the gate**, where the behaviour
actually lives. One detail mattered: the walker must have **no `Destination`**, because otherwise
`walk_toward_destination` walks them in and the test passes with the gate disabled — which is exactly how the
earlier attempts failed silently.

A host reads one sentence: *a place packed fuller than its arrival disc can seat still counts the queue as arrived,
one body diameter per square-root of the crowd, and you can turn it off with `0.0`.*

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## Not shipped, deliberately

- Separating arrival radius from contact radius (the recorded top item). Still the cleaner structural answer, but
  it is a schema change and the clamp has made the numeric fix affordable — so it is no longer urgent, and the
  decision file says so.
- Any change to `interaction_radius` itself: it is shared with contact, and that coupling is measured.
