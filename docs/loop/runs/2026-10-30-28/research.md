# 2026-10-30-28 — research: the household provisions itself (`Input::Stock`)

## The recorded defect

Pass 27's top item: "**The 60-person town starves for good.** 100 % of villager-ticks at `food` < 0.05 from about
game-day 17 with no bread anywhere, before and after the starting larder; 0-2 bakes a day." Pass 27 hypothesised that a
starving person scores `eat` above the dawn `bake` and that the two share the same two places; it was untested.

## What a measurement says (HEAD `7ead295`)

Town of 60, seed 7 (the evidence harness's village with `--people 60`). New in this pass: the harness writes
`activities.txt`, a per-game-day count of each activity's starts, completions and interruptions, and a histogram of the
clock hour each was *chosen* at (`examples/evidence.rs`, `Ledger`).

**It is the economy, not the walking.** The harness's 64-second day makes a walk cost game-hours (pass 27's item 3),
so the town was re-run at the library's own 240-second day (`--seconds-per-day 240`, 12 game-days):

| game-day | at the floor on `food` | bread a head | bakes | meals done / started |
|---|---|---|---|---|
| 2 | 20 % | 1.13 | 5 | 48 / 71 |
| 5 | 32 % | 0.71 | 1 | 41 / 68 |
| 8 | 57 % | 0.25 | **0** | 81 / 101 |
| 10 | 75 % | 0.09 | 1 | 100 / 119 |
| 12 | **86 %** | 0.05 | **0** | 117 / 137 |

Walking is not what starves it: 100+ meals a day are *started* and reach a table at the 240-second day. They find
nothing, because bread a head falls from 1.1 to 0.05 and **nobody replaces it**. At the 64-second day (10 game-days)
the same town makes 3 bakes in 10 days (60 people) against 3.9 a day in a village of 24.

**Why no bake: the hour list of the choice.** At the 64-second day, `activities.txt` (all ten days) shows who decided at
hour 6, the only hour `bake` scores above zero (`Hours(6, 7)`): about 30 decisions a day, of which **6 in ten days were
`bake`** (2 %), against 20 `eat`, 123 `compliment`, 67 `gossip`. Three things produce it:

1. **The window is one hour wide and a decision needs to land in it.** A person decides when an activity ends; at 22:00
   a sleep lasts eight hours, and in the crowded town it starts after 23:00, so the sleeper wakes at 07:30. Half the town
   never has a decision in `[6, 7)` at all.
2. **`bake` is a lottery ticket against nine others.** Scores are drawn by `weight · score` among the options within
   `tier_cutoff` of the best. `bake` scores about 0.6 (the `craft` skill, compensated), `compliment` about 0.9, `eat` about
   0.97 when starving: a starving person at the dawn window is one draw among eight.
3. **`bake` pays no attention to the larder.** Its score is the clock and the craft skill. Whether the shelf holds forty
   loaves or none, a person is as likely to bake. Nothing in the pack closes the loop from "the shelf is empty" to "make
   bread", which is the one feedback a stock-and-flow economy needs.

**Supply was not scaling because nothing responds to the stock.** The pack's own comment already says "the supply does
not scale with the population, and that is the trap": a fixed one-hour dawn window gives a fixed number of bakes a day
whatever the town eats. A person with an empty shelf cannot even *see* it from a consideration: `Need`, `Trait`,
`Affinity` and the others read the person, the tie or the clock; `Surplus` reads two stores but needs a `Person` target and
reads the *joint* surplus of a trade. The actor's own holding of a good is not an input.

## What the literature says

- **A balancing loop closes on a stock** (Meadows, *Thinking in Systems*, corpus doc `Thinking in Systems A Primer`; a
  book, no DOI). "Economies are extremely complex systems; they are full of balancing feedback loops with delays." A
  household that tops up its shelf when it runs low is the textbook loop; a producer that works by the clock is an open
  loop, stable only if the clock's output happens to equal the consumption. Zunic's production-consumption model (corpus
  doc `eth-cgl-ar_games-Zun16a`, §4): "if the parameters are chosen such that the inflow of resources into the system
  equals the outflow, then the economy is stable ... However, by upsetting this balance we can simulate ... shortages".
  A town of 60 on a pack tuned at 24 is that imbalance, and an open loop cannot recover from it.
