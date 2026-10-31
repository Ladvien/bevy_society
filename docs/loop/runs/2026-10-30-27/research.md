# 2026-10-30-27 — research: a society begins with empty larders

## The recorded defect

Pass 26 left this at the top of its backlog: "Needs hit the floor: at seed 99 (24 people, starter + village) **19 of
24 people reach food 0.00 within 6 days**, and every person's first crossing lands on one tick; cause not
investigated (supply, place capacity, or identical initial needs)."

## What a measurement says (HEAD `48e2260`, village of 24, seed 99, throwaway probe, 10 game-days)

"6 days" in pass 26's probe were 256-update days (the helper's `0.25 s` step against `seconds_per_day` 240), so the
run was **1.6 game-days**, not 6. The floor is a start-up transient, not a steady state:

| game-day | person-ticks with an empty larder | person-ticks with `food` < 0.05 | mean `food` | bakes | meals |
|---|---|---|---|---|---|
| 1 | **99.8 %** | 0.0 % | 0.55 | 4 | 11 |
| 2 | 34.0 % | **38.8 %** | 0.33 | 10 | 64 |
| 3 | 0.0 % | 6.2 % | 0.54 | 8 | 27 |
| 4 – 10 | 0.0 % | 1.4 – 10.7 % | 0.50 – 0.59 | 5 – 10 | 22 – 30 |

The cause, read off the activity stream:

1. A person is created with `food` at its `initial` level (1.0) and a `Stock` of **0.0 loaves**
   (`size_person_state`, `src/defs.rs`). No one has bread, and `eat` (`consumes: bread 0.25`) pays `satisfies` only for
   what `consumes` found, so a meal from an empty larder restores nothing.
2. `bake` is a dawn errand (`Hours(6, 7)`), and the run starts at 08:00. The first bread is made 22 hours in. Until then
   no `eat` is even chosen: `eat` is a `Logistic(-10, 0.35)` on `food`, so it wins only below about 0.35, which is
   01:00 on the first night, when everyone is asleep.
3. Everyone's `food` fell from the same 1.00 at the same 0.9 a day, so the whole village reaches the floor together
   (day 2, 38.8 % of person-ticks). The first crossing of any line is on one tick for the same reason.

Steady state is fine. Larders fill to the 6-loaf ceiling (119 loaves among 24 by day 7), a person eats about once a
day (25 meals, 24 people), and `food` averages 0.54. Only the first day is wrong, and it is wrong for every host:
`DefsFile::starter()` declares bread and consumes it, so a fresh society always begins hungry with a night of famine
ahead of it. It also pollutes anything that fast-forwards a fresh society (`fast_forward`, `preview_forward`): the
first game-day of every preview is the transient.

## What the literature says about it

This is the initialization-bias problem of simulation output analysis, and it has one standard remedy besides
discarding the transient: start in a state typical of the system once it is running.

- Suh 2016, *Mitigating Initialization Bias in Transportation Modeling Applications*: "Models are typically
  initialized in a particular, often 'empty' state and therefore must be 'warmed-up' for an unknown amount of
  simulation time before reaching a 'quasi-steady-state' representative of the systems' performance." The portion of
  output influenced by the arbitrary initialization is the initial transient.
- Murray & Kelton 1988, *Initializing for bias reduction*, DOI 10.1145/318123.318252: stochastic initialization, the
  initial state of a run drawn from a distribution built from a pilot phase, "is effective in reducing bias in the
  point estimate and increasing coverage of the interval estimate". The pilot phase here is the measurement above:
  the running village holds bread.
- Mahajan & Ingalls 2004, *Evaluation of Methods Used to Detect Warm-Up Period in Steady State Simulation*, DOI
  10.1109/wsc.2004.1371374: the alternative, truncating a warm-up, needs a rule to detect where it ends; a host
  that wants to *watch* a society from its first minute cannot discard its first day.

And why the state that is typical of a running village holds food in the house:

- Winterhalder, Puleston & Ross 2015, *Production risk, inter-annual food storage by households and population-level
  consequences in seasonal prehistoric agrarian societies*, DOI 10.1179/1749631415y.0000000025: "household-level,
  risk-sensitive adaption ... is a sufficient explanation for the origins of normal agrarian surplus and,
  consequently, of household-level incentives for inter-annual food storage". A household is a store before it is a
  producer.
- Longhurst 1986, *Household Food Strategies in Response to Seasonality and Famine*, DOI
  10.1111/j.1759-5436.1986.mp17003005.x: the first seasonal coping strategy of a rural household is "drawing down of
  stored product".

None gives a number of loaves; the number is a design default (below).

## The formula

Nothing to fit: the larder is a starting level, like a need's `initial`. A person is created holding `initial`
units of each good (clamped to `Tuning::stock_ceiling`), exactly as a person is created with each need at its
`initial` level.

## The value, and where it comes from

`1.0` loaf a head for bread. Three of the crate's own numbers meet there, which is why it is the least arbitrary
choice rather than a measured one:

- `Tuning::price_full_store` 1.0: what the crate calls a full store (its doc: "a shade under one loaf a head a day").
- `Tuning::trade_reserve` 1.0: a giver must hold *more than* this to share, so a starting larder of exactly 1.0
  begins with nobody holding a surplus. No `share` fires on a stock nobody earned.
- A meal is `0.25` loaf (`eat`'s `consumes`) and the village eats about once a day, so one loaf is four days of
  food: enough to reach the first dawn bake (22 hours) and not so much that `bake` is out-competed on day 1.

The ceiling `6.0` is where a running village ends (119 of 144), so `1.0` is a sixth of steady state, not the state
itself; the transient that remains is the one the village builds in a day by baking.

## What believable output looks like

- Day 1: no person-tick with an empty larder; the first meals come from the person's own larder at the first meal
  hour, not from a bake.
- Day 2: `food` below 0.05 on a share of person-ticks in the range days 3-10 show (1-11 %), not 39 %.
- Day 3 onwards: unchanged in kind. Bread still rises to the ceiling, `share` still fires only above the reserve,
  meals per day stay near one a person.
- `initial: 0.0` (the serde default, and every pack that does not write the field) reproduces HEAD bit for bit.

## Not in this change

Identical starting needs (everyone at 1.00) make every first crossing land on one tick. Measured above as the second
cause, but it is a cosmetic lockstep once larders exist: food 1.00 falling at 0.9 a day is a smooth curve that only
the hunger gate on `eat` bends. Heterogeneous starting levels or metabolisms (the endowment spread of agent-based
artificial societies) need a per-person draw, with the determinism order that implies, and are a change of their own.
