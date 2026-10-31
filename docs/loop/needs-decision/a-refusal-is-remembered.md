# A refusal is remembered: `Tuning::rebuff_memory_hours`, and whether it is on

> **Assumed decision (owner may override), 2026-10-31-03: ship `rebuff_memory_hours` with the default `None` (off) and the measured value `Some(12.0)` documented, because `Some(12.0)` trips one protected check by a single event (option (c)).** Researched and measured in `docs/loop/runs/2026-10-31-03/`.

## The state

A person who is refused chooses the same person again at the next decision: on the 24-person village 29 % of refused approaches were followed within the hour by another approach to the same person (24-29 % in four villages, three towns). `rebuff_memory_hours: Some(h)` keeps a person from choosing someone who has just refused them, for `h` game-hours, as the target of any affiliative `Person` activity (Maner et al. 2007, DOI 10.1037/0022-3514.92.1.42). At 12 hours the share falls to 0.3-1.1 % in all seven runs and a person approaches 5.2-5.4 different people a day in the villages (4.1-4.4 before), 5.4-6.3 in the towns (4.7-5.1). `evidence/rebuff.md`.

## The check it trips

`tests/label_hysteresis.rs::the_band_stops_the_low_end_flapping_and_moves_no_relationship` asserts `demotions_with / demotions_without <= 0.05` on the village fixture (seed 7, 6 400 ticks). With the memory at 12 hours it reads **16 with the band against 311 without, 5.14 %**, and fails. The same fixture at other memories (one probe, the fixture's own seed and clock, only the memory varied; the harness's other three assertions in that test, fewer messages with the band and equal friend and rival counts, hold at every value):

| memory | off | 3 h | 6 h | 9 h | 12 h | 18 h | 24 h |
|---|---|---|---|---|---|---|---|
| demotions with / without a band | 12 / 317 | 11 / 285 | 15 / 269 | 16 / 269 | 16 / 311 | 9 / 319 | 11 / 299 |
| share | 3.8 % | 3.9 % | **5.6 %** | **6.0 %** | **5.1 %** | 2.8 % | 3.7 % |

The with-band count is 9-16 events, so the share is a Poisson count at the edge of its bar (the mean is about 13, a standard deviation 3.6, and the bar is 15.5 events at 311): the memory changes which trajectory the village takes and the test reads one trajectory. It is not a finding that the band works less well with the memory. The loop's rule is that a protected check is never loosened, so the pack does not switch the memory on.

## Options

- **(a) Default `Some(12.0)` and relax the bar to `0.08`.** The seven values above run 2.8-6.0 %, and 8 % is 25 events at 311 (3.3 standard deviations above the mean count). `rebuff-default.patch` in the run directory does exactly this (one line in `src/core.rs`, one number in the test); it applies cleanly and the patched fixture passes (`eval.md`, section 6). Then CHANGELOG and README say the memory is on by default (`Changed`, one sentence each) and the 24-person village's friend ties and insults move as `evidence/rebuff.md` shows.
- **(b) Default `Some(24.0)` or `Some(18.0)`, which pass the bar as it stands.** Chosen only for passing: 24 hours leaves 1.8 % of refused approaches retried within the day (a refusal becomes a standing one) and costs the most approaches (10.4 → 8.8 a person-day at seed 7); 18 hours was not run through the evidence harness. Rejected: it picks the value by a noisy pin and not by what the sources and the sweep say.
- **(c) Ship opt-in, `None`** (picked). Reversible, touches no protected check, and a host that wants it sets one field. A run with the default is the previous build byte for byte (`cmp` on `summary.json`, `narrative.txt`, `series.csv`).

## What deferring costs

Nothing is lost; the shipped village keeps asking a refuser again within the hour 29 % of the time until the owner applies (a) or a host sets the field. To switch it on: `git apply docs/loop/runs/2026-10-31-03/rebuff-default.patch`, then change the field's doc paragraph "The default is `None`", the `concepts.md` paragraph heading "(opt-in)", and the CHANGELOG and README sentences from "opt-in" to "on by default".
