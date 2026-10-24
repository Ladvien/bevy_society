# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the crate still find the right neighbours?

Claim: perception's output is exactly what brute force produces, and there is now a test that says so.

**Evidence.** `tests/perception.rs` builds a reference double loop from `Perception::pos` and compares
against the real `near` and `contact` lists:

```
compared 32016 neighbour entries across 180 people
```

**Critique.** The test is `O(n²)` by construction, so 180 people is the practical ceiling; a future index
with a bug that only appears at 5 000 would not be caught. It is still the strongest check available
without a second reference implementation, and the layout is chosen to be awkward — people spaced at
0.98 × the perception radius, so many land on cell boundaries, plus clusters and lone figures.

**Verdict: GREAT.**

## Pass 2 — Zoom: the numbers

**Critique — this is the pass that decided the iteration.** The grid *worked*: it found the same
neighbours and gave a 1% improvement at 5 000. A weaker review would call that a win and ship it.

The table says otherwise:

| | brute force | grid |
|---|---|---|
| 100 | **0.573** | 0.707 |
| 5 000 | 300.8 | **297.96** |

**19% slower at 100 people** and 1% faster at 5 000. The crate's own README and guides describe a society
of tens to hundreds; a change that taxes the common case to shave noise off the rare one is a regression
wearing a percentage.

And the `BTreeMap` variant — the obvious second idea, ordered cells so the sort could be skipped — was
**slower still** at 1 000 (16.97 vs 15.67). That is what closed it: the sort was not the only cost.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: what did the iteration leave behind?

**Critique.** Nothing in `src/`. What it left is a reference implementation in `tests/perception.rs` and a
documented cost on `perceive`, plus a corrected claim: perception is **11.8%** of a tick at 5 000, not the
9.2% the log carried.

That is a smaller deliverable than a working index. But the alternative — shipping a 19%-slower small
village so a benchmark column improves by a percent — is exactly the failure this loop's review passes
exist to catch. The recorded negative result stops the next iteration rebuilding the same grid.

**Critique.** I did not try a third variant. The paper's own reasoning (amortise over several queries per
step) explains why this shape cannot win here, so a third attempt would be re-rolling dice rather than
testing a hypothesis. The loop's rule is five failed rounds then log the blocker; one measured round plus
a structural explanation is enough to record it.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — identical neighbours, pinned by a 32 016-entry comparison |
| Zoom | GREAT — 19% slower at 100 people; the 1% win at 5 000 is noise, and shipping it would be a regression |
| Fresh eyes | GREAT — a reference implementation, a documented cost, and a corrected claim |

## Top 5 remaining

1. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, and 87% of the book
   is untellable. A decision, not a fix.
2. **The two-clocks decision** — warned about, measured, no longer the cause of starvation, API open.
3. **`food` at 15% of ticks at zero** — the worst-served need now that `fun` is fixed.
4. **The village's rumor origin** — reaches 1/24 in `village-evening`.
5. **Perception is 11.8% at 5 000 and `O(n²)`** — a real limit, now measured. An index needs the sort
   amortised, which means either a coarser cell size that keeps index order or a query that runs more than
   once per tick.
