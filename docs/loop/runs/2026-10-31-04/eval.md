# Eval - a week (`Input::Week`, `Curve::Days`, `SocietySettings::start_weekday`, `ctx.weekday()`)

Raw series (`series.csv`, 15-60 MB each) are ignored by git and regenerate with the commands below; everything cited here is committed under `evidence/` (`week.txt` and `summary.json` per run, the tables `week.md` and `week-fresh.md`, the plots, the narratives, the overlay drafts).

## Commands (`cargo build --release --example evidence` first)

`before` = the default (no `--weekly`), `after` = `--weekly` (`examples/assets/weekly.society.ron`, the final overlay).

```sh
B=./target/release/examples/evidence
for S in 7 42 11 99; do                                  # villages of 24, 29 game-days at 64 s/day (full days 1-28: four of each weekday)
  $B --scenario village-evening --seed $S --days 29            --out evidence/before/v$S
  $B --scenario village-evening --seed $S --days 29 --weekly   --out evidence/after/v$S
done
for S in 7 99; do                                        # towns of 60
  $B --scenario village-evening --seed $S --people 60 --days 29            --out evidence/before/t$S
  $B --scenario village-evening --seed $S --people 60 --days 29 --weekly   --out evidence/after/t$S
done
$B --scenario village-evening --seed 3 --people 40 --days 29            --out evidence/before/t40s3
$B --scenario village-evening --seed 3 --people 40 --days 29 --weekly   --out evidence/after/t40s3
# fresh eyes: other seeds, another size, a calendar that starts on weekday 3
$B --scenario village-evening --seed 123 --days 29 --start-weekday 3            --out evidence/fresh/before/v123
$B --scenario village-evening --seed 123 --days 29 --start-weekday 3 --weekly   --out evidence/fresh/after/v123
$B --scenario village-evening --seed 5 --people 50 --days 29 --start-weekday 3            --out evidence/fresh/before/t50s5
$B --scenario village-evening --seed 5 --people 50 --days 29 --start-weekday 3 --weekly   --out evidence/fresh/after/t50s5
# the sweep at seed 7 (a draft overlay is read from disk with --weekly-file; each is the final overlay with `weight: W`)
for W in 1 8; do sed "s/weight: 3.0/weight: $W.0/" examples/assets/weekly.society.ron > /tmp/w$W.ron
  $B --scenario village-evening --seed 7 --days 29 --weekly-file /tmp/w$W.ron --out evidence/sweep/flat-w$W; done
python3 -m venv /tmp/plotenv && /tmp/plotenv/bin/pip install matplotlib pillow
cd docs/loop/runs/2026-10-31-04
/tmp/plotenv/bin/python ../../../../tools/plot_weekly.py . evidence/plots/week.png evidence/week.md
/tmp/plotenv/bin/python ../../../../tools/plot_weekly.py . evidence/plots/week-fresh.png evidence/week-fresh.md fresh
```

`evidence/attempt1/weekly-v1.society.ron` is the first overlay (work shut, a rest-day `visit`); `evidence/attempt1/<run>/` its results. `evidence/sweep/fun-w{1,4,8,16}/` are the `fun`-scored `laze` (`overlay.ron` in `fun-w1`, the weight edited for the others; these also carry the `visit`), and `evidence/sweep/with-visit-v{7,42}/` the final overlay plus the rest-day `visit` (`with-visit-v7/overlay.ron`).

## 1. Off is the previous build, byte for byte (prediction P4: holds)

