# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is a person's rumor memory bounded?

Claim: `Knowledge` plateaus like `Memories` and `Ties` do.

**Evidence.** `tests/knowledge_bound.rs`, capacity 32, 3 000 ticks, book of 325 rumors: max entries per
person **287 → 32**, exactly the cap. In the long benchmark `knowledge mean` sits at 64.0 from tick 500
to 5 000 while the book grows to 8 357.

**Critique.** The stronger claim — that behaviour is *unchanged* at the defaults — is the one I
actually had to prove, because a cap that forgets a rumor changes what a person says next. Verified
two ways at the same seed: all **4 200** `series.csv` rows are byte-identical, and the narrative is
identical. The cap cannot bind in a 24-person village whose book peaks at 34, and it does not.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the artifacts closely

**Critique — and this pass changed what I shipped.** Plotting the fix exposed two of my own mistakes:

1. The first version of the harness change spliced `RumorId(0)`'s reach in *front* of the planted
   rumor, so the line fell to zero at the plant tick. That is not a thing that happened; it was two
   different rumors drawn as one series. The plot made it obvious; the code did not.
2. With the splice removed, the village's planted rumor reads **0.042 = 1/24** — it never spreads at
   all, where the old column reported a flat 0.417. Tracing it: the origin gossiped **twice in 3 600
   ticks** and its pick was stifled or ineligible both times. So the scenario picked an arbitrary
   villager as town crier. The model is not at fault, and saying so with the trace is more useful than
   a nicer number.

The cluster scenario's curve is the one that vindicates the harness change: 0 → 0.083 → 0.167 → 0.25 →
0.333 over ~350 ticks, against a single step in the old column. Same endpoint, real diffusion.

**Critique of the shipped scope.** I found a second, larger defect (93% of the rumor book is
untellable, and the per-tick walk over it drives a 4× cost growth) and did **not** fix it: changing
`alive` changes what a host reads. Filed with measurements and a recommendation. Shipping the cap
alone is honest but partial, and `perf.md` says so plainly rather than implying the growth is solved.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: a different seed and population

**Critique.** Seed 99, 40 people, plus the cluster at seed 11. The cap is population-independent by
construction — it is a per-person bound — and the 200-person probe independently shows it holding at
64 while the book grows to 8 357. The narrative is unchanged line for line at 24 people.

**Critique.** What a host developer would ask: "does my run get slower the longer it runs?" The answer
is **yes**, still, by 4× over 3 000 ticks, and this iteration reduces that by only 2.3%. That is the
honest headline and it belongs in the log rather than being buried under a `knowledge_capacity` field.

**Verdict: GREAT** for the cap and the harness; the remaining growth is a filed decision, not an
unnoticed regression.

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 287 → 32 at capacity 32; defaults byte-identical |
| Zoom | GREAT — caught two harness errors of mine, and traced the village plant to its origin |
| Fresh eyes | GREAT — cap holds at 40 and 200 people; remaining 4× growth is the filed decision |

## Top 5 remaining

1. **Retire untellable rumors** — 93% of the book cannot spread again, and the per-tick walk over it is
   a 4× cost growth. `needs-decision/retiring-untellable-rumors.md`, recommends option 1.
2. **The two-clocks decision** — still the only cause of need starvation in a real layout; measured
   both ways two iterations ago.
3. **The village's rumor origin** — the scenario asks an arbitrary villager to be town crier; the
   planted rumor reaches 1/24. Either pick a teller that gossips or accept a lower target.
4. **`Field` / `Gradient` wiring** — unchanged.
5. **`warn_unscored_fields` says "1 definitions"** — plural helper.
