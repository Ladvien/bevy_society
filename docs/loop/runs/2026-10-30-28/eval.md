# Eval: `Input::Stock` and the provisioning overlay (households make bread when hungry and bare)

Regenerate (release, from the repository root; `series.csv` is ignored by git, `summary.json`, `narrative.txt`,
`activities.txt`, `provisioning.md` and the plots are committed under `evidence/`):

```sh
cargo build --release --example evidence
# before = the default pack (byte-identical to HEAD 7ead295: checked below); after = `--provisioning`
./target/release/examples/evidence --scenario village-evening --seed S --days 17 [--provisioning] --out DIR            # S in 7 42 11 99
./target/release/examples/evidence --scenario village-evening --seed S --people 60 --days 30 [--provisioning] --out DIR # S in 7 99
./target/release/examples/evidence --scenario village-evening --seed 3 --people 40 --days 20 [--provisioning] --out DIR # fresh eyes
./target/release/examples/evidence --scenario village-evening --seed 7 --people 60 --seconds-per-day 240 --days 12 [--provisioning] --out DIR
/tmp/plotenv28/bin/python tools/plot_provisioning.py docs/loop/runs/2026-10-30-28 evidence/plots/provisioning.png evidence/provisioning.md
```

(`/tmp/plotenv28` is a venv with matplotlib and Pillow; any Python 3 with both does.) The `t7d240` pair was run on the
binaries of the experiment itself (HEAD's pack, and the pack the overlay encodes), before the `--provisioning` flag existed;
a 64-second-day run through each route gave identical `summary.json` and `series.csv` (`diff` clean), so they are the same
run. Village is 24 people on the harness's 64-second game-day (4 096 ticks); the town is 60 on the same layout (the
tavern and the market are home to 16 people each, as the harness's own warning says).

## What the harness found (HEAD, before any change)

`activities.txt` (new) for the town of 60 at seed 7, ten game-days (`evidence/diagnosis/`):

| what | count |
|---|---|
| `bake` started in ten game-days | **3** (the 24-person village: 0.5-2.7 a day) |
| decisions made at clock hour 6, the one hour `bake` scores | about 30 a day; **6 in ten days were `bake`** (2 %) |
| at that hour: `compliment` / `gossip` / `eat` / `play` | 123 / 67 / 20 / 51 |
| `eat` started a day / completed | 65-99 / 34-78 |
| bread a head, day 2 → day 10 | 0.91 → 0.13 |
| villagers at the floor on `food`, day 2 → day 10 | 22 → 41 of 60 |

And at the library's own 240-second day (`before/t7d240`, 12 game-days): bakes 5, 1, 2, 1, 0, 0, 0, 0, 0, 1, 1, 0 a day,
bread a head 1.13 → 0.05, **86 %** of person-ticks at `food` below 0.05 on day 12. So it is not the 64-second day's
walking: 66-140 meals a day are *started* and 48-117 reach a table at the 240-second day, and find nothing.

Not reproduced in the unit-test helper's town (`tests/common/village.rs`, 0.25 s updates, everyone homed at the well, start
08:00): there the clock-driven pack holds 2.75-3.56 loaves a head on game-day 8. The harness's layout (tavern and market are
homes) and its 1/64 s tick are what produce the collapse; which of the two matters was not isolated. The regression
tests (`tests/provisioning.rs`) therefore pin the **mechanism**, not the town.

## The metric: villagers at the floor on `food`, bread a head, bakes a day (`evidence/provisioning.md`)

| run | side | floor %, days 2-6 | floor %, last 6 days | bread a head, last 6 days | bakes a day, last 6 days | friends / rivals | insults | shares |
|---|---|---|---|---|---|---|---|---|
FILLED-BY-SCRIPT

