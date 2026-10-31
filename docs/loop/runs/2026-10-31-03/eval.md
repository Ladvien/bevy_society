# Eval - `Tuning::rebuff_memory_hours` (a refusal is remembered)

Raw series (`series.csv`, 15-60 MB each) are ignored by git and regenerate with the commands below; everything cited here is committed under `evidence/` (`reapproach.txt`, `partners.txt`, `summary.json` per run, the table `rebuff.md`, the plots, the narratives).

## Commands (`cargo build --release --example evidence` first)

`before` = `--no-rebuff` (`Tuning::rebuff_memory_hours = None`, which is also the shipped default), the previous build byte for byte (section 1). `after` = `--rebuff-memory 12`, the value measured and **not** the default (section 6).

```sh
B=./target/release/examples/evidence
for S in 7 42 11 99; do                                  # village, 24 people, 17 game-days at 64 s/day
  $B --scenario village-evening --seed $S --days 17 --no-rebuff        --out evidence/before/v$S
  $B --scenario village-evening --seed $S --days 17 --rebuff-memory 12 --out evidence/after/v$S
done
for S in 7 99; do                                        # towns of 60, 30 game-days
  $B --scenario village-evening --seed $S --people 60 --days 30 --no-rebuff        --out evidence/before/t$S
  $B --scenario village-evening --seed $S --people 60 --days 30 --rebuff-memory 12 --out evidence/after/t$S
done
$B --scenario village-evening --seed 3 --people 40 --days 20 --no-rebuff        --out evidence/before/t40s3   # fresh eyes
$B --scenario village-evening --seed 3 --people 40 --days 20 --rebuff-memory 12 --out evidence/after/t40s3
for H in 3 6 12 24; do $B --scenario village-evening --seed 7 --days 17 --rebuff-memory $H --out evidence/sweep/h$H; done
# the narratives: the owl of pass 2026-10-31-02 and a fresh person at another seed
$B --scenario village-evening --seed 7  --days 6 --chronotype-sd 1.5 --track 16 --no-rebuff        --out /tmp/owl-before   # narratives/owl-quill-s7-6days-before.txt
$B --scenario village-evening --seed 7  --days 6 --chronotype-sd 1.5 --track 16 --rebuff-memory 12 --out /tmp/owl-after
$B --scenario village-evening --seed 42 --days 6 --track 3 --no-rebuff        --out /tmp/p3-before                        # narratives/person3-s42-6days-before.txt
$B --scenario village-evening --seed 42 --days 6 --track 3 --rebuff-memory 12 --out /tmp/p3-after
python3 -m venv /tmp/plotenv && /tmp/plotenv/bin/pip install matplotlib numpy pillow
/tmp/plotenv/bin/python tools/plot_rebuff.py docs/loop/runs/2026-10-31-03 evidence/plots/rebuff.png evidence/plots/sweep.png evidence/rebuff.md evidence/plots/over-time.png   # from the run directory's parent
```

## 1. Off is the previous build, byte for byte

`--no-rebuff` against the baseline binary (HEAD before this change, `/tmp/bs71/evidence-base`), village seed 7, 17 game-days: `cmp` finds `summary.json`, `narrative.txt` and `series.csv` (69 632 rows) **identical**. The memory is read and written only when `rebuff_memory_hours` is `Some`, and its filter runs after `targets_for`, so the RNG draws are the same.

## 2. The re-approach table (`evidence/rebuff.md`, `evidence/plots/rebuff.png`)

