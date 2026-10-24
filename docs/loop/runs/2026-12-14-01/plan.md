# Plan: make the guide's trigger syntax match the loader

## Why this over the runner-up

The runner-up was recovering the clamp's mood cost. But a host follows the **guide** before they ever reach
mood, and three of its nine trigger forms were copy-paste errors. No iteration had compiled the guide, and the
crate's loader is strict by design — it rejects unknown fields by name — so a stale example fails loudly for the
reader and silently for us.

## Design

1. The prose list becomes a **table** of all ten forms with what each fires on.
2. Two syntax traps documented where they bite: struct variants take parentheses, and `TieLabel` variants are
   capitalised.
3. `tests/guide_blocks.rs` asserts:
   - the guide's **opening definitions file** parses (extracted from the markdown, not restated);
   - **every row** of the trigger table parses;
   - the guide's **conversation block** parses, and `TieLabel::Friend.script_name()` is `"friend"` — the
     lowercase Rhai form is deliberate and different from the RON variant.

## Measured

| | before | after |
|---|---|---|
| documented trigger forms that parse | **6 of 10** | **10 of 10** |
| guide's opening definitions file | untested, parses | asserted |
| guide's conversation block | untested, parses | asserted |

## What was found while doing it

`TieLabel(friend)` fails — the variants are capitalised. The guide said `TieLabel(label)`, which is not a
writable form at all. Fixed with the four valid names stated.

## Touched

- `docs/guide/authoring.md` — the table and the two traps.
- `tests/guide_blocks.rs` (new, three tests).
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. No crate code changed.
