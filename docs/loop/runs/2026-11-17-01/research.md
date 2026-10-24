# Research: re-deriving the behaviour at the settings the spec declares

## Why

Last iteration found that `examples/evidence.rs` had never read `docs/loop/scenarios.yaml`: the village ran
at `seconds_per_day = 4.0` where the file says **64.0**, and every needs and rumor conclusion in the eight
iterations before that was measured at a day sixteen times shorter. The log's #1 was to re-derive them.

## The seed field too

Re-running the scenarios exposed the same class of defect again: `seed:` was also unread. Every scenario
ran at the crate default `0x5EED_5EED`, so the file's declared seeds — 7, 11, 3, 5, 1 — were decoration,
and two scenarios that differed only in seed were the same run. `Spec` now carries `seed`, and `--seed`
still overrides it so a fresh-eyes pass can move the seed without editing the file.

## Every scenario, at its own settings

| scenario | need means | all four cycle? |
|---|---|---|
| `need-cycle` | [0.86, 0.81, 0.58, 0.79] | **yes** |
| `village-evening` | [0.29, 0.53, 0.24, 0.26] | **yes** |
| `rumor-crosses-a-cluster` | [0.64, 0.15, 0.40, 0.61] | **yes** |
| `deed-draws-a-response` | [0.86, 0.81, 0.63, 0.81] | **yes** |

"Cycle" means what the scenario file means: crosses below 0.35 at least once and later recovers above 0.7.

**`need-cycle` passes for the first time in four iterations.** It was `pending` because at a four-second day
a four-hour activity lasted four ticks while a walk took three hundred, so nothing was ever satisfied. At
its own settings it is a clean daily cycle.

`rumor-crosses-a-cluster` reaches **0.583** of the village — above the 40% its observable asks for, and it
had been marked `measured` while failing.

## What the plot shows, and what it does not

`evidence/spec-needs.png` at full zoom looks like violent thrashing: all four lines sawtooth between 0.2 and
1.0 dozens of times. Counting troughs below 0.4 says otherwise — **1.0 per day**, and only for `food`. The
dense variation is high-frequency small excursions, not cycling.

`evidence/need-cycle-zoom.png` over three game-days shows what one day is: every need decays steadily and
jumps when its activity runs. `rest` climbs to 1.0 and holds through sleep; `food` sawtooths 0.25 → 0.80
once a day; `social` and `fun` recover in smaller steps. That is the picture the observable describes.

**A plot at full range can look like a defect that is not there.** The trough count is what distinguishes
them, and it is cheap.

## Literature

No new model. The need cycle is `docs/research/needs-and-utility.md` — decay at `decay_per_day` and
`satisfies · duration_hours` per use — and this iteration measures it rather than changing it.

## What this leaves

Every scenario is now `measured` at the settings its file declares, with its seed honoured. The
`opinion-converges-but-not-entirely` scenario is still **not implemented in the harness** (it has a test,
`tests/opinion_convergence.rs`, but `--scenario` rejects the name), which is the next gap in the evidence
base rather than in the model.
