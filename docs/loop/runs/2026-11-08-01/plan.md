# Plan: a bounded decision trace, so a host can ask why

## Why this over the runner-up

Runner-up was the back-to-back activity loop the previous iteration recorded as a blocker. Lost on
one sentence: **that blocker was my test's own made-up definition, not the crate's** — the starter
pack's `sleep` is gated `Hours(22, 6)` and ranked 1, so one person sleeps **once per day** (measured:
10 sleeps in 10.1 days), and last iteration's 29-in-20 000 came from a test definition with no hour
gate. Chasing that would have fixed a bug that does not exist in the crate.

This iteration takes the brief's own word *explainable*, which had no implementation at all.

## Design

A new `src/trace.rs`, opt-in, additive, no behaviour change:

- `DecisionTrace` — a `VecDeque` ring of `Decision`, holding `capacity` of them.
- `Decision` — tick, person, name, and up to `options_per_decision` `ScoredOption`s, best first.
- `ScoredOption` — activity name, score, chosen.
- `Decision::shortfall()` and `took_the_best()` — how far the draw fell below the best on offer.
- `Decision::describe(&[(Entity, String)])` — a one-line account for a log or a HUD.

`decide_activities` gains an `Option<ResMut<DecisionTrace>>` and, when it is present, records from
`all_scored` — a list filled beside the existing `best` list. **Nothing draws from `rng`, nothing
re-scores, nothing reorders.** That is the chapter's third constraint and the test that matters.

## The mistake this design caught in itself

The first version of the closeness measure was `(chosen − runner-up) / chosen`, which produced
**`margin -0.33`** on the village. Selection is Dill's *weighted random draw*, so the chosen option is
often not the best — a negative "fraction better than the next one" is meaningless. Replaced with
`shortfall` against the best score, and the trace now says *"a weighted draw picked this over a higher
scorer"*, which is true and was invisible before.

## Touched

- `src/trace.rs` (new), `src/lib.rs` (module + re-exports).
- `src/behavior.rs` — `all_scored`, the trace sink, `Option<ResMut<..>>`.
- `tests/trace.rs` (new) — three tests, including the no-outcome-change assertion.
- `examples/evidence.rs` — writes `why.txt`; `examples/benchmark.rs` — `BENCH_TRACE=1` to measure it.
- `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. Capacity and options-per-decision are `DecisionTrace` construction parameters, because a trace
is a debugging aid a host may or may not add, not a rate the simulation reads.

## Tests

| Test | What it proves |
|---|---|
| `recording_does_not_change_what_people_do` | the same seed gives a **byte-identical chosen sequence** with and without the trace |
| `the_trace_says_what_else_was_weighed_and_by_how_much` | the chosen option, its score, its shortfall, and the options it beat |
| `the_trace_is_a_fixed_size_ring` | exactly `capacity` decisions after 4 000 ticks, newest first |

Plus four unit tests in `src/trace.rs`: the ring bound, no shortfall when the best wins, a 25% shortfall
when a weighted draw picks a lower scorer, and a lone option reading as the best.
