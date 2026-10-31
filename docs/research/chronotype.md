# Chronotype: every person keeps their own hours

`src/mind.rs` (`Chronotype`), `Input::BodyHour` / `Signal::BodyHour` (`src/defs.rs`, `src/behavior.rs`), `ActivityDef::unavailable`. Run notes with the evidence: `docs/loop/runs/2026-10-31-02/`.

## The problem

The starter pack chose `sleep` on `Hour(22, 6)`: one window, read off the wall clock, the same for everyone. On the village at seed 7 (24 people, 17 game-days at 64 s/day), 280 of 450 sleeps began in the 22:00 hour, 81 in the 23:00 hour, and all 24 people woke within the same two hours. The crate had been warning about its own consequence for a long time (`warn_unreachable_places`: "They all go to bed at the same hour, so the queue behind the disc times out"). It also produced a second defect nobody had measured, because a village asleep together has nobody awake to notice it: **8 % of all exchanges were aimed at a person who was asleep** (349 of 4 305 at seed 7; 324-378 at the other three seeds), because a sleeper stood in a home place's `near` list like anyone else.

## Sources

- **Roenneberg et al. 2007**, *Epidemiology of the human circadian clock*, Sleep Medicine Reviews 11, DOI `10.1016/j.smrv.2007.07.005`: chronotype is the phase at which a person's clock entrains to the day, measured by the middle of sleep on free days.
- **Fischer, Lombardi, Marucci-Wellman & Roenneberg 2017**, PLoS ONE 12(6):e0178782, DOI `10.1371/journal.pone.0178782` (n = 53 689 diaries): mid-sleep is near-normal at every age; 25 % of people earlier than 02:24, 50 % between 02:24 and 04:15, 25 % later than 04:15 (a normal fit has a standard deviation of 1.37 h); the standard deviation by age and sex runs 1.57-2.65 h with recall noise included; sleep duration is unrelated to chronotype (b = -0.0004). The same paper's worked example is the model's arithmetic: "assuming an 8 h sleep duration, a chronotype of 4:00 falls asleep at midnight and wakes up at 08:00".
- **Reiter, Sargent & Roach 2021**, DOI `10.3390/clockssleep3020021`: MSFsc 04:38 ± 1:11 h (n = 57 adults, range 02:22-07:45).
- **Sogawa et al. 2023**, DOI `10.5334/jcr.225`: MSF 4:22 ± 1:36 h (n = 362).
- **Wittmann, Dinich, Merrow & Roenneberg 2006**, *Social jetlag*, Chronobiology International 23, DOI `10.1080/07420520500545979`: the body clock and the social clock are two clocks, and the gap is what a person lives with. The reason `Hour` stays the village's clock.
- **Roenneberg, Pilz, Zerbini & Winnebeck 2019**, Biology 8(3):54, DOI `10.3390/biology8030054`: MSFsc as the definition of chronotype.

## The model

A person's **chronotype** is one number, the hours their body clock runs behind the wall clock (`Chronotype(f32)`, default `0.0`). The hour on their own clock is `(clock_hour - offset) mod 24`, read by `Input::BodyHour` exactly as `Input::Hour` is (`h / 24`, valid with `Hours(from, to)`). The pack's window `Hours(22, 6)` is centred on 02:00, so the offset is a person's mid-sleep minus 02:00: `+2.0` falls asleep at 00:00 and wakes at 08:00, `-3.0` falls asleep at 19:00.

- **`Hour` is not shifted.** It is the social clock (the baker's dawn, the working day). An owl who wakes at 09:00 beside a workshop that opens at 08:00 is Wittmann's social jetlag, and the pack's author decides whether the story needs it.
- **The crate draws nothing.** The host writes a chronotype as it writes a personality: `Chronotype::sample(&mut rng, sd_hours)` (twelve uniform draws summed, Irwin-Hall, so a normal with no transcendental function and a stream that is the same on every platform, truncated at three standard deviations), or `PersonDef::chronotype_hours` in a placement file. A person without one is the person the crate always had, bit for bit: the harness's run with no chronotype and sleepers approachable equals the previous release byte for byte (`summary.json`, `series.csv`, `narrative.txt` at seed 7).
- **`Chronotype::POPULATION_SD_HOURS` = 1.5 h is a design default**, inside the published range: 1.18 h (Reiter), 1.37 h (the interquartile fit to Fischer), 1.6 h (Sogawa), against 2-2.5 h in diary data that carry recall noise. The mean stays 0: the published mean belongs to a country and a latitude, and the pack's window is the village's average night.
- **Sleep length is the pack's 8 h**, because chronotype is independent of sleep duration (Fischer 2017).
- **A sleeper cannot be approached** (`ActivityDef::unavailable`). A person who has *arrived* at an `unavailable` activity is in nobody's `near` list (not a `Person` target, not a `Distance` neighbour), and an activity still walking toward them ends, `completed: false`. Someone still walking to bed is on their way and can be met. The starter pack marks `sleep`. This is `Dormant`'s rule (out of perception, no exchanges) applied per activity.

Not modelled: personality-derived chronotype (the Big Five explain a few per cent of its variance, far too little to produce the spread, and a derived value would hide a free parameter in a trait), age and sex (Fischer 2017 has both; the crate has neither), homeostatic sleep pressure beyond the existing `rest` need, free-running drift, light exposure.

## What it did (same seeds, 17 game-days at 64 s/day; `docs/loop/runs/2026-10-31-02/evidence/chronotype.md`)

| | before | after (`sd` 1.5 h) |
|---|---|---|
| sleeps begun in the busiest clock hour, seeds 7 / 42 / 11 / 99 | 63 / 60 / 64 / 64 % | 19 / 23 / 27 / 21 % |
| exchanges aimed at a sleeper | 349 / 378 / 324 / 362 of ~4 400 (8 %) | 0 / 1 / 2 / 3 |

With body clocks and **no** `unavailable` flag the number of exchanges aimed at a sleeper rises to 646 / 586 / 493 / 670 (15 %): an owl awake at 01:00 stands among sleepers. The flag is what makes body clocks safe to switch on.

What did **not** move, and the claim that was wrong: the interrupted-sleep rate stays 10-18 % either way. The crate's warning blames a queue at the bed; at 64 s/day the cause is the walk home (a 10 m walk is 2.7 game-hours, so a person decides to sleep three hours before they lie down), and body clocks do not shorten walks. `travel-time-in-two-clocks.md` is the item for that.