The default run against a binary built from `main` (`8bfdb25`, in a scratch worktree), village seed 7, 29 game-days: `cmp` finds `summary.json`, `series.csv`, `narrative.txt`, `activities.txt`, `partners.txt`, `reapproach.txt`, `sleep.txt` and `floor.txt` **identical**. (The first baseline I copied, `target/release/examples/evidence`, was not `main`: the previous pass's flip check had built into the same target directory with `rebuff_memory_hours = Some(12.0)` as the default, and the files differed from line 3 of the narrative. A binary built from the right commit is identical.)

## 2. The week, seven runs (`evidence/week.md`, `evidence/plots/week.png`)

Predictions, pre-registered in `research.md` before any run:

| | prediction | result |
|---|---|---|
| P1 | work and craft are zero on rest days, the same share on workdays | **holds**: 0.0 % on days 5 and 6 in all seven runs; workday share 17.2 % against 15.5 % (the work moved to five days) |
| P2 | fewer different people on rest days, more of the approaches to friends (Sekara) | **struck by the first overlay, holds with the third** (below) |
| P3 | the society is not damaged | **struck by the first overlay** (friend ties halved, rival ties up 1.8-3×), **holds with the third**: rival ties fall or hold in seven of seven, insults fall in seven of seven, friend ties are mixed in the villages (51 → 39, 59 → 67, 64 → 44, 58 → 71) and higher in the towns |
| P4 | off is byte-identical | holds (section 1) |

Mean of the seven (rest days against workdays within the overlay's own week; the no-week columns are the same seeds):

| | no week, workdays | no week, rest | overlay, workdays | overlay, rest |
|---|---|---|---|---|
| work, % of villager-ticks | 15.5 | 15.1 | 17.2 | 0.0 |
| leisure, % | 2.3 | 2.3 | 2.4 | 36.5 |
| approaches a villager-day | 10.7 | 10.8 | 10.7 | 7.5 |
| different people met | 4.64 | 4.73 | 4.65 | 3.59 |
| share of approaches to a friend, % | 33.4 | 32.8 | 36.8 | 44.0 |
| mean `social` need | 0.76 | 0.76 | 0.76 | 0.71 |
| mean `fun` need | 0.62 | 0.61 | 0.64 | 0.83 |

Per run, rest-day minus workday inside the overlay's week: approaches -2.6 to -4.3, different people -0.7 to -1.5, friend share +1.4 to +13.0 points, `social` -0.036 to -0.076; the sign is the same in seven of seven.

**Struck: "a rest day with no work is narrower and friendlier".** With `work` shut and a rest-day `visit` (`evidence/attempt1/`) a rest day had 27-30 approaches a villager (workday 10; the towns 19-20), 6.6-7.1 different people against 4.2-4.7, and friend ties 51 / 59 / 64 / 58 → 25 / 21 / 34 / 22, rival ties 44 / 51 / 33 / 34 → 80 / 134 / 98 / 102, insults 491 / 433 / 466 / 429 → 650 / 800 / 692 / 686 at seeds 7 / 42 / 11 / 99. The freed hours go to whichever activity is open, and `compliment`, `gossip` and `snub` score on affinity or grudge and not on a need, so they absorb them. `visit` was begun 0.03-0.07 times a villager-day. A `laze` that scores by the `fun` need fills `fun` and drops out (`evidence/sweep/fun-*`: leisure 12-14 % at weights 4, 8 and 16, 21-23 approaches). A `laze` with a flat score and a weight did what the paper says (section 3).

## 3. The sweep and the visit control (seed 7; `evidence/sweep/`)

| weight of `laze` | 1 | **3** | 8 |
|---|---|---|---|
| rest-day approaches a villager (workday 10.1 / 11.0 / 10.4) | 11.5 | 7.8 | 6.2 |
| rest-day different people (workday 4.2 / 4.3 / 4.2) | 4.4 | 3.4 | 3.1 |
| rest-day friend share, % (workday 39.6 / 40.6 / 40.3) | 42.4 | 49.5 | 48.3 |
| rest-day `social` need (workday 0.75 / 0.76 / 0.76) | 0.78 | 0.71 | 0.64 |
| friend / rival ties (no week: 51 / 44) | 57 / 35 | 39 / 17 | 70 / 21 |

3 is the first weight at which a rest day is clearly quieter and narrower; 8 pushes the `social` need to 0.64. The weight is a design default and the overlay says so.

With and without the rest-day `visit` (final overlay, seeds 7 and 42): rest-day friend share 47.0 and 48.5 % with it, 49.5 and 55.3 % without; approaches 8.1 and 8.1 with it, 7.8 and 8.0 without; different people 3.4 and 3.6 with it, 3.4 and 3.5 without. The narrowing does not come from the visit, so it was dropped.

## 4. Fresh eyes (seed 123, 24 people; seed 5, 50 people; `start_weekday` 3; not used to choose the weight; `evidence/week-fresh.md`, `evidence/plots/week-fresh.png`)

Rest day against workday in the overlay's week: approaches 8.5 and 6.9 against 10.4 and 10.2, different people 3.35 and 3.52 against 4.01 and 5.21, friend share 46.0 % and 39.9 % against 34.5 % and 33.2 %, `social` 0.75 and 0.69 against 0.77 and 0.75. `start_weekday` 3 moves the rest days to game-days 2-3 of each seven and the ledger finds them on weekdays 5 and 6. Friend ties 74 → 39 and 105 → 186, rival ties 26 → 24 and 101 → 47, insults 412 → 461 and 957 → 768.

## 5. Costs and what is not explained

- The `social` need is 0.04-0.08 lower on a rest day than on a workday, in all seven runs: a quiet day is under-social. It is the price of the weight and is recorded in the overlay's comment.
- **Monday** is the busiest workday of the overlay's week (12.6 approaches a villager against 10.1-10.5 on the next four days and 10.3 on a Monday with no week; the fresh two repeat it, 12.1). The need ran down over the rest days and is spent when the week begins. This is believable and has not been tuned.
- Friend ties in the villages are mixed (3 of 5 lower: 51 → 39, 64 → 44, 74 → 39; 2 higher). Approaches over the whole week fall about 9 % (9.8 against 10.7) and contact makes friends in a small village as much as rivals. I did not find a mechanism beyond the lower contact rate.
- **Not measured:** the overlay on the provisioning pack, on the village pack's `craft` outside the harness's copy of the gate, and at 5 000 people.

## 6. Gate

See `perf.md` for the cost; the test gate results are at the foot of this file once the batches finish.
