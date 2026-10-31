# Eval: a starting larder (`GoodDef::initial`, starter pack bread `1.0`)

Regenerate (release, from the repository root; `series.csv` is ignored, `summary.json`, `narrative.txt`,
`floor.md`, `summary.md` and the plot are committed under `evidence/`):

```sh
cargo build --release --example evidence
# before = this branch's parent (48e2260, the same harness plus the floor columns) with bread `initial` 0
# after  = this branch
./target/release/examples/evidence --scenario village-evening --seed S --days 17 --out DIR            # S in 7 42 11 99
./target/release/examples/evidence --scenario village-evening --seed S --people 60 --days 30 --out DIR  # town, S in 7 99
python3 tools/plot_larder.py docs/loop/runs/2026-10-30-27 evidence/plots/larder.png evidence/floor.md
```

(`/tmp/plotenv27/bin/python` is a venv with matplotlib; any Python 3 with matplotlib does.) The baseline reproduced
the previous pass's `after` numbers exactly (village s7: 28 rival ties, 62 friend ties, 251 insults), so "before" is
HEAD and the two sides differ only by the larder. Village is 24 people on the evidence harness's 64-second game-day
(4 096 ticks); the test helper `tests/common/village.rs` uses the library default 240-second day (960 updates), which
is where the 38.7 % figure comes from (`tests/starting_larder.rs` prints it).

## The metric: villagers at the floor on `food`

`series.csv` has `floor2` per tick: villagers whose `food` is below 0.05. `evidence/floor.md` is the per-game-day share
of villager-ticks (`evidence/plots/larder.png` is the same data; red before, blue after).

| run | side | day 1 | day 2 | day 3 | day 4 | day 5 | day 6 | mean of day 7 on |
|---|---|---|---|---|---|---|---|---|
| village s7 | before | 0.0 | 78.6 | 72.9 | 48.9 | 26.3 | 28.3 | 19.5 |
| village s7 | after | 0.0 | 12.6 | 20.4 | 17.6 | 18.7 | 28.9 | 19.7 |
| village s42 | before | 0.0 | 88.9 | 72.7 | 52.9 | 41.8 | 33.9 | 23.1 |
| village s42 | after | 0.0 | 15.6 | 23.2 | 19.5 | 17.3 | 34.9 | 24.2 |
| village s11 | before | 0.0 | 89.6 | 83.1 | 62.3 | 40.7 | 35.7 | 35.0 |
| village s11 | after | 0.0 | 9.3 | 13.6 | 22.7 | 25.5 | 14.3 | 16.7 |
| village s99 (fresh eyes) | before | 0.0 | 90.0 | 89.2 | 88.1 | 79.2 | 71.4 | 23.2 |
| village s99 (fresh eyes) | after | 0.0 | 14.7 | 13.8 | 14.5 | 18.6 | 9.4 | 22.9 |
| town 60 s7 | before | 0.0 | 90.8 | 93.6 | 80.7 | 71.3 | 68.4 | 94.1 |
| town 60 s7 | after | 0.0 | 36.0 | 44.1 | 43.9 | 48.3 | 53.6 | 92.0 |
| town 60 s99 (fresh eyes) | before | 0.0 | 83.3 | 82.1 | 84.1 | 85.6 | 88.8 | 92.6 |
| town 60 s99 (fresh eyes) | after | 0.0 | 38.1 | 38.5 | 32.7 | 35.6 | 35.2 | 91.4 |

- **The village's start-up famine is gone at all four seeds.** Day 2 falls from 79-90 % of villager-ticks at the floor to
  9-16 %, and the village is at its settled share (14-35 %) from day 2 rather than from day 5-7. The settled share
  itself (the last column) is unchanged at three seeds (19.5 / 23.1 / 23.2 → 19.7 / 24.2 / 22.9) and lower at one
  (35.0 → 16.7). It is not a larder effect: it is where the dawn-bake economy of the 64-second day sits.
