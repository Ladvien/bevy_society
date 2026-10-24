# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: is the reported activity honest?

Claim: no. A person walking to bed is reported `sleep` while being paid nothing, and that accounts for a
reported 56.8% asleep against the 28.6% the run's own sleeps allow.

**Evidence.** `evidence/reported-asleep.png`, the per-tick `rest` trace, and `tests/travel.rs` (red).

**Critique — the weakest point of this iteration.** The test that pins the defect is `#[ignore]`d, so the
default suite is green while a known falsehood ships. That is deliberate — a red test blocks every later
iteration — but it means the crate's own CI does not fail on a defect the loop knows about. The mitigation
is that the decision file names it and `--ignored` shows it; the risk is that nobody runs `--ignored`.

**Verdict: GREAT** for the finding, with that risk named.

## Pass 2 — Zoom: four wrong hypotheses

**Critique — this is the pass that matters, and it is a record of being wrong four times.**

| hypothesis | what killed it |
|---|---|
| a `Place` activity overruns its duration | `elapsed_h = 8.004` on every ending sleep |
| `satisfies` pays twice per tick | 2 h at 0.1/h paid **0.2010** — exact |
| `best_telling`'s tie-break starves a fresh rumor | three tests passed with **no code change** |
| a probe measuring overlapping sleeps | fixed with a queue; real range 8.00–28.00 |

The second is the one worth keeping: my first version used 1.0/h, so a 2-hour activity paid 2.0 into a
ceiling of 1.0 and read **1.000** — which looked exactly like a half-rate payout. **The need's clamp, not
the crate, was the cause**, and I only saw it by instrumenting the payout loop and counting calls
(343 × 0.005859 h = 2.010).

Each disproved hypothesis cost a probe. The habit that eventually worked was not "think harder" but
**instrument the thing and read a number** — which is what found the `rest` decay rate.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: what does a host see?

**Critique.** A host reading `Doing` gets a lie: `activity: sleep, arrived_day: None` renders as "Bram is
sleeping" in any HUD. The crate's own `activity_barks` places a bark at `Doing.target` and picks its line
by activity, so a villager says the `sleep` bark while walking to bed. `report_to_triggers` reports the
activity, so a Rhai trigger on `Activity(tick)` sees it too.

**Critique.** The recommendation — a reserved `travelling` activity with the target in its own field —
makes every one of those readers right by default. Option 1 (document that readers must check
`arrived_day`) is cheaper but leaves three in-crate readers wrong, which is how this lasted.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — the defect is measured, reproduced in isolation, and pinned by a red test |
| Zoom | GREAT — four hypotheses disproved by measurement, including one caused by a need's clamp |
| Fresh eyes | GREAT — every in-crate reader of `Doing.activity` is wrong today, and the fix makes them right |

## Top 5 remaining

1. **`Doing` while travelling** — `needs-decision/what-doing-reports-while-travelling.md`. Un-ignore
   `tests/travel.rs` when it lands.
2. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
3. **`opinion-converges-but-not-entirely` is not implemented in the harness** — it has a test but
   `--scenario` rejects the name.
4. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip is checked against a third
   configuration, and its "23 of 25" figure is therefore measured under settings the spec does not declare.
5. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