| | before | after |
|---|---|---|
| refused approach followed within 1 h by another to the same person, not an insult: villages s7 / s42 / s11 / s99 | 29.0 / 24.2 / 26.8 / 24.9 % | **0.3 / 1.0 / 0.8 / 1.1 %** |
| towns of 60 (s7, s99), of 40 (s3) | 27.4 / 25.2 / 24.3 % | **0.7 / 0.8 / 0.6 %** |
| ...within 24 h, all seven | 39.4-49.6 % | 12.6-20.2 % |
| followed an *accepted* approach, within 24 h | 60.5-67.9 % | 59.9-68.2 % (unchanged) |
| different people approached per person-day, villages / towns | 4.10-4.39 / 4.67-5.07 | **5.16-5.43 / 5.38-6.29** |
| exchanges with the initiator's previous partner | 30.5-34.4 % | 21.5-25.3 % |
| refused share of all approaches | 35.2-40.1 % | 34.5-40.1 % (unchanged) |
| approaches per person-day, villages / towns | 9.29-10.37 / 9.73-12.21 | 9.16-9.44 / 8.86-10.86 |
| friend ties (directed), seven runs | 48 66 51 69 128 106 103 | 86 69 61 70 168 140 107 |
| rival ties | 48 19 31 24 108 128 78 | 30 22 36 33 123 94 59 |
| insults | 301 227 288 252 1442 1425 527 | 229 262 247 267 1309 1302 503 |

The targeted behaviour is gone in every run: after a refusal the person turns elsewhere and the chance of going back within the hour is under 1.2 % (it was a quarter of refusals). Beyond the memory it comes back (13-20 % within a day, against 60-68 % after an accepted approach), which is what 12 hours means.

**Struck: "the refused share falls".** It does not (35.2 → 36.6 % at seed 7, flat everywhere). The acceptance draw reads the target's side of the tie, mood and personality, not who is being asked again; the memory changes whom a person asks and not how often they are refused. `docs/research/rebuff.md` says so.

**Honest costs.** Approaches per person-day fall in six of seven pairs (by 4.5-11 %; the village at seed 42 rises 2 %); the run's mean `social` need is lower by 0.007-0.025 in five of seven pairs (`evidence/rebuff.md`, last column) and `fun` by 0.005-0.031 in five; people with few neighbours run out of people to ask for twelve hours and go to work, to sleep or to wander. The social outcomes (friends up in seven of seven, insults down in five, rivals and mood mixed) are one chaotic trajectory per seed, so only the direction of friends is reported, and not as a mechanism.

## 3. The sweep (`evidence/plots/sweep.png`, village seed 7)

| hours | off | 3 | 6 | 12 | 24 |
|---|---|---|---|---|---|
| refused approach followed within 1 h / 24 h | 29.0 / 49.6 % | 0.9 / 28.3 | 1.3 / 24.9 | 0.3 / 18.3 | 0.4 / **1.8** |
| different people per person-day | 4.29 | 5.00 | 5.17 | 5.16 | 5.10 |
| approaches per person-day | 10.37 | 9.73 | 9.64 | 9.28 | 8.83 |
| friend / rival ties | 48 / 48 | 89 / 29 | 58 / 33 | 86 / 30 | 81 / 22 |
| insults | 301 | 230 | 271 | 229 | 216 |

Any memory removes the one-hour re-approach; the hours set how long the person is held out. 24 h leaves **no** retry within the day (1.8 %), which makes every refusal a standing one and costs the most approaches (8.83 a day); 3 h returns 28 % of them within the day. 12 h: the person refused at midday is a candidate again tomorrow morning, 18 % come back within the day. A design default; no paper gives the number.

## 4. The narratives

`narratives/owl-quill-s7-6days-{before,after}.txt` (`--track 16`, the person who asked Garet for gossip seven times in one evening): before, `gossip with Garet` at 20:38 (refused), 20:56 (accepted), 21:27, 22:43, 23:01, 23:19, 23:37 (refused); after, none (Garet appears once, as the one who approached the owl at 19:19 on day 3). Starts of gossip 37 → 15 and of compliment 36 → 22, over six days, and the lines for the evening read as a person who asked once, was turned down, and did something else. `narratives/person3-s42-6days-{before,after}.txt` (a fresh seed and person): 6 of 20 refused approaches were re-asked within 12 h before, 1 of 18 after, and that one is a *compliment refused by Wren at 19:14 followed by an insult to Wren at 19:26*: hostility after a rejection is the case the memory deliberately leaves to `retaliation_refractory_hours` (Bondü & Richter 2016).

