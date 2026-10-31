# Eval — body clocks (`Chronotype`, `Input::BodyHour`, `ActivityDef::unavailable`)

Raw series (`series.csv`, 12-60 MB each) are ignored by git and regenerate with the commands below; everything cited here is committed under `evidence/` (`sleep.txt`, `summary.json`, `narrative.txt`, `activities.txt`, `chronotypes.txt` per run, the two plots, the table `chronotype.md`).

## Commands (`cargo build --release --example evidence` first)

`before` is the society as 0.8.0 had it: nobody has an offset and a sleeper can be approached (`--sleepers-available`). `after` is the same seed with a population of offsets (`--chronotype-sd 1.5`, drawn from a stream of its own, `seed ^ 0xC1C0C10C`, so personalities and every other draw are the ones `before` had). Two controls separate the two halves of the change: `no-clocks` (the pack as shipped, no offsets: sleepers cannot be approached) and `sleepers-open` (offsets, sleepers approachable).

```sh
B=./target/release/examples/evidence
for S in 7 42 11 99; do                                       # village, 24 people, 17 game-days at 64 s/day
  $B --scenario village-evening --seed $S --days 17 --sleepers-available                        --out evidence/before/v$S
  $B --scenario village-evening --seed $S --days 17 --chronotype-sd 1.5                         --out evidence/after/v$S
  $B --scenario village-evening --seed $S --days 17                                             --out evidence/variants/no-clocks-v$S
  $B --scenario village-evening --seed $S --days 17 --chronotype-sd 1.5 --sleepers-available    --out evidence/variants/sleepers-open-v$S
done
# towns, 30 and 20 game-days; the 240-second day is the library's own
$B --scenario village-evening --seed 7  --people 60 --days 30 --sleepers-available   --out evidence/before/t7      # and --chronotype-sd 1.5 -> after/t7; the controls as above
$B --scenario village-evening --seed 99 --people 60 --days 30 --sleepers-available   --out evidence/before/t99     # fresh eyes
$B --scenario village-evening --seed 3  --people 40 --days 20 --sleepers-available   --out evidence/before/t40s3   # fresh eyes
$B --scenario village-evening --seed 7  --seconds-per-day 240 --days 6 --sleepers-available --out evidence/before/v7d240
$B --scenario village-evening --seed 7 --days 6 --chronotype-sd 1.5 --track 7  --out /tmp/lark    # narratives/lark-hilda-s7-6days.txt
$B --scenario village-evening --seed 7 --days 6 --chronotype-sd 1.5 --track 16 --out /tmp/owl     # narratives/owl-quill-s7-6days.txt
python3 -m venv /tmp/plotenv && /tmp/plotenv/bin/pip install matplotlib numpy pillow
/tmp/plotenv/bin/python tools/plot_chronotype.py docs/loop/runs/2026-10-31-02 evidence/plots/sleep.png evidence/plots/day.png evidence/chronotype.md
```

## 1. A person with no offset is the person 0.8.0 had, bit for bit

`--sleepers-available` (offset 0 everywhere, the old `sleep` rule on `BodyHour`) against the 0.8.0 binary, village seed 7, 17 game-days: `cmp` finds `summary.json`, `series.csv` (69 632 rows) and `narrative.txt` **identical**. `BodyHour` with no offset is `(h - 0.0).rem_euclid(24.0)`, the same `f32`.

## 2. What the village does (`evidence/chronotype.md`, `evidence/plots/sleep.png`)

| | before | no-clocks (flag only) | after (offsets + flag) | sleepers-open (offsets, no flag) |
|---|---|---|---|---|
| **village 64 s/day, seeds 7 / 42 / 11 / 99: sleeps begun in the busiest clock hour** | 63 / 60 / 64 / 64 % | 64 / 63 / 59 / 63 % | **19 / 23 / 27 / 21 %** | 21 / 21 / 31 / 20 % |
| exchanges aimed at somebody asleep | 349 / 378 / 324 / 362 | 0 / 0 / 0 / 0 | **0 / 1 / 2 / 3** | 646 / 586 / 493 / 670 |
| friend ties (directed) | 41 / 43 / 37 / 42 | 48 / 66 / 51 / 69 | 79 / 60 / 53 / 57 | 54 / 46 / 52 / 52 |
| rival ties | 36 / 36 / 35 / 38 | 48 / 19 / 31 / 24 | 28 / 30 / 27 / 20 | 41 / 38 / 41 / 22 |
| insults | 273 / 290 / 308 / 301 | 301 / 227 / 288 / 252 | 233 / 253 / 247 / 286 | 266 / 283 / 257 / 238 |
| sleeps interrupted (all before arrival, see 3) | 14 / 14 / 15 / 12 % | 18 / 14 / 14 / 16 % | 16 / 12 / 17 / 17 % | 10 / 12 / 14 / 11 % |
| **town 60 s7 / s99, 30 days: busiest bedtime hour** | 49 / 47 % | 48 % | **18 / 17 %** | 17 % |
| town: exchanges aimed at a sleeper | 1 950 / 1 620 of ~22 500 | 141 | **87 / 118** | 3 471 |
| town: interrupted sleeps | 27 / 32 % | 35 % | 32 / 35 % | 29 % |
| **town 40 s3 (fresh eyes), 20 days: busiest bedtime hour** | 48 % | | **20 %** | |
| **village 240 s/day, seed 7, 6 days: busiest bedtime hour** | **75 %** | | **19 %** | |
| 240 s/day: clock hours with 10-90 % of the village asleep | 4 | | **12** | |

