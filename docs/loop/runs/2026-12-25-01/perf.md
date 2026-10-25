# Perf

## The change

Two constants folded into one and a test file added. No simulation code touched: the harness reads
`common::scene::PLACES[..].1` at compile time where it used to read a local `const`, and both produce the same four
`Vec2`s.

## Cost

Zero, and demonstrated rather than asserted: `series.csv` is byte-identical, which for a deterministic simulator
means every computed value is unchanged.

## Scale curve, still unverified

Ninth iteration. The guide's `0.6 / 13.5 / 245.1 ms` table has not been checkable on this machine at any point in
this loop — load between 6 and 12, and a 7% spread on consecutive runs of the same binary at 1 000 people.

It is worth being plain about this: nine iterations of carrying the item means it is probably not going to be
resolved by waiting for a quiet moment. It needs either a benchmark machine or a different measurement method
(more repetitions, median rather than mean, and a report of the spread). That is a small, self-contained job
someone should do rather than a blocker to keep noting.
