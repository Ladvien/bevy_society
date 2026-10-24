# Review

Three passes, each with a written critique before the verdict.

## Pass 1 — Behaviour: does the village behave better with a fitted timeout?

Claim: yes, substantially, and it is a settings value rather than a model change.

**Evidence.** Same seed and spawn, 67 200 ticks:

| | 20 h | **4 h** |
|---|---|---|
| reported asleep | 56.8% | **36.4%** |
| reported working | 14.8% | **21.5%** |
| total need mean | 0.322 | **0.526** |

and at seed 42: 36.2% asleep, 0.511 total.

**Critique.** The 36.4% is still above the 33.3% that one 8-hour sleep a day implies, so walks are still
being mislabelled — the `Doing`-while-travelling decision is untouched, exactly as intended. This iteration
reduces the *scale* of the lie from 2.5× to 1.1×; it does not remove it.

**Verdict: GREAT.**

## Pass 2 — Zoom: reading the plot

**Critique.** `evidence/timeout-fitted.png` shows both effects plainly. The asleep panel: the green line
(4 h) sits near the dotted 8-per-day mark most nights and peaks around 15, while the red line (20 h) spikes
to 22. The need panel: green holds 0.45–0.60 for the whole run against red's 0.20–0.40.

The one thing to check and not find is a compensating regression: `rest` falls 0.538 → 0.395. That is
correct — people no longer sleep for hours they have not earned — and the other three rise enough that the
total goes up by 63%.

**Critique of a trap avoided.** The cluster scenario looked like it regressed (0.583 → 0.333), and the
temptation was to restore 20 hours. Measuring both showed **0.167 at 20 h**, so 12 h is the better of the
two and neither passes. The 0.583 predated the `contact_radius` fix and was a different run.

**Verdict: GREAT.**

## Pass 3 — Fresh eyes: what the harness was hiding

**Critique.** Two harness bugs, both silent:

| | consequence |
|---|---|
| `contact_radius` declared, never applied | a scenario's stated value did nothing. It was right by coincidence (4.0 is the crate default) and would have been wrong for any other number |
| `scenarios.yaml` parsed once per tick | **57 000 warnings** in one run, each a filesystem-free but real `eprintln!`, drowning any message a reader might need |

The second is the kind that hides the first: the output was so noisy that the one warning worth reading —
"the harness ignores `contact_radius`" — was repeated 57 000 times and unreadable. `cargo test` never runs
an example, so no gate caught either.

**Critique.** The mitigation is the same as the last four iterations: a test. `tests/scenario_spec.rs` now
asserts the cluster states a `contact_radius`, so a scenario that declares one the reader does not map fails
in CI.

**Verdict: GREAT.**

## Summary

| Pass | Verdict |
|---|---|
| Behaviour | GREAT — 56.8% → 36.4% asleep, need total 0.322 → 0.526, holds at a second seed |
| Zoom | GREAT — both panels agree; `rest` falling is correct, and the cluster's apparent regression was measured not assumed |
| Fresh eyes | GREAT — two silent harness bugs found, one of which was drowning the other |

## Top 5 remaining

1. **`Doing` while travelling** — the labelling defect stands; 36.4% is still above 33.3%.
   `needs-decision/what-doing-reports-while-travelling.md`; un-ignore `tests/travel.rs` when it lands.
2. **`rumor-crosses-a-cluster` is `pending`** — 0.333 at 12 h against a 40% observable. Two groups 40 m
   apart need a 10.7 game-hour walk: the gap, the timeout, or a bridge activity.
3. **`need-cycle`'s observable** — three needs stay above 0.5 at these rates; a design question.
4. **`examples/capture.rs` still hardcodes 8.0 s/day** — the README clip's "23 of 25" is measured under
   settings the spec does not declare.
5. **Retire untellable rumors** — right for memory: ~5 MiB at 5 000 over a long run, 87% untellable.
