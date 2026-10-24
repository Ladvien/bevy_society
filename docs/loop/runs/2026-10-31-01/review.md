# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the number measure what it claims?

Claim: `ms_per_tick` is what one society tick costs at N persons.

**Evidence.** Two independent runs at 1 000 persons give 14.168 and 14.167 ms/tick. `ms_per_million_pairs`
converges to 28.1–28.4 at 1 000 and 5 000, which is the signature of a pair-proportional cost — the
brute-force perception the design claims.

**Critique.** Two things could flatter the number and both were checked:

1. **Idle people are cheap.** An unsettled crowd skips `decide_activities` and `resolve_interactions`.
   The harness settles 400 ticks before measuring, and the 5 000-person run has to settle *and* measure
   to see anything but idle.
2. **Per-update would be wrong.** A society tick is once per `tick_secs`; reporting per update would
   understate the cost by however many fixed steps a tick spans. The harness pins one tick per update
   and reports ticks, not updates.

What the number does *not* isolate: which system costs the most. Perception is assumed dominant
because the cost is quadratic in persons, but that is inference from the shape, not a profile.

**Verdict: GREAT** for the cost figure, with the per-system attribution named as inference.

## Pass 2 — Zoom: what the numbers do at the edges

**Critique.** Reading the table:

- 100 persons: 0.549 ms/tick, 110.8 ms/million-pairs. The per-pair figure is 4× the large runs',
  which at 4 950 pairs means fixed overhead dominates. Correct and expected; worth noting so nobody
  reads it as superlinearity.
- 1 000: 14.086 ms, 28.2 ms/million-pairs.
- 5 000: 351.640 ms, 28.1. The constant is flat, so the growth from 1 000 to 5 000 is exactly the
  25× in pairs (499 500 → 12 497 500) — 25.0× in cost. Quadratic confirmed numerically, not just
  asserted.
- The amortised frame column assumes 60 fps and `tick_secs = 0.25` (one tick per 15 frames). Both are
  stated, because the amortisation is only true for those settings.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: does a host get the answer they need?

**Critique.** The guide previously said "fine" up to about a thousand. The measurement says: 100 is
free, 1 000 is 4.9% of a 60 fps frame, 5 000 is **140%** — unplayable. So the design's stated limit is
*correct at 1 000 and wrong at 5 000*, and a host reading "fine" had no way to know where the cliff
was.

The table now states it, and states the assumption behind the amortisation instead of leaving it
implicit.

**Verdict: GREAT** — a claim the crate made about itself is now checkable, and the point where the
design stops working is named.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 0.01% reproducible; idle and per-update traps both closed; per-system attribution flagged as inference |
| Zoom | GREAT — quadratic confirmed as exactly 25.0× for 25× pairs; the 100-person outlier explained |
| Fresh eyes | GREAT — a host now learns where the cliff is, not just that the near side is "fine" |

## Next iteration's candidates

1. **The cluster's 1:24 starvation** — option 1: a second need. A schema change; the only remaining
   path, and now the oldest open finding.
2. **The clock decision** (`travel_timeout_hours` real-time) — escalated, still open.
3. **`Signal::Field` / `Gradient`** — Moussaïd 2011 needs re-converting.
4. **A spatial index** for perception, if a host needs past ~2 000 people — now that the measurement
   shows where the cliff is, this is a `needs-decision/` note rather than a guess.
5. A log-capture harness so warnings can be asserted rather than read by hand.
