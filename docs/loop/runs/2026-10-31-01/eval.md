# Eval

## The measurement

Release build, M1 Max, starter pack, `tick_secs = 0.25`, 400 ticks at 100 and 1 000 persons, 60 at
5 000 (it takes four minutes). Evidence: `evidence/scale-{100,1000,5000}.json`.

| persons | ms per tick | pairs per tick | ms per million pairs | amortised % of a 60 fps frame |
|---|---|---|---|---|
| 100 | 0.549 | 4 950 | 110.8 | 0.2% |
| 1 000 | **14.086** | 499 500 | 28.2 | 4.9% |
| 5 000 | **351.640** | 12 497 500 | 28.1 | **140%** |

## Reproducibility

Two independent 400-tick runs at 1 000 persons:

```
{"people": 1000, "ticks": 400, "ms_per_tick": 14.168, ...}
{"people": 1000, "ticks": 400, "ms_per_tick": 14.167, ...}
```

A 0.01% spread, so the number is stable enough to document and to regress against.

## Quadratic, confirmed

`ms_per_million_pairs` converges to 28.1–28.4 between 1 000 and 5 000 persons — the signature of a
cost proportional to pair count. The 100-person figure (110.8) is dominated by fixed per-system
overhead because 4 950 pairs is a rounding error.

## What changed

| | before | after |
|---|---|---|
| `docs/guide/concepts.md` | "a few hundred thousand distance checks per tick — fine" | the table above, and the frame-cost interpretation |
| `scenarios.yaml` `scale-1000` | `pending` | `measured`, with the numbers in its observable |

## Gates

| Command | Result |
|---|---|
| `cargo test` | 17 binaries ok |
| `cargo test --no-default-features` | 17 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Dead-code inventory

Unchanged at zero. `examples/benchmark` is an example binary, not library surface.