- **The town holds its bread.** Bread a head ends at 0.00 / 0.05 (64-second day, 30 game-days) and 0.19 (240-second day) on
  the default pack and at 1.16 / 1.10 / 1.40 on the overlay; the floor share ends at 100 / 96 / 63 % against 38 / 37 / 23 %.
  Bakes a day answer to the shelves: 4.2 and 5.3 on the overlay town against 0.0 and 0.3.
- **The village is not hurt.** Floor 19.4 / 21.7 / 17.7 / 21.9 % → 15.8 / 17.9 / 16.7 / 17.4 %, bread a head 1.0-2.5 → 1.4-1.5
  (the default pack's bread wanders between 0.5 and 2.8 a head with the lottery; the overlay's is flat).
- **What else moved.** Friend ties rise in all four villages (41 / 43 / 37 / 42 → 50 / 58 / 48 / 75); rivals fall at two
  seeds and rise at two (36 / 36 / 35 / 38 → 30 / 42 / 47 / 37); insults −4 % to −17 % at three seeds, +7 % at one. A fed
  village talks more (pass 27 saw the same direction). The towns' ties barely move (60 / 7: 101 / 127 → 109 / 119 friends /
  rivals; 60 / 99: 89 / 116 → 103 / 167), and `shares` rise from 16 / 43 to 611 / 631 because there is bread to give.

## What the overlay did not fix

The overlay town still has 37 % of person-ticks at the floor on `food` with 1.1 loaves a head. Bread is there; people do not
eat in time, because `eat` is one draw among many by `weight · score` (a starving person's `eat` and a flattered person's
`compliment` both score near 1) and a meal costs a walk to a table in a place with 18 seats for 60 people. That is
hunger's **prepotency** (Maslow 1943, DOI 10.1037/h0054346), a measured change of its own and the next backlog item; it
is the difference between this town (37 %) and the village (17 %).

## The block, and why the starter pack is unchanged

Window variants, measured against `tests/starting_larder.rs`'s `empty > 0.2` (day-2 floor share of an empty-larder village,
240-second day) and the 64-second-day town at game-day 10 (HEAD: 69 %, bread 0.13):

| bake window, shelf term | `empty` (needs > 0.2) | town floor, bread, day 10 |
|---|---|---|
| HEAD: `Hours(6, 7)`, none | 0.387 | 69 %, 0.13 |
| `Hours(6, 7)` + shelf | 0.322 (passes) | **70 %, 0.24** (not fixed) |
| `Hours(6, 8)` + shelf | 0.195 (fails by 0.005) | not run |
| `Hours(6, 12)` + shelf | fails (not recorded) | 39 %, 1.34 |
| **`Hours(6, 20)` + shelf + hunger** (the overlay) | **0.057** (fails) | 33 %, 0.93 |

The one-hour window is the defect, not the missing shelf term, and any window wide enough to fix the town cures the
day-2 transient on its own: the overlay's empty-larder village has a day-2 floor of 0.057 because its households bake the
first time they are hungry. That is the assertion's precondition falsified by the fix; the loop's rule is that an assumed
decision never removes or loosens a protected assertion, so the starter pack stays as it is and the flip is
`docs/loop/needs-decision/households-provision-themselves.md` with `provisioning-default.patch` (the patched pack and the
one deleted assertion). The **second** assertion of that test (`stocked < empty / 2`, 0.000 against 0.057) holds on the
overlay.

## Default behaviour unchanged

The default pack is bit-identical to HEAD: a three-day village run at seed 7 on this branch's harness and on HEAD's
(`7ead295` plus the ledger) gave identical `summary.json`, identical `series.csv` (`cmp`) and identical `activities.txt`.
`Input::Stock` is an unused match arm unless a pack names it.

## The flip (for the owner)

FLIP-RESULT

## Gate

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps --all-features`: clean.
GATE-RESULT
- New: `tests/stock_input.rs` (4), `tests/provisioning.rs` (2). Both fail on `7ead295` (it has no `Stock` input: the
  definitions do not parse). Protected checks: none edited; the one block is recorded above.