## 5. Gate (default `None`; release, all features unless noted)

At the final code (`1217c11`, after `cargo fmt`), on a host at load 15-48 with other jobs running:

- `cargo fmt --check`: clean. `cargo clippy --all-targets --all-features -- -D warnings`: clean. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`: clean. `cargo build --no-default-features`: the 5 dead-code warnings `main` has (dialogue-only items). `cargo build --examples`: clean.
- `cargo test --release --all-features --no-fail-fast`, two batches (50 test binaries, then `--lib` and the other 50): **101 `test result: ok`, 0 failures** (432 tests) and `--doc` (3 doc tests) ok, including `label_hysteresis`, `friendship_bound`, `rumor_book`, `starting_larder`, `need_cycle`, `village_evening`, `determinism`, `save`, `save_upgrade`, `readme`, `guide_blocks`, `leaf`, `tuning_covers_rates` and the new `tests/rebuff.rs` (7: a refused person is asked again and again with the memory off; one approach until the memory runs out; the memory ends and the person is asked again; a person turned down turns to someone new; an accepted approach is not remembered; a refused insult is not a rebuff; a save mid-memory restores the same list).
- `cargo test --release --no-default-features --no-fail-fast` (every integration binary, the library and the doctests): **102 `test result: ok`, 392 tests, 0 failures, exit 0**.

The same suite ran once before the default was set to `None`, with `Some(12.0)`: **every binary passed but `label_hysteresis`** (100 of 101), which is the check in section 6.

## 6. The protected check, and the flip check

`tests/label_hysteresis.rs::the_band_stops_the_low_end_flapping_and_moves_no_relationship` asserts `demotions_with / demotions_without <= 0.05` for the label band on the village fixture. With `Some(12.0)` it read **16 with the band against 311 without, 5.14 %**. One probe at the fixture's own seed and clock (a throwaway test, removed) varied only the memory:

| memory | off | 3 h | 6 h | 9 h | 12 h | 18 h | 24 h |
|---|---|---|---|---|---|---|---|
| demotions with / without a band | 12 / 317 | 11 / 285 | 15 / 269 | 16 / 269 | 16 / 311 | 9 / 319 | 11 / 299 |
| share | 3.8 % | 3.9 % | 5.6 % | 6.0 % | 5.1 % | 2.8 % | 3.7 % |
| messages with / without | 1 639 / 2 249 | 1 629 / 2 177 | 1 684 / 2 192 | 1 678 / 2 184 | 1 678 / 2 268 | 1 740 / 2 360 | 1 772 / 2 348 |
| friend / rival pairs, with = without | 19 / 44 | 27 / 30 | 36 / 37 | 32 / 44 | 26 / 32 | 36 / 30 | 33 / 31 |

The other two assertions of the test (fewer messages with the band, equal friend and rival counts) hold at every memory. The with-band count is 9-16 events (mean about 13, standard deviation about 3.6): the bar is 15.5 events at 311 and the memory only changes which trajectory the village takes. **A protected check is not loosened, so the memory ships off** (`docs/loop/needs-decision/a-refusal-is-remembered.md`).

**Flip check.** `rebuff-default.patch` (the default `Some(12.0)` and the bar at `0.08`) applied in a scratch worktree (`/home/ladvien/.cache/loop-runs/bs71-flip`, mine, `git apply` clean) against `e992429` (`1217c11` differs from it by `cargo fmt` and the benchmark switch only): `cargo test --release --all-features --test label_hysteresis --test rebuff`: **9 passed and 7 passed**. Together with the earlier run of the whole suite at `Some(12.0)` (every other binary passed), the owner's flip is a two-line change and nothing else in the suite moves.
