# Eval: `Tuning::forgiveness`

Regenerate (release, from the repository root):

```sh
cargo build --release --example evidence
# before = HEAD c95eac7 (or this commit with --forgiveness 0, byte-identical: see below)
./target/release/examples/evidence --scenario village-evening --seed S --days 17 --out DIR          # S in 7 42 11 99
./target/release/examples/evidence --scenario village-evening --seed S --people 60 --days 30 --out DIR  # town, S in 7 99
# sweep: add --forgiveness G (G in 1 2 4 8 16 32)
/tmp/plotenv/bin/python tools/plot_forgiveness.py docs/loop/runs/2026-10-30-25 evidence/plots/before-after.png <sweep-dir> evidence/plots/sweep.png
```

`series.csv` stays ignored; `summary.json`, `narrative.txt`, `why.txt`, `grudge_sources.txt`, `partners.txt`, `shares.txt` are committed under `evidence/{before,after}/`.

## Before / after at the shipped default (12.0)

| run | insults | accepted compliments | compliments : insults | rival pairs | friend pairs | became / left rival |
|---|---|---|---|---|---|---|
| village s7 | 343 → 251 | 1005 → 1211 | 2.9 → 4.8 | 54 → 28 | 46 → 62 | 105/51 → 67/39 |
| village s42 | 382 → 240 | 1082 → 1251 | 2.8 → 5.2 | 56 → 24 | 30 → 65 | 128/72 → 61/37 |
| village s11 | 348 → 255 | 1101 → 1181 | 3.2 → 4.6 | 59 → 18 | 40 → 47 | 114/55 → 47/29 |
| village s99 (fresh eyes) | 361 → 249 | 1151 → 1195 | 3.2 → 4.8 | 66 → 36 | 35 → 54 | 129/63 → 76/40 |
| town60 s7 | 1740 → 1401 | 5746 → 6087 | 3.3 → 4.3 | 277 → 111 | 49 → 101 | 819/542 → 530/419 |
| town60 s99 (fresh eyes) | 1720 → 1491 | 5747 → 5770 | 3.3 → 3.9 | 229 → 155 | 52 → 86 | 846/617 → 574/419 |

Snubs answering an insult received and accepted (`grudge_sources.txt`, village s7): 349 of 613 (56.9%) → 224 of 440 (50.9%). Bram's narrative: 24 snubs → 12.

## Gain sweep (`evidence/plots/sweep.png`)

| forgiveness | village s7 insults / rivals / friends | town60 s7 insults / rivals / friends |
|---|---|---|
| 0 (HEAD) | 343 / 54 / 46 | 1740 / 277 / 49 |
| 1 | 335 / 43 / 36 | |
| 2 | 341 / 58 / 31 | |
| 4 | 301 / 41 / 41 | 1683 / 205 / 73 |
| 8 | 306 / 51 / 32 | 1450 / 148 / 81 |
| 12 (shipped) | 251 / 28 / 62 | 1401 / 111 / 101 |
| 16 | 233 / 28 / 58 | 1310 / 89 / 107 |
| 32 | 182 / 10 / 64 | 1213 / 53 / 120 |

The town responds monotonically and smoothly; the 24-person village is noisy below 8 (one seed is chaotic at that size) and clear from 12.

## The rumor-book pin (`tests/rumor_book.rs`, 200 people, 12 game-days)

Measured with a throwaway copy of the test's `crowded()` at each gain (deleted before commit):

| forgiveness | rumors | still tellable | tellable % (pin: under 20%) |
|---|---|---|---|
| 0 | 1678 | 266 | 15.9 |
| 4 | 1636 | 263 | 16.1 |
| 8 | 1517 | 270 | 17.8 |
| 12 | 1495 | 274 | 18.3 |
| 16 | 1315 | 264 | **20.1** (fails) |
| 24 | 1206 | 267 | 22.1 (fails) |

Tellable rumors stay near 265 whatever the gain; fewer insults only shrink the book. 16 was the first default tried and the full suite failed on this pin alone (every other binary green), so 12 ships.

## Off is HEAD

`--forgiveness 0` on this commit against the HEAD binary, village s7: `summary.json` and `narrative.txt` byte-equal (`cmp`).
