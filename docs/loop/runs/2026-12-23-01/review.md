# Review

## Pass 1 — Behavior: does the warning say something true?

**GREAT.** On the project's own village it names the two crowded homes:

```
'well' is home to 8 people, but an arrival disc of radius 1.5 seats about 4. They all go to bed at the same
hour, so the queue behind the disc times out and their sleeps are interrupted.
```

and the test asserts both halves — it fires on a twelve-resident home and stays quiet on a two-resident one. The
bite check fails with an empty capture, so the assertion is measuring the feature and not the harness.

## Pass 2 — Zoom: read the instrument, not the symptom

**GREAT, and this is the pass that produced the answer.** `evidence/crowding.png`, left panel: the histogram of
bodies standing inside the goal disc at the moment a sleep times out peaks at **6**, with a median of 6 and a tail
to 10 — and the homes declare `capacity: 6`. The failures are not lost walkers; they are the **queue behind a full
disc**.

A 1.5 m disc seats `(1.5 / 0.7)² ≈ 4.6` bodies. Six residents do not fit. That single comparison explains every
lever last iteration, including the one that did nothing: `stop_radius` cannot help when there is no room to walk
into.

Right panel: three more fixes, all trading interruptions for rivalries — 400/38 → 349/59, 290/51, 316/61. The
mechanism is **time**, not radius: an earlier arrival means the activity starts earlier and spends its full
duration standing still inside `contact_radius` instead of walking.

## Pass 3 — Fresh eyes: is a warning the right deliverable?

**GREAT, after I talked myself out of the fix three times.** The instinct is to ship a lever because an iteration
is supposed to produce code. But every lever here changes the village's social structure as a side effect, and the
choice between "more rivalries" and "interrupted sleeps" is a **design decision the host owns**. Shipping either
silently would be choosing for them.

What the crate can do is make the choice visible with numbers. That is the same reasoning as the reachability
warning two iterations ago, and it is the honest shape of the deliverable.

The arithmetic is labelled as a **lower bound**: ORCA queues at the centre rather than packing around it, so hex
packing understates the crowding. The warning says "about" and does not claim to simulate. Calling 4.6 a simulation
result would have been the easy, wrong thing.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## Not shipped, deliberately

- All three fix levers — measured, and each costs the tie network.
- A `Tuning` field for arrival-vs-contact separation. It is still the right structural answer and is recorded in
  `needs-decision/people-cannot-reach-their-beds.md`, but it is a schema change and it is not needed to report the
  crowding.
