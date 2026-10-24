# Perf

Not measured: this iteration changed documentation, doctests and the evidence harness. No library
system, rate or loop was touched, so no scenario's tick cost can move.

The standing numbers (`docs/guide/concepts.md`, two iterations old) are unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.55 |
| 1 000 | 14.1 |
| 5 000 | 351.6 |

One harness cost does change, and is worth stating because the brief asks for it: `--people 40` on
`village-evening` takes **13.6 s** for 4 200 ticks, against **9.6 s** at the authored 24 — the extra
is perception over 40 bodies, not the documentation change. Both are dominated by `cargo run` startup
and PNG/CSV writing, not by the society.
