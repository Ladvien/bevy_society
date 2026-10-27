# Performance: one multiply per arrived person, one `Stock` read per consuming activity

The change is a `Stock` component read plus a `min(1, held/asked)` fold inside `arrive_and_satisfy`'s
per-person loop, for **arrived** people only, and only when the activity declares a `consumes`. An activity with
none — every activity in the pack but `eat` — takes the `_ => 1.0` arm and never touches the query. `Stock::get`
is a `Vec` index.

**Measured, interleaved, on the laptop**: `examples/benchmark --people N --ticks 200 --reps 5`, alternating the
before build (`~/.cache/bs-before`, compiled from `55475ea`) and the after build, two rounds each.

| persons | before (ms/tick min / median) | after (ms/tick min / median) |
|---|---|---|
| 100 | 1.633 / 2.022 · 1.904 / 2.260 | 1.399 / 1.867 · 2.176 / 2.797 |
| 1 000 | 49.62 / 78.85 · 50.80 / 80.79 | 45.19 / 76.16 · 51.85 / 80.89 |

**Inside the noise at both sizes, and the noise is large.** The benchmark reports its own spread: **55% at 100
people** and **126% at 1 000** across five reps. The after build is faster in one round at each size and slower
in the other, which is what a change with no measurable cost looks like on this machine. No claim of a speedup
is made, and none is possible at this spread.

**Why it cannot cost much.** The added work is per arrived person, and the decision pass above it is
`people × activities × neighbours` — the ratio at 1 000 people is 499 500 neighbour pairs a tick against at
most 1 000 `Stock` reads. The `Stock` query is `Query<&crate::Stock>`, which Bevy resolves through the same
archetype as the `Needs` query already beside it, so it adds a column and not a traversal.

**The benchmark's pack has no consumer of its goods**, so the fold's body never runs there: this measurement
prices the query and the branch, not the arithmetic. The arithmetic is one multiply per need slot per tick,
which is the same shape as the `satisfies` payout it scales.

**`big` was not used this iteration.** The suite, the gates and both evidence builds ran on the laptop; the
release suite takes ~6 minutes and the four gate commands ~2 minutes more, which did not justify the round trip.
`tools/big-cargo.sh` is unchanged and documented in `docs/DEMOS.md`.
