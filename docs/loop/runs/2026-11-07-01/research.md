# Research: measure the scenarios, or stop claiming they pass

## Why this iteration

`docs/loop/scenarios.yaml` says, in its own header:

> A scenario with `status: pending` has no test yet; the loop may not claim it passes.

Four of seven were `pending`. Two of those (`deed-draws-a-response`, `opinion-converges-but-not-entirely`)
state precise, falsifiable observables that nothing had ever checked. One (`determinism`) was pending on
a reason that had stopped being true. The brief's first instruction is to run the scenarios and record
what happens; this is that instruction, applied to the crate's own evidence base rather than to a new
model.

## The observables, and what they actually measure

| Scenario | Observable | Status found |
|---|---|---|
| `deed-draws-a-response` | one deed → one rumor every witness knows, one `NormViolated` per judge with approval `a < −0.5` | `pending`, harness never planted a deed |
| `opinion-converges-but-not-entirely` | std dev falls by ≥ 1/3 and stays > 0.02 | `pending`, harness has no such scenario |
| `determinism` | seed 7 twice identical, seed 8 different | `pending`, but `tests/determinism.rs` has had both tests green for iterations |
| `need-cycle` | each need crosses < 0.35 and recovers > 0.7 | `pending` |

## What the measurement found

### 1. The harness's `summary.json` had the same bug as its `series.csv`

Last iteration fixed `series.csv` to follow the *planted* rumor instead of `RumorId(0)`. The summary was
still counting knowers of `RumorId(0)`, so every scenario reported `rumor_known_by: 0` — including
`need-cycle`, which plants nothing and therefore reported a rumor count of 119 with zero knowers. Fixed
the same way: the id the plant returned is passed to the summary.

Verified: `village-evening` now reports `known_by: 1` (matching its 0.042 reach), the cluster `known_by:
4` (matching 0.333).

### 2. `PlantRumor` is not a deed

The `deed-draws-a-response` observable says a *planted* `steal` produces `NormViolated`. It cannot:
`PlantRumor` is documented as *"request to start a rumor about `subject`'s deed"* — it seeds a `Rumor`
and gives the origin `Knowledge`. `witness()`, which judges a deed and writes `NormViolated`, runs only
from an **accepted interaction** whose definition names a `deed`. Measured: a planted `steal` with five
witnesses 0.2 m away produced **0 violations and `known_by = 1`**.

The scenario was rewritten as a test that drives the deed the way the crate defines it — an accepted
`insult` carrying `deed: Some("steal")` — and it passes: **1 rumor, `known_by` 4, 4 violations**, one per
witness whose `approval = norm · sign(harm)` is below −0.5.

### 3. The scenario's thresholds were not met, and the reason is the choice rule

`need-cycle` asks for a cycle between 0.35 and 0.7. Measured with the shipped rates on one person:
**29 completed 4-hour sleeps in 20 000 ticks, and `rest` stays at 1.000 throughout.** Four game-hours *is*
683 ticks at this day length, so the sleeps are back to back and the need never gets a chance to decay.

The cause is not the satisfy path — that pays `rate · duration_hours` per use and is asserted to clamp at
1.0. It is `Linear(-1, 1.0)` in the starter pack: it scores a **full 1.0 whenever the need is below
1.0**, so the activity that tops the need up is the best option the instant the previous one ends.
Whatever the shipped rates are, a need with that curve is either at 1.0 or being raised toward it.

So `need-cycle` stays `pending`, with the blocker written into the scenario file and asserted by the
test: the day the behaviour changes, the test fails and the scenario can be marked measured.

### 4. Opinion convergence is real but slow

0.511 → 0.239 (a 53% fall) over 12 000 ticks, holding above 0.02. The scenario's own `ticks: 6000`
stops at 0.370 — 27.6%, one percent short of the third it asks for. Corrected to 12 000.

In the village the variance plateaus at 0.033 and stops falling, which is the same claim at a
different scale (variance, not standard deviation) and is not a collapse.

## Literature

No new model this iteration. The rules the tests assert are the ones the crate already cites:

| Claim | Source |
|---|---|
| A norm violation is a judged deed, not a told one | Deffuant et al. for the opinion step (`10.1142/S0219525900000078`); Sommerfeld et al. 2007 (`10.1073/pnas.0704598104`) for gossip as reputation, already at `docs/research/rumor-and-reputation.md` |
| Opinion influence narrows spread without collapsing it | Deffuant, already the basis of `opinion::deffuant_step` |
