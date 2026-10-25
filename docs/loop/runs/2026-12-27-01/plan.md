# Plan: measure whether the village ever makes up

## Why this over the runner-up

Items 1, 3, 4 and 5 in the log are all decisions. Item 2 I closed last iteration. So instead of working the list,
I looked at the village — and the tie network was the thing that read wrong: **18 friends against 42 rivals**, with
rivals climbing 0 → 42 while friends stayed flat.

Before deciding whether that is a defect, I needed to know whether rivalries ever *end*. Nothing recorded it.

## Change

`examples/evidence.rs` now drains `TieLabelChanged` and reports `became_rival` / `left_rival` in `summary.json`.
Their difference is exactly the standing rival ties:

```
became_rival - left_rival == 2 * rivals
```

## Measured

| seed | became rival | left rival | ratio | final rivals | friends |
|---|---|---|---|---|---|
| 7 | 272 | **188** | 1.45 | 42 | 18 |
| 42 | 308 | **238** | 1.29 | 35 | 19 |
| 11 | 377 | **297** | 1.27 | 40 | 14 |

The village **does** reconcile, forming rivalries 1.3-1.45x faster than it dissolves them, and saturating.

## The lever, measured and not pulled

`affinity_relax_per_day` 0.01 → 0.03 takes rivals 42 → 10 and friends 18 → 5. Monotone on the target and wrong on
the neighbour, because relaxation cannot tell a grudge from a fondness. Recorded as a decision rather than chosen.

## Touched

- `examples/evidence.rs` — the two counters and their JSON fields.
- `tests/reconciliation.rs` — new; pins the identity and that reconciliation exists.
- `docs/loop/needs-decision/what-does-the-village-owe-its-grudges.md` — new.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.

## Verification

| check | result |
|---|---|
| identity holds | became − left == 2 × standing rivals, on the fixture and the village |
| swapped counters caught | yes — the identity reads −84 against +84 |
| `cargo test` both feature sets | **70** binaries |

## A test that was deleted

A first version asserted the ratio was sensitive to `affinity_relax_per_day`. It passed with the rate at **0.0 and
at 0.02** — the fixture's dynamics do not depend on relaxation, whatever the village's do. Rather than keep a test
that reports protection it does not give, it was replaced with the identity check, which is arithmetic and holds
for any dynamics.