- **The town does not recover and the larder does not claim it.** Day 2 improves (91 / 83 → 36 / 38 %), then both sides
  decline to 100 % of villager-ticks at the floor with no bread anywhere (`bread_mean` 0.000 from about game-day 17,
  price 2.000 = the ceiling). The larder only delays it: a loaf a head is eaten down over about ten days and nothing
  replaces it. That is a separate defect, diagnosed below and logged as the top backlog item.
- Test village (240-second day, `tests/starting_larder.rs`, seed 99): game-day 2 floor **38.7 % → 4.7 %**.

## What else moved (`evidence/summary.md`)

| run | side | rival ties | friend ties | insults | accepted compliments | shares |
|---|---|---|---|---|---|---|
| village s7 | before → after | 28 → 36 | 62 → 41 | 251 → 273 | 1211 → 1284 | 200 → 166 |
| village s42 | before → after | 24 → 36 | 65 → 43 | 240 → 290 | 1251 → 1372 | 100 → 138 |
| village s11 | before → after | 18 → 35 | 47 → 37 | 255 → 308 | 1181 → 1262 | 104 → 159 |
| village s99 (fresh eyes) | before → after | 36 → 38 | 54 → 42 | 249 → 301 | 1195 → 1246 | 119 → 163 |
| town 60 s7 | before → after | 111 → 127 | 101 → 101 | 1401 → 1459 | 6087 → 6413 | 28 → 16 |
| town 60 s99 (fresh eyes) | before → after | 155 → 116 | 86 → 89 | 1491 → 1417 | 5770 → 6171 | 35 → 43 |

At day 17 the village has **more rival ties and fewer friend ties at every one of four seeds** (rivals +8 / +12 / +17
/ +2, friends −21 / −22 / −10 / −12) and 10-20 % more insults. The town is mixed. The 17-day count is a cumulative
history, and the fed village starts conversing a day or four earlier than the starved one (rival ties at the end of day
4: s11 3 → 24, s42 7 → 24); the long-run check below asks whether the difference persists.

LONGRUN

## The town's famine (diagnosis only; not changed here)

From `series.csv`'s `doing` column (person-hours per game-day), town of 60, seed 7, before:

| day | bakes | meals | person-hours walking to eat |
|---|---|---|---|
| 1 | 1.0 | 17.6 | 65 |
| 2 | 2.0 | 90.6 | 187 |
| 5 | 0.0 | 90.7 | 191 |
| 12 | 0.0 | 103.1 | 160 |

The town holds 60 people in four places (`eat` and `bake` both target `Place("food")`: the tavern, capacity 10, and the
market, capacity 8). Hungry people choose `eat` all day (its score depends on `food` only, so an empty larder does not
lower it): 80-100 meals a day, none of which restores anything once the bread is gone, and 12-13 % of every person's
day spent walking to a table that holds nothing. The dawn `bake` is one hour long (`Hours(6, 7)`), it needs a slot at
the same two places, and a hungry baker scores `eat` higher than `bake`. The pack's own comment names the first half
("the supply does not scale with the population"); the second half, that eating from an empty larder crowds out the
activity that would fill it, is new here and is the next pass's item. The literature distinction is appetitive
(seeking, producing) against consummatory (eating) behaviour: hunger with no food present should drive the first, not
the second (Berridge 2004, DOI 10.1016/j.physbeh.2004.02.004, title and DOI checked against Crossref, abstract not
yet read; Torrealba et al. 2012, DOI 10.3389/fnsys.2012.00051, on the separate appetitive and consummatory phases of
motivated behaviour: the next pass reads both before it builds anything).

## Gate

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps --all-features`: clean.
- `cargo test --release --all-features --no-fail-fast`: 97 binaries, 0 failing. `cargo test --release --no-default-features
  --no-fail-fast`: 97 binaries, 0 failing; `cargo build --no-default-features` builds with the 5 warnings `main` has.
- New: `tests/starting_larder.rs` (6), `tests/save_upgrade.rs::a_version_one_save_holds_nothing_a_tick_after_it_restores`
  (fails on the parent's `save.rs`: left `1.0`, right `0.0`, passes with the fix).
- Protected checks: none edited. `rumor_book`, `friendship_bound`, `label_hysteresis`, `need_cycle`, `trade`, `economy`
  all pass unmodified.
