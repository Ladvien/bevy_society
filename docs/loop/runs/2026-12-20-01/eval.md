# Eval

## Change

`binary_search` → `contains` for the "the target person still exists" check in `end_activities`.

## Before / after, same seed (`village-evening`, 17 game-days)

| | interrupted | share | `compliment` failures | friends | rivals |
|---|---|---|---|---|---|
| before | 2 237 | **22.8%** | 1 045 | 22 | 46 |
| after | **400** | **5.9%** | **11** | **31** | **38** |

Interruptions **down 82%**, `compliment` failures **down 99%**, friends **up 41%**, rivals **down 17%**.

## Causal, not coincidental

Reverting the single expression restores 2 237 and 1 045 exactly. Nothing else changed.

## The diagnosis

1. A per-activity breakdown (`interrupted_by_activity`) showed `compliment` was 47% of all failures.
2. Instrumenting **every** exit path at once showed all 1 045 on the `gone=true` branch.
3. `binary_search` and `contains` were compared **on the same call** and disagreed: "gone" vs present, in a list
   of 24.
4. Printing raw entity ids showed the vector is **strictly descending** (`4294967234, 4294967233, …`), which is
   exactly the condition that breaks a binary search.

Each step was a measurement, not an inference. Step 3 is the one that would have been easy to skip and is the
reason the fix is provably right rather than plausible.

## Why the totals move the other way

Total endings fall 9 808 → 6 768. A compliment that used to die on its first tick and be re-chosen immediately is
now one completed activity, so the village does **fewer, longer** things. The activity mix keeps its shape
(sleep 40%, work 24%, eat 10%, compliment now visible at 7%), so this is not a village doing less — it is a
village no longer restarting the same thing repeatedly.

## Gates

| Command | Result |
|---|---|
| `cargo test` / `--no-default-features` | **58** binaries |
| clippy / docs / fmt | 0 errors |

## Coverage, honestly

The shipped test is a **threshold guard**, not a reproduction. Six attempts to reproduce the defect failed —
including using the real village fixture — because it needs the exact `PersonId` ordering the evidence harness
produces. That is stated in the test's own doc comment rather than left as an implication.
