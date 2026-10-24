# Plan: correct the claim, pin the real cost

## Why this over the runner-up

Runner-up was doing the retirement itself (`needs-decision/retiring-untellable-rumors.md`, option 1).
Lost on one sentence: **the justification for that breaking change is wrong**, and shipping a semantic
change to `RumorBook.alive` on a false premise is worse than shipping nothing.

`alive` means "the subject is still in the world". Three places read it. A host reading `RumorBook` sees
it. Changing when it is cleared is a decision the operator has to make — and the operator's decision is
made harder, not easier, by a 4× cost that does not exist.

## Design

No library change. Two tests that pin what is true, so the next reader inherits a measurement instead of
a rumour:

| test | asserts |
|---|---|
| `the_book_accumulates_rumors_nobody_can_tell` | on a 3 000-tick run, the book holds > 1 000 rumors, **every one still `alive`**, and fewer than a fifth of them tellable by anyone |
| `the_book_costs_memory_not_ticks` | the book grows with run length, roughly linearly (4 865 vs 2 250 for twice the ticks), and the byte cost is printed |

And the documents are corrected at the source: the decision file gains a `Measured` section replacing the
4× claim, and the log's carried line is retired.

## Why a test and not just a doc fix

The 4× claim survived eight iterations because nothing could fail on it. A number in a top-5 list is
prose; a number in a test is checkable. The new tests fail if the book stops accumulating (which would
mean a retirement rule landed) and fail if it starts growing super-linearly (which would mean a real
defect).

## Touched

- `tests/rumor_book.rs` (new).
- `docs/loop/needs-decision/retiring-untellable-rumors.md` — the 4× claim replaced with the measurement.
- `docs/loop/log.md`, `CHANGELOG.md`.

## New `Tuning` fields

None.

## Tests

Both were written against a measurement taken first, so neither is aspirational. The first draft used
`common::settings()`'s default 240-second day and reported **0 rumors** — a 3 000-tick run is then half a
game-day — which is why the helper now sets `seconds_per_day = 4.0` with a comment saying so.