- **The bedtime distribution moves where it should, at every population and at both day lengths.** The 24-person village goes from 60-64 % of its sleeps in one clock hour to 19-27 %; the towns 47-49 % → 17-20 %; at the library's own 240-second day a cliff (75 % at 22:00) becomes a ramp.
- **At 64 s/day the *asleep* curve moves less than the bedtimes do**, because a 10 m walk is 2.7 game-hours: a person decides to sleep three hours before lying down, so the walk already smears the curve over four or five hours (red in `sleep.png`'s top row). The bottom row (decisions) and the 240-second column show the change plainly.
- **Sleepers stop being greeted.** 7-9 % of the shipped village's exchanges were aimed at a sleeper; now 0-3 of ~4 000. In the town the residue is 87-141 of ~22 500 (0.4-0.6 %; an exchange that resolves in the tick its target lies down). With offsets and no flag the share doubles (12-17 %; 15 % in the town): the flag is what makes clocks safe.
- **The social outcomes move, and the larger part of the move is the flag, not the clocks.** Friend ties rise in every one of the seven pairs, before → after (41 → 79, 43 → 60, 37 → 53, 42 → 57, 101 → 121, 89 → 111, 69 → 79) and rival ties fall in five of the seven (36 → 28, 36 → 30, 35 → 27, 38 → 20, 127 → 94; 116 → 123 and 50 → 72 go the other way). Flag alone (no-clocks), the four villages go 41 / 43 / 37 / 42 → 48 / 66 / 51 / 69 friends, so removing the 7-9 % of exchanges that landed on a sleeper accounts for a good part of it. The mechanism is not established; every number here is one seed's trajectory of a chaotic society, so the direction (more friends, no more insults) is the finding and the size is not. `friendship_bound`, `label_hysteresis` and `rumor_book` pass unmodified (section 4).

## 3. What did not move, and a claim that was wrong

The first prediction (`research.md`) was that interrupted sleeps would fall, because the crate's own warning blames a queue at the bed. The harness now records the walker's state on the tick before each ending: **every interrupted sleep in six runs ended before the person reached the bed** (63 of 63, 74 of 74; 577 of 577, 707 of 707; 18 of 18, 24 of 24). The walk home outlasts `travel_timeout_hours`; body clocks do not shorten a walk. The rate is flat in the village (10-18 % in every arm) and **a few points worse in two of three towns** (27 → 32 %, 32 → 35 %, 24 → 23 %); not explained. It is `travel-time-in-two-clocks.md`'s item, not this one's. The prediction is struck in `research.md`.

## 4. Gate (release, all features unless noted; 99 test binaries plus the library)

- `cargo test --release --all-features --no-fail-fast`, in two batches (50 and 49 binaries + `--lib`) **run twice, on the first implementation and again on the final code (`848b9df`)**: **all ok, 0 failures both times**, including `friendship_bound`, `label_hysteresis`, `rumor_book`, `starting_larder`, `need_cycle`, `village_evening`, `determinism`, `save`, `readme`, `guide_blocks`, `leaf` and the new `tests/chronotype.rs` (9 tests: the arithmetic, the sampler, bedtimes by each person's own clock, the same seed the same bedtimes, a sleeper is not a target and the flag-off control is, a walk toward someone who lies down ends interrupted, the `Hours` refusal, placement in and out, save in and out with an old file). `cargo test --release --all-features --doc`: ok. No assertion of an existing test was edited.
- `cargo test --release --no-default-features` (every integration binary, the library and the doctests): **101 `test result: ok`, exit 0**, on the first implementation; the final change is a refactor of one lookup, and its output is byte-identical (section 1's comparison repeated on the final build: `summary.json`, `narrative.txt` and `series.csv` equal for `--sleepers-available`, the default and `--chronotype-sd 1.5` at seed 7).
- `cargo clippy --all-targets --all-features -- -D warnings`: clean. `cargo fmt --check`: clean. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`: clean. `cargo build --no-default-features`: the 5 dead-code warnings `main` has; `cargo build --examples`: clean.
- Perf: `perf.md`.
