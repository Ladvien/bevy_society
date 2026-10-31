# Eval: `ActivityDef::urgent` and hunger that takes precedence (provisioning overlay)

Regenerate (release, from the repository root; `series.csv` is ignored by git, `summary.json`, `floor.txt`, `activities.txt`,
`narrative.txt`, `urgent.md` and the plots are committed under `evidence/`):

```sh
cargo build --release --example evidence
# before = the provisioning overlay with `--no-urgent`; after = the overlay as shipped (`eat` and `bake` urgent below 0.25)
./target/release/examples/evidence --scenario village-evening --provisioning --no-urgent --seed S --days 17 --out DIR   # S in 7 42 11 99, then without --no-urgent
./target/release/examples/evidence --scenario village-evening --provisioning [--no-urgent] --seed S --people 60 --days 30 --out DIR   # S in 7 99
./target/release/examples/evidence --scenario village-evening --provisioning [--no-urgent] --seed 3 --people 40 --days 20 --out DIR  # fresh eyes
./target/release/examples/evidence --scenario village-evening --provisioning [--no-urgent] --seed 7 --people 60 --seconds-per-day 240 --days 12 --out DIR
# the sweep (evidence/sweep/): `--urgent 0.15|0.25|0.35` on the default pack and on `--provisioning`
/tmp/plotenv29/bin/python tools/plot_urgent.py docs/loop/runs/2026-10-30-29 evidence/plots/urgent.png evidence/urgent.md
/tmp/plotenv29/bin/python tools/plot_larder.py docs/loop/runs/2026-10-30-29 /tmp/larder.png /tmp/larder.md evidence/plots/zoom-first-days.png
```

(`/tmp/plotenv29` is a venv with matplotlib and Pillow.) Village is 24 people on the harness's 64-second game day (4 096 ticks);
the town is 60 on the same layout (the tavern and the market are homes), the 40-person town and the 240-second day are the
fresh-eyes runs. Every pair is the same seed, same binary, one flag apart.

## What was measured first (HEAD, no change): `evidence/sweep/village-s7-default/floor.txt`

`floor.txt` is new in the harness. Village, seed 7, default pack, 17 game-days, from game-day 2:

- 19.4 % of villager-ticks at `food` < 0.05; **96.8 % of those ticks the household held a meal** (a quarter loaf or more).
- What they were doing: asleep 25.8 % (+ 4.7 % walking to bed), walking to eat 19.6 %, at the table 0.6 %, **`work` 17.2 % (+ 6.5 %
  walking to it), `craft` 4.6 % (+ 1.8 %), company and leisure 19.2 %**: 49 % choosing something that is not the meal on the shelf.
- Spells of hunger (`food` under 0.35 until back over): 316, mean 12.4 h, median 9.5 h, p90 27.5 h, longest 86 h.

## The sweep (village, seed 7, 17 game-days; `evidence/sweep/`)

| pack | `urgent.below` | floor | spell mean / median / p90 (h) | bread a head at the end | friends / rivals / insults |
|---|---|---|---|---|---|
| default | off | 19.4 % | 12.4 / 9.5 / 27.5 | 3.24 | 41 / 36 / 273 |
| default | 0.15 | 18.1 % | 11.0 / 7.8 / 13.4 | 0.70 | 47 / 28 / 260 |
| default | 0.25 | **26.9 %** | 12.6 / 6.7 / 17.3 | 0.29 | 76 / 17 / 185 |
| default | 0.35 | **52.2 %** | 14.2 / 6.2 / 13.9 | 0.00 | 66 / 2 / 149 |
| overlay | off | 18.8 % | 11.8 / 9.0 / 25.1 | 1.20 | 50 / 30 / 262 |
| overlay | 0.15 | 3.6 % | 7.4 / 7.1 / 11.4 | 1.65 | 40 / 40 / 329 |
| overlay | 0.25 | 2.7 % | 6.8 / 6.4 / 10.5 | 1.41 | 37 / 39 / 338 |
| overlay | 0.35 | 1.8 % | 6.0 / 6.0 / 9.5 | 1.60 | 47 / 33 / 335 |

