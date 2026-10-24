# Eval

## Change

1. `perceive` runs a **uniform grid** (cell = `perception_radius`) instead of an all-pairs scan. The answer is
   identical — same neighbours, same ascending index order.
2. `close_doing` is built from the person's close ties, not the whole population cloned per person.
3. The docs and the benchmark's header now state the **measured** cost breakdown instead of the false claim
   that perception is the quadratic term.

## Perf

| persons | all pairs | **grid** | saving |
|---|---|---|---|
| 100 | 0.99 ms | **0.59 ms** | 40% |
| 1 000 | 14.55 ms | **13.50 ms** | 7% |
| 2 000 | 53.96 ms | **49.91 ms** | 8% |
| 5 000 | 254.54 ms | **245.13 ms** | 4% |

## The finding that matters more than the saving

Instrumented at 5 000 people: `perceive` is **36 ms of a ~320 ms fixed step** (29 ms with the grid). The
crate claimed perception was the quadratic term. It is **12–14%**.

The real term: every `Person`-targeted activity is scored against every neighbour — 182 of them at 5 000
people on the benchmark's disc, which is the honest density (0.995/m²), so `5 000 x 4 x 182 = 3.6M`
evaluations a tick against 144 000 at 1 000 people. That is the curve. **No index removes it**; it is a
decision-design cost.

I measured this after two wrong guesses (a `close_doing` clone, then a lookup map), both of which moved the
number by ~5%, and both of which I had reasoned would be large. The instrumentation is what settled it.

## Behaviour

**Unchanged, bit for bit.** 67 199 of 67 199 `village-evening` series rows identical.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **40** binaries |
| `cargo test --no-default-features` | **40** binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

`tests/leaf.rs` caught a `HashMap` I had put in `behavior.rs`; removed rather than allow-listed, since it
bought ~5%.

## Dead-code inventory

Zero.
