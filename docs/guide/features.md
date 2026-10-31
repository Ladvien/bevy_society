# What you get

Each bullet the README summarises is expanded here: the mechanism, the paper behind it and the numbers measured on this crate's own villages. Every rate named below is a field on `Tuning`; a default either comes from a paper or says it is a design default. The DOIs are in [`docs/research/`](../research/README.md).

## People with a mind

Big Five personalities, moods on a pleasure/arousal/dominance axis, an OCC emotion set, needs that decay, and memories that fade. [personality-and-mood](../research/personality-and-mood.md), [memory](../research/memory.md) Each person can **keep their own hours**: a `Chronotype` is how far their body clock runs behind the village's, `Input::BodyHour` reads it, and a sleeping person is out of everyone else's reach. Measured on the 24-person village over 17 game-days, four seeds: 60-64 % of the sleeps used to begin in one clock hour and 7-9 % of all exchanges were aimed at somebody asleep; with a population of lark and owl offsets the busiest hour holds 19-27 % of them and under 0.1 % of exchanges find a sleeper. A person with no offset keeps the village's own clock. [chronotype](../research/chronotype.md)

## Daily life and trades

Utility-AI decisions over tagged places and other people, with travel, arrival, durations and need satisfaction. A **task is not a skill**: an activity can declare a `profession`, and a `Skill("craft")` consideration reads the trait that capability comes from, so who does the skilled work is decided by comparative advantage rather than by assignment. Nothing is gated — an unskilled villager can still take the task, at a lower score. [needs-and-utility](../research/needs-and-utility.md) A **full place costs** instead of vanishing: a place with no free seat keeps `1 - Tuning::crowding_penalty` of its score, so a villager with an alternative goes elsewhere and one with none still arrives, while `capacity: 0` is closed. Measured, this holds a village's rival density at 2.9-6.1% of pairs as it grows from 24 to 48 people in a 30-seat village, where the old veto took it to 23% and lost the working day. [capacity](../loop/runs/2026-10-25-10/eval.md)

## A week

`Input::Week` and a `Days(from, to)` curve are the day-of-week analogue of `Hour` and `Hours` (with `SocietySettings::start_weekday` and `ctx.weekday()`), so a pack can shut the workshop on rest days without a script. The starter pack has no week; `examples/assets/weekly.society.ron` is an opt-in overlay with two rest days. **Shutting `work` alone damages the village** (the freed hours go to company: friend ties 51 → 25 and rival ties 44 → 80 at seed 7), so the overlay adds a `laze` that takes two-hour spells of nothing; measured over seven runs, a rest day then has 7.5 approaches a villager against 10.7 on a workday, 3.6 different people met against 4.7, and 44 % of approaches to a friend against 37 % (Sekara et al. 2016: weekends are spent with a few selected friends), at the price of a `social` need 0.05 lower. [weekly-rhythm](../research/weekly-rhythm.md)

## Relationships

Directed ties carrying affinity, trust, respect and familiarity, with power-law forgetting, structural balance, a Dunbar-style cap and friend/rival labels. Affinity is **clamped to `±Tuning::affinity_ceiling` (0.35)** and a `Friend` needs trust as well, so a large town with daily contact keeps friendship scarce: measured, 60 people over 30 game-days reach 155 friend ties of 3 540 ordered pairs (**4.4%**). A person who has been **turned down can be made to not ask the same person again**: set `Tuning::rebuff_memory_hours` to `Some(12.0)` (off by default; Maner et al. 2007, excluded people do not seek reconnection with those who excluded them) and, measured on the 24-person village and towns of 40 and 60, a refused approach that was followed within the hour by another to the same person 24-29 % of the time is followed by one 0.3-1.1 % of the time, and people approach 5.2-6.3 different people a day where they approached 4.1-5.1. [relationships](../research/relationships.md), [rebuff](../research/rebuff.md), [the measurement](../loop/needs-decision/friendship-has-no-diminishing-return.md)

## Opinions, rumors, reputation and factions

- **Opinions** — bounded-confidence influence, stubborn anchors and conformity to a clear group majority. [opinion-dynamics](../research/opinion-dynamics.md)

- **Rumors and reputation** — gossip with the Daley–Kendall stifling rule, belief scaled by trust, witnesses who judge deeds and hold grudges. [rumor-and-reputation](../research/rumor-and-reputation.md)

- **Factions and status** — in-group affinity, standing as conferred respect, contests and intergroup contact. [groups-and-status](../research/groups-and-status.md)

## Crowd fields

A stigmergy grid of presence and commotion per cell, readable as a `Field(Crowd)` consideration and walkable as a `Gradient` target, so an activity can be "join the crowd" without naming a place. [needs-and-utility](../research/needs-and-utility.md)

## Scripts and decision traces

- **Scripts** — Rhai triggers over a per-tick snapshot, with a `ctx` reader and an `fx` writer that cannot see its own writes. [scripting](scripting.md)

- **Decision traces** — why a person chose what they did: every option weighed, its score, and how far the chosen one fell short of the best. Insert a `DecisionTrace` and read it; recording changes no outcome. [decisions](decisions.md)

## Dialogue

Barks and conversations through `bevy_speech_bubbles`. [dialogue](dialogue.md)

## Obstacles and steering

Solid buildings that walkers path around, as hard walls in the ORCA solver, with a resolver that pushes anyone who lands inside back out. [getting started](getting-started.md#solid-buildings-obstacle)

## A basic economy

Goods as quantities rather than currency, a per-person store, `produces`/`consumes` on a task, a price per good **read from how full the village's stores are, per head** — empty is double, full is half, the shape the corpus's own game-economy model uses ([economics](../research/economics.md)) and readable from a script as `ctx.price("bread")`, plus **goods that change hands**: an interaction can `actor_gives` / `target_gives` a good, and a `Surplus` consideration scores a transfer by the joint surplus of two stores. In the village that is sharing — measured over nine seeds, it cuts the spread of bread across the village by 38% — and the engine does a two-way swap too. Nothing gates: an empty store never stops anyone eating — but it stops the meal **feeding** them, because a task's `satisfies` is paid in proportion to what its `consumes` actually found, so a meal from an empty larder costs the walk and restores nothing while half a loaf feeds half. Measured, that closed a village fed by arithmetic: 4.80 loaves baked a game-day against 21.06 meals, ~77% of them out of an empty shelf. A person starts holding each good's authored `initial` (the starter pack gives one loaf), so a fresh village is fed at its first dawn; with every larder empty, 39% of its second game-day's villager-ticks were spent with `food` below 0.05. A consideration can read the actor's own shelf (`Stock("bread")`), so a pack can make production answer to the larder instead of the clock: a hungry household with nothing bakes, one holding bread does not (`examples/assets/provisioning.society.ron`; on the evidence town of sixty it is the difference between a store that holds and one that runs out). [economics](../research/economics.md) · [authoring](authoring.md#goods-stores-and-trade)

## Persistence

The whole society captured and restored through `serde`: ties, memories, rumors, the clock, the random stream and every trigger's bookkeeping, keyed by your own ids, so the ticks after a reload are the ticks that would have followed. [persistence](persistence.md)

