# Eval

## Change

`summary.json` reports `became_rival` and `left_rival` — the two halves of reconciliation. No simulation code
changed.

## The finding

| seed | became rival | left rival | ratio | final rivals | friends |
|---|---|---|---|---|---|
| 7 | 272 | **188** | 1.45 | 42 | 18 |
| 42 | 308 | **238** | 1.29 | 35 | 19 |
| 11 | 377 | **297** | 1.27 | 40 | 14 |

**The village reconciles.** A quarter of rivalries end. But it forms them 1.3-1.45x faster than it dissolves them,
and settles at **15% of all pairs being rivals against 6.5% being friends**. The rival count decelerates
(second-half growth 0.68x the first), so it saturates rather than running away.

Before this, `summary.json` reported only how many rivalries *exist*. A reader could not have told a village that
holds grudges from one that never releases them, and both would have shown a rising `rivals` count.

## The lever, and why it was not pulled

| `affinity_relax_per_day` | rivals | friends | ratio |
|---|---|---|---|
| 0.01 (shipped) | 42 | 18 | 1.45 |
| 0.015 | 28 | 7 | 1.22 |
| 0.02 | 25 | 7 | 1.20 |
| 0.03 | **10** | 5 | **1.08** |

Monotone on rivalry, and **wrong on friendship**, because relaxation pulls all affinity toward zero and cannot
distinguish a grudge from a fondness. Choosing a point here is choosing a village's temperament, which belongs to
the host — recorded in `needs-dependency/what-does-the-village-owe-its-grudges.md` with three options.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **70** binaries |
| clippy / docs / fmt | 0 errors |
| identity | became − left == 2 × standing rivals |
| swapped counters | caught: −84 against +84 |

## Coverage

The identity is arithmetic and holds for any dynamics, which is why it is the right thing to pin. It catches the
way this metric would silently lie — a counter wired to the wrong side of the transition — and a swapped-counter
run demonstrates it.

**A test was deleted rather than kept.** The first version asserted the ratio responds to `affinity_relax_per_day`;
it passed at 0.0 and at 0.02, so the fixture did not exercise what it claimed. The identity replaced it.