- **Utility scores what an action can deliver** (Game AI Pro, "Utility Systems", corpus doc `Game AI Pro Collected
  Wisdom`, on *The Sims*): "the fact that I am 'very hungry' combined with the availability of 'poor food' would certainly
  be more attractive than if I was only 'a little hungry'". The score of eating is the need *and* what is on offer. Smart
  objects "advertise" an affordance whose precondition is on the state of the user (Zun15a, corpus doc
  `eth-cgl-ar_games-Zun15a`): "a precondition ... checks if f can be executed based on their current states". `eat` here
  scores by hunger alone, so an empty-handed person walks to a table that cannot feed them: the pack's own comment says
  "the failure is priced, never prevented" and it is priced only in the payout after the walk.
- **Appetitive, then consummatory** (the ethology of motivated behaviour, Woodworth 1918 and Konorski 1976, as set out in
  the corpus doc `10.1002/hipo.22488`): "animals are usually separated from motivational stimuli by distance and time, they
  must allocate considerable resources toward stimulus-seeking behavior ... an on-going sequence of 'preparatory'
  (appetitive) actions leading to 'consummatory' events". Eating is the consummatory act. With nothing to eat the
  behaviour that follows hunger is the preparatory one: making or finding food. `bake` is that act, and it is driven by
  the clock.
- **Prepotency** (Maslow 1943, DOI `10.1037/h0054346`, corpus doc `10.1037_h0054346`): the physiological needs "are the most
  pre-potent of all needs"; a starving person is "dominated by the physiological needs"; but "a want that is satisfied is
  no longer a want". Not this pass: hunger still competes with the other options by score here (see "Not done").
- **The household is the producing unit** (Winterhalder, Puleston & Ross 2015, DOI `10.1179/1749631415y.0000000025`, cited
  by pass 27). A person here has a `Stock` and a `bake` that fills the baker's own shelf, so the household is already
  the unit; what it lacks is the household's own rule for when to produce.

## What believable output looks like

A town of any size holds a roughly constant store of bread a head, because each household tops up its own when it is low:
production is the sum of what the households' shelves ask for, so it **scales with the population by construction**,
which the dawn-hour clock could not. The price (`storage_price`, from the same stores) stays near its resting value
instead of pinning at the ceiling. A hungry person with bread eats; one without it does not walk to an empty table and
tries to make some. Bakes are a few a day in the village (it eats about 6 loaves a day: one bake a day and a half) and
more in the town, on days when stores run low, not on every day.

## The decision (recorded; the owner may override)

An **input**, not a new rule in the engine: `Input::Stock(good)`, the actor's own holding of a good clamped to `[0, 1]` like
every input, so the pack says in its curves what low means and the crate gains no economic constant. The alternatives:

1. A `Script` consideration with `ctx.price("bread")`, no code change. Rejected: the price is *village-wide*, which is
   information no household has; a script is a Rhai call per person per option per tick, which is the one cost this crate
   spends carefully; and the actor's own shelf is the thing a producer actually reads.
2. An engine rule that multiplies every activity with a `consumes` list by what the shelf holds. Rejected: it hides in
   the engine what the pack should say, it would change every host's pack (including `eat` in a pack whose bread is
   brought in by a host), and pass 27's note records that gating a dominant activity changed the whole social fabric.
3. Prepotent hunger by `rank`. Not this pass: see below.

## Not done (and why)

- **Prepotency of hunger.** At the 64-second day a fed town still holds 20-49 % of its person-ticks at the floor, and the
  village 8-27 %, because `eat` is one draw among many (a starving person's `eat` and a flattered person's `compliment`
  are both near 1). `rank` can make a need pre-potent (Dill's buckets, `docs/guide/decisions.md`), but a rank-1 `eat` needs a
  curve that is exactly zero above the line, a measured change to every society's day, and a second experiment. It is
  the next backlog item.
- **A bake that scales with the price.** The price is an observation the pack could read in a script; a household
  reading its own shelf already scales production, and nothing is gained by the crowd's information.
