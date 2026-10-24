# Eval

## The gap, before and after

Before: `Doing.activity` — one name, no alternatives, no scores.

After, from the village at seed 7, tick 4147 (`evidence/village/why.txt`):

```
# tick 4147: someone chose compliment (0.752), 25% below snub (1.000)

 1.000  snub
 1.000  relax
 1.000  wander
 0.971  eat
 0.912  chat
 0.752  compliment   <- chosen
```

Six options weighed; the lowest was chosen; the sentence says so and says by how much.

## The constraint that matters

Game AI Pro 3 ch. 6: recording must have **no impact on the outcome**. Asserted by running the same
seed twice, once with a trace and once without:

| | chosen activity sequence |
|---|---|
| without a trace | 300 choices |
| with a trace | **identical, element for element** |

`tests/trace.rs::recording_does_not_change_what_people_do`. Determinism also still holds
(`tests/determinism.rs`, 2/2), so the recorder has not touched the RNG stream.

## Memory is fixed

`DecisionTrace::with_capacity(2, 8)` after five pushes holds exactly 2, dropping the oldest. After
4 000 ticks with `capacity: 8` it still holds exactly 8, newest first. That is the chapter's "fixed
memory budget" and its "discard the oldest frame" policy, asserted.

## The measure the iteration had to fix

The first version reported `margin -0.33`, which is impossible for a "fraction better than the next
best". Cause: selection is Dill's **weighted draw over the viable tier**, so the chosen option is often
not the highest scorer, and comparing it with the next item in a sorted list can go negative.

| | village output |
|---|---|
| before | `margin -0.33` |
| after | `25% below snub (1.000)` |

Four unit tests pin the new semantics, including "a weighted draw that picked a lower scorer reports a
shortfall".

## Perf

| | ms/tick at 1 000 people |
|---|---|
| without the trace | 15.28 |
| **with the trace** | **15.16** |

Statistically identical: the recording is `Option<..>`, so a host that never inserts the resource pays
nothing, and a host that does pays nothing measurable. Re-measured twice each way.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **26** binaries (25 + `trace`) |
| `cargo test --no-default-features` | 26 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero. The new module is public and exercised by three integration tests and four unit tests; nothing
was added that nothing consumes.