On the default pack the more prepotent `eat` is, the worse the floor: its `eat` prices an empty shelf instead of reading it, its
`bake` is a dawn errand outside the bucket, and a hungry person on a bare shelf keeps choosing a meal that feeds nothing.
`0.25` was picked (design default): within the overlay the floor is 2.7 % against 1.8 % at 0.35, and the lower line leaves what a person
does while merely peckish alone (0.35 is the midpoint of `eat`'s own curve, so below it people are already inclined to eat).
`0.15` also helps but leaves the longest spell at 23 h with a thinner margin on the floor at a long walk.

## Before and after (`evidence/urgent.md`, `evidence/plots/urgent.png`)

The overlay with and without `urgent`, same seed. Floor is from game-day 2 (all) and the last six game-days; spells are `food` under
0.35 until back over; "held a meal" is the share of the floor ticks with a quarter loaf in hand.

| run | side | floor % | last 6 days | spells n / mean / median / p90 / max (h) | bread a head | friends / rivals | insults | shares |
|---|---|---|---|---|---|---|---|---|
| village s7 | before | 18.8 | 15.8 | 320 / 11.8 / 9.0 / 25.1 / 54.9 | 1.48 | 50 / 30 | 262 | 181 |
| village s7 | after | **2.7** | 2.9 | 383 / 6.8 / 6.4 / 10.5 / 23.8 | 1.60 | 37 / 39 | 338 | 233 |
| village s42 | before | 19.9 | 17.9 | 310 / 12.1 / 8.3 / 29.2 / 76.0 | 1.48 | 58 / 42 | 311 | 148 |
| village s42 | after | **2.6** | 2.2 | 378 / 6.7 / 6.5 / 10.2 / 30.4 | 1.52 | 56 / 46 | 303 | 221 |
| village s11 | before | 15.5 | 16.7 | 317 / 11.1 / 8.8 / 23.0 / 40.3 | 1.41 | 48 / 47 | 286 | 228 |
| village s11 | after | **3.3** | 2.8 | 384 / 7.0 / 6.6 / 11.0 / 24.2 | 1.65 | 46 / 44 | 312 | 219 |
| village s99 (fresh eyes) | before | 16.2 | 17.4 | 323 / 11.1 / 8.0 / 24.2 / 53.2 | 1.40 | 75 / 37 | 249 | 216 |
| village s99 (fresh eyes) | after | **2.6** | 2.7 | 387 / 6.9 / 6.7 / 10.1 / 24.7 | 1.61 | 47 / 30 | 298 | 200 |
| town 60 s7 | before | 36.1 | 37.8 | 1167 / 19.6 / 12.1 / 43.0 / 212.3 | 1.16 | 109 / 119 | 1386 | 611 |
| town 60 s7 | after | **7.9** | 5.7 | 1694 / 8.3 / 7.2 / 14.7 / 41.7 | 1.45 | 94 / 130 | 1356 | 823 |
| town 60 s99 (fresh eyes) | before | 37.0 | 37.3 | 1157 / 19.8 / 12.2 / 47.3 / 195.5 | 1.10 | 103 / 167 | 1363 | 631 |
| town 60 s99 (fresh eyes) | after | **8.7** | 9.0 | 1673 / 8.5 / 7.1 / 15.4 / 52.3 | 1.54 | 100 / 70 | 1303 | 787 |
| town 40 s3 (fresh eyes) | before | 22.8 | 22.6 | 607 / 13.2 / 9.6 / 30.2 / 84.5 | 1.29 | 70 / 54 | 521 | 429 |
| town 40 s3 (fresh eyes) | after | **5.4** | 4.6 | 745 / 7.8 / 7.2 / 12.8 / 29.1 | 1.50 | 71 / 78 | 582 | 354 |
| town 60 s7, 240 s day | before | 25.9 | 23.3 | 517 / 13.9 / 8.9 / 32.9 / 99.6 | 1.40 | 103 / 243 | 758 | 350 |
| town 60 s7, 240 s day | after | **5.4** | 5.0 | 663 / 7.2 / 6.5 / 13.2 / 30.0 | 1.51 | 82 / 170 | 669 | 422 |

- **The floor falls by 4-7x at every size and both clocks**, and the spell of hunger shortens at the tail more than in the middle (p90
  23-47 h → 10-15 h; the longest 40-212 h → 24-52 h). The mean spell is 6-8 h, not 0: a spell begins at 0.35 and a person finishes
  what it is doing, walks to the table (at the harness's 64-second day a walk is two to four game-hours) and eats.
- **What is left of the floor is the walk and the night** (`after/*/floor.txt`): village s7, 60 % walking to the table, 26 % asleep,
  12 % bread-making and the table; town s7, 71 % walking to the table, 23 % asleep or walking to bed. Work, craft and company: 0.4-0.6 %.
- **The shelf is no emptier**: bread a head 1.1-1.5 → 1.45-1.65 over the last six days, because people no longer wait at 0.05 and the
  provisioning loop is hungry-driven. More meals are eaten (`eat` starts: village 425/401/419/409 → 469/485/475/476; town 1 847/1 833 →
  2 499/2 555) and the share that completes holds (village 87 → 87 %, town 65 → 68 %; interrupted walks stay 11-14 % in the village and
  30-34 % in the town, the harness clock's travel timeout, `needs-decision/travel-time-in-two-clocks.md`).
- **The social fabric moves inside its seed-to-seed range, and one direction is worth watching.** Village friends over four seeds 58 → 47
  (50 / 58 / 48 / 75 → 37 / 56 / 46 / 47; the seeds' own standard deviation is 12 before), rivals 39 → 40, insults 277 → 313 (+13 %),
  shares 193 → 218. Town rivals go both ways (119 → 130, 167 → 70, 243 → 170 at the 240-second day, 54 → 78 in the 40-person town).
  The village does more with its day: accepted approaches 2 588 / 2 808 / 2 797 / 2 702 → 3 093 / 3 013 / 2 999 / 2 750 (+2 to +19 %),
  and with them more churn, rivalries formed 78 / 93 / 83 / 74 → 100 / 103 / 97 / 85 and left 48 / 51 / 36 / 37 → 61 / 57 / 53 / 55; the
  60-person towns do 3-4 % fewer (13 554 / 13 168 → 13 152 / 12 717) with 8-10 % fewer rivalries formed. Neither direction is large
  against the seed-to-seed range and no protected pin moves (below).
- **The first day.** `evidence/plots/zoom-first-days.png`: every run, before and after, still jumps from 0 to a spike of villagers at the
  floor at hour 24 (everyone starts at `food` 1.0 and falls at the same rate: pass 27's lockstep, not touched here: 8-12 before, 2-7
  after), but with `urgent` the spike is gone within a game-hour or two where it persisted at 3-7 for a day.

## The default pack is untouched

`rank_for` returns `spec.rank` when `urgent` is `None`, and the starter pack names no `urgent`. A village run of the starter pack at
seed 7, 17 game-days, on the pre-change binary (`evidence/sweep/village-s7-default`) and on this branch's gives byte-identical
`series.csv`, `summary.json`, `narrative.txt`, `activities.txt` and `floor.txt` (`cmp`, checked).

## Gate

Run on this branch, rustc 1.98.1, on a host at load 35-50 (the all-features integration run took 1 180 s and the no-default-features one
1 494 s, against the roughly 1 500 s cap on one async command):

- `cargo fmt --check`: clean. `cargo clippy --all-targets --all-features -- -D warnings`: clean. `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps --all-features`: clean. `cargo build --examples`: ok. `cargo build --no-default-features`: ok, with the same 5 dead-code warnings
  as HEAD (the dialogue-gated `push_script_actions`, `ConversationSpec`, `NodeEffectSpec`, `TriggerEvent::Dialogue`, `ScriptAction::Converse`).
- `cargo test --release --all-features --tests`: 99 binaries (the library's unit tests and 98 integration binaries), **0 failed**, and
  `--doc` ok. `cargo test --release --no-default-features --tests`: 99 binaries, **0 failed**.
- The protected pins that depend on the starter pack are unchanged because the starter pack is: `rumor_book`, `friendship_bound`,
  `label_hysteresis`, `need_cycle`, `starting_larder`, `economy`, `trade`, `save`, `leaf`, `readme`, `guide_blocks`, all ok. The one edit to an
  existing test is additive: the five `ScoredOption` literals in `src/trace.rs`'s unit tests gain `rank: 0` for the new field
  (`assumed owner exception`, in the commit message); no assertion is removed, loosened or re-pinned.
- New: `tests/urgent.rs` (7, both feature sets) and one in `src/trace.rs`; none of them compiles on `6e2947e` (no `urgent`, no `rank`), so
  "fails before" is a compile error, and the control arms inside each test (a pack without the line, a draw that includes `play`) show the line
  is what changes the choice.
- Perf: `perf.md` (no measurable cost; the default path is bit-identical).
