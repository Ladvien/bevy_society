# Should everyone go to bed at the same hour?

> **Assumed decision (owner may override), 2026-10-31 (pass 2026-10-31-02): a `Chronotype` component the host writes, a second input `BodyHour`, an `unavailable` flag on activities, and the starter pack's `sleep` uses both.** The research is `docs/research/chronotype.md`; the run is `docs/loop/runs/2026-10-31-02/`.

## The problem

The starter pack chose `sleep` on `Hour(22, 6)`: one window, the same for everyone. Measured on the village (four seeds, 17 game-days), 60-64 % of all sleeps begin in a single clock hour; and 7-9 % of all exchanges (349, 378, 324 and 362 of ~4 400) were aimed at a person who was asleep, because a sleeper stands in a home place's `near` list like anyone else. A village that goes dark at 22:00 sharp reads as a clockwork village, and the second number is a defect no one could see while everybody slept at once.

## Decisions taken, with the options

**1. Where does a person's clock come from?**

| | |
|---|---|
| **(a) A component the host writes, like `Personality`** | **Picked.** `Chronotype(f32)`, `Chronotype::sample(rng, sd)`, `PersonDef::chronotype_hours`. No new random draw inside the crate, so no existing seed's stream moves; a host with its own roster (or an editor) authors it. |
| (b) Derived from personality | Rejected: the Big Five explain a few per cent of chronotype, far too little to produce the measured spread, and a derived value hides a free parameter inside a trait. |
| (c) The crate draws it at spawn from `Tuning::chronotype_sd_hours` | Rejected: a hidden draw in `Add<Person>` shifts every trajectory of every seed, and a host with a roster would have to override it. Reversible later if the owner wants it default-on: it is one observer. |

**2. How does the body's clock reach a consideration?**

| | |
|---|---|
| **(a) A new input, `BodyHour`, beside `Hour`** | **Picked.** `Hour` stays the village's clock (shop hours, the baker's dawn, the working day), so an owl who sleeps late next to a workshop that opens early is the social jetlag of Wittmann et al. 2006, and the author chooses which clock each rule reads. `Input` and `Signal` gain a variant (breaking for exhaustive matches). |
| (b) Shift `Hour` for a person who has a chronotype | Rejected: every window, including the working day, would move with the person, and the second clock Wittmann describes could not be written. |

**3. How is a sleeper kept from being greeted?**

| | |
|---|---|
| **(a) `ActivityDef::unavailable`, per activity** | **Picked.** A person who has arrived at an `unavailable` activity is in nobody's `near` list, and a walk toward them ends. Data, not code: the pack marks `sleep`; a host marks `pray`, `fight`, whatever should not be interrupted. Not applied while the person is still walking to the bed. |
| (b) Insert `Dormant` while asleep | Rejected: `Dormant` also stops the crate choosing for the person and ends their own activity (`ActivityEnded { completed: false }`), which is the opposite of sleeping. |
| (c) Nothing | Rejected: with body clocks an owl awake at 01:00 would greet the village's sleepers 12-17 % of the time (646, 586, 493 and 670 of ~4 000 exchanges, `evidence/variants/sleepers-open-*`). |

**4. Does the starter pack change?**

| | |
|---|---|
| **(a) `sleep` reads `BodyHour` and is `unavailable`** | **Picked.** With no `Chronotype` anywhere `BodyHour` equals `Hour`, so the first half changes nothing (the harness run with `--sleepers-available` is byte-identical to 0.8.0 at seed 7: `summary.json`, `series.csv`, `narrative.txt`). The second half does change the shipped society, because it removes the 7-9 % of exchanges that were aimed at sleepers: at four seeds friend ties go 41 / 43 / 37 / 42 → 48 / 66 / 51 / 69 and rival ties 36 / 36 / 35 / 38 → 48 / 19 / 31 / 24. All 99 test binaries stay green, including `friendship_bound`, `label_hysteresis`, `rumor_book`, `starting_larder` and `determinism`, with no assertion touched. |
| (b) Leave the pack alone and ship the three pieces opt-in | Reversible and touches no pin, but leaves the 7-9 % defect in the pack every host starts from. |

To take (b) after the fact, set `unavailable: false` on `sleep` in `assets/starter.society.ron` (and `BodyHour` back to `Hour` if no host wants clocks); the harness's `--sleepers-available` reads the society that results.

## What is not decided here

- A default-on population (option 1c). A host that wants one writes `Chronotype::sample(&mut rng, Chronotype::POPULATION_SD_HOURS)` where it writes `Personality::sample`.
- `ctx.body_hour(person)` for scripts: the snapshot does not carry the offset yet; a trigger can read `ctx.hour` (the village's clock) as before.
- The interrupted-sleep rate (10-18 %): body clocks do not touch it. At 64 s/day the cause is the walk home; see `travel-time-in-two-clocks.md`.
