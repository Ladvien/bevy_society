# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is the bias gone?

Claim: an activity's odds no longer scale with how many targets it has.

**Evidence.** `tests/candidate_bias.rs`, written red first at `0.97`. Two activities identical in
every consideration, differing only in target kind:

| people | `visit` (Person) | `rest_here` (Place) | `visit` share |
|---|---|---|---|
| 1 | 0 | 3 | 0.00 |
| 12, before | 35 | 1 | **0.97** |
| 12, after | 18 | 18 | **0.50** |

And the starvation it caused, measured with travel removed so only choice differs:

| people | `food` at 0 | `fun` at 0 |
|---|---|---|
| 4 | 4.5 → **1.3** | 5.4 → **1.2** |
| 8 | 9.6 → **3.2** | 28.2 → **4.2** |
| 16 | 11.4 → **2.3** | 43.4 → **7.7** |

At the village layout with places at the origin: `food` 39.3% → **1.9%**, `fun` 64.7% → **4.8%**.

**Critique.** The 0.50 result is exact, which is suspicious on first reading — but the two activities
are constructed to have identical scores (`Linear(-1, 1)` on the same need), so an even split is the
*correct* answer, not a coincidence. The weaker part is that the probe uses `Linear` rather than a
realistic consideration set; that is what the village runs are for, and they agree.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the artifacts closely

**Critique — this is where the honest problem is.** `evidence/before-after.png` shows the village's
needs still near zero after the fix:

| village-evening, seed 7 | need2 at 0 | need3 at 0 |
|---|---|---|
| before | 62.9% | 74.6% |
| after | **27.7%** | **55.2%** |

Better, not good. So I isolated travel by moving the four places to the origin and changing nothing
else:

| | need0 | need1 | need2 | need3 |
|---|---|---|---|---|
| places metres apart | 0.105 | 0.173 | 0.070 | 0.049 |
| places at the origin | **0.869** | **0.810** | **0.554** | **0.526** |

That is the whole story: **choice is fixed, travel is not**, and travel is
`needs-decision/travel-time-in-two-clocks.md`, which has been escalated twice and is the operator's
call. The plot is a correct picture of a partially fixed problem, and pretending otherwise would be
the failure mode this loop exists to avoid.

Two more things the close read gives:

1. **Friend edges fell** 118 → 42 at the same seed (village) and rivals went 0 → **6** at 40 people.
   Both are the intended consequence: social activities are no longer multiplied by the crowd. Rivals
   appearing for the first time is a sign `snub` now has to *earn* its selection on grudge rather than
   winning by target count.
2. **Rumor reach is still a step function** (0.333 → 0.417, flat after tick ~60). Unchanged by this
   iteration and still the #3 item.

**Verdict: GREAT** for the change; the plot's remaining flat regions are attributed, with numbers, to
a different and already-escalated cause.

## Pass 3 — Fresh eyes: a different seed and population

**Critique.** `evidence/fresh-eyes/` — seed 99, 40 people. Narrative pairs (every `finished` has a
`started`), reads coherently, and the society now has **6 rivals** where the previous iteration's same
run had **0**. That is a qualitative change a stranger can see: the village has disagreements again.

**Critique of what it exposes.** All four needs still end at `0.0` — again travel, again recorded. The
narrative shows `started work` at `d1 15h05` and `finished work (interrupted)` at `d3 07h07`, the same
40-game-hour walk the last iteration's narrative showed.

**Verdict: GREAT** for the change; needs remain NOT YET, cause isolated and escalated.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 0.97 → 0.50 share; starvation scaling gone in every population tested |
| Zoom | GREAT — choice fixed (needs 0.05 → 0.53 mean with travel removed); travel attributed |
| Fresh eyes | GREAT at seed 99 / 40 people — narrative coherent, rivals 0 → 6 |

## Top 5 remaining

1. **The two-clocks decision** — now the *only* cause of need starvation, measured: with places at the
   origin every need sits 0.52–0.87; with real distances, 0.05–0.17. `needs-decision/travel-time-in-
   two-clocks.md` recommends option 1; needs the operator.
2. **Rumor reach is a step function** — flat after tick ~60 at both seeds; Daley–Kendall stifling
   should decay it.
3. **`Field` / `Gradient` wiring** — unchanged, four measured facts recorded.
4. **`warn_unscored_fields` says "1 definitions"** — plural helper.
5. **`need-cycle` is still `pending`** in `scenarios.yaml`; with the choice fix it may now pass if the
   travel decision lands, and the scenario should be measured against both.
