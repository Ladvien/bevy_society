# Eval

## Perception's cost, measured standalone

| people | brute-force loop, ms/tick | pairs | edges found |
|---|---|---|---|
| 1 000 | **1.45** | 999 000 | 169 000 |
| 5 000 | **35.49** | 24 995 000 | 4 264 430 |

35.5 ms of a 300.8 ms tick is **11.8%** at 5 000. The log's claim said 9.2%; that was the *scoring* cost,
not perception's, and it is corrected.

## The index, built and measured

| | brute force | uniform grid (`HashMap`) | `BTreeMap` grid |
|---|---|---|---|
| 100 people | **0.573** | 0.707 | — |
| 1 000 people | 15.67 | 15.99 | **16.97** |
| 5 000 people | 300.8 | **297.96** | — |

**Reverted.** A 1% gain at 5 000 is inside noise; the regressions at 100 and 1 000 are not.

## Why it did not pay

Brute force produces each neighbour list in ascending index order for free — both indices walk `0..n`.
The grid visits cells, so it must sort each list afterwards: **4 264 430 neighbour entries** at 5 000.
That sort costs about what the 25 M distance checks cost, and the nine hash lookups per person add to it.

SPH amortises a neighbourhood query over several solver passes per step. This crate makes **one**
perception query per society tick.

## What shipped

`tests/perception.rs`: a reference brute force and an entry-for-entry comparison against `Perception`.

```
compared 32016 neighbour entries across 180 people
```

That is the value of the iteration — the next attempt at an index is checked against a computed
reference rather than trusted, which is the same discipline that caught the `RumorId(0)` bug three times
and the `4×` rumor claim last iteration.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **32** binaries (31 + `perception`) |
| `cargo test --no-default-features` | 32 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. `src/core.rs` is unchanged from `HEAD`; the new test uses only public items.
