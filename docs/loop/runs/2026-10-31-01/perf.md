# Perf

This iteration's whole content is the perf number. Full detail in `eval.md` and `research.md`;
evidence in `evidence/scale-{100,1000,5000}.json`.

| persons | ms per society tick | amortised % of a 60 fps frame at `tick_secs = 0.25` |
|---|---|---|
| 100 | 0.549 | 0.2% |
| 1 000 | **14.086** | 4.9% |
| 5 000 | **351.640** | **140%** |

Reproducibility: 14.168 vs 14.167 ms/tick across two 400-tick runs at 1 000 persons.
Quadratic confirmed: cost grew 25.0× for 25× the pairs (1 000 → 5 000), and `ms_per_million_pairs`
stays flat at 28.1–28.2.

**No code was optimised**, so nothing was made slower. The crate's own per-tick cost is unchanged by
construction: `examples/benchmark` is a binary that reads it.
