# Eval

## Change

**None.** Seven levers measured, none reduces the sleep jam; three damage the tie network. Reverted.

## The table that is the deliverable

| lever | interrupted | sleep | friends | rivals | comparable |
|---|---|---|---|---|---|
| baseline | 400 | 150 | 31 | 38 | — |
| scatter (RNG) | 456 | 173 | 25 | 42 | **no** |
| scatter (deterministic) | **484** | 174 | 25 | 36 | yes |
| `stop_radius` 0.75 → 1.5 | **400** | **150** | 31 | 38 | yes — byte-identical |
| `interaction_radius` → 2.2 | 378 | 160 | 24 | **90** | yes |
| `interaction_radius` → 2.6 | 364 | 136 | 19 | **82** | yes |
| `Home` honours capacity | **440** | 155 | 22 | **69** | yes |

## The three findings worth keeping

1. **`stop_radius` is a no-op.** A byte-identical simulation from a change to walker push distance means the wish
   is not the limiting factor. That reframes the problem from "tune the walker" to "instrument the solver".
2. **Scattering the goal hurts, both ways.** So the crowd is not failing because it shares a point. The obvious
   geometric story is wrong.
3. **`interaction_radius` is shared with contact.** Lowering sleep from 150 to 136 costs 38 rivals becoming 90. This
   is the same coupling six earlier clamp iterations mapped: proximity changes ties, so it is not a free lever.

## A methodology error I caught and corrected

The first scatter test **consumed the RNG stream**, so 456 versus 400 was not a comparison at all — it was a
different simulation. Re-running the same idea deterministically (golden angle by `index.sorted` position, no draw)
gave the real answer: 484. The purple bar in `evidence/levers.png` marks the non-comparable run so the record does
not quietly present it as evidence.

## Gates

| Command | Result |
|---|---|
| `git diff --stat src/` | **empty** |
| `cargo build` | 0 errors |
| `cargo test` (baseline, before any lever) | **60** binaries green |

## Dead-code inventory

Zero, and zero deltas: nothing was added.
