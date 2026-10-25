# Research: the village existed three times, and one copy had drifted

## Why this

Item 3 in my log for four iterations: "the harness's village builder should move where `tests/` can reach it —
carried over four iterations, and it is why four fixtures last iteration could not reproduce the village."

That framing was **wrong**, and reading the code rather than the log is what showed it. `tests/common/village.rs`
already existed; the builder had been extracted. The problem was the opposite of the one I had written down.

## The actual state

The village is built in **three** places:

| file | role |
|---|---|
| `examples/common/scene.rs` | canonical — what the capture frames from |
| `examples/evidence.rs` | mirrors it, with a comment promising it agrees |
| `tests/common/village.rs` | what the tests assert against |

Two of them agree. `tests/common/village.rs` had **drifted**:

| | tavern | market | workshop | farthest pair |
|---|---|---|---|---|
| `examples/common/scene.rs` | (6.5, -5.0) | (-6.5, -5.0) | (6.5, 5.0) | **16.4 m** |
| `tests/common/village.rs` | **(8.0, -6.0)** | **(-8.0, -6.0)** | **(8.0, 8.0)** | **21.3 m** |

So `tests/determinism.rs`, `tests/friendship_bound.rs` and `tests/save.rs` were all asserting things about a
village **nobody watches or measures**. That is a worse defect than the one my log described, and it explains last
iteration's failed fixtures: I was writing tests against a layout that did not match the harness I was measuring.

## Why the distance matters, in the crate's own arithmetic

Reachable distance is `speed * seconds_per_day * travel_timeout_hours / 24`:

| layout | farthest pair | needs `travel_timeout_hours` at 64 s/day | declared |
|---|---|---|---|
| canonical | 16.4 m | **4.4** | 4.0 |
| drifted | 21.3 m | **5.7** | 4.0 |

Both are short of what they need, by different amounts. A test written on the drifted layout would have exercised
walks the measured village cannot make, and reported pass or fail for reasons the shipped clip never sees.

## What shipped

1. **`tests/common/village.rs` now matches `examples/common/scene.rs`** — same five numbers. The three tests that
   use it still pass, which is itself evidence the drift was invisible to them.
2. **`examples/evidence.rs` no longer keeps a copy.** It reads `common::scene::PLACES[..].1` directly — it is in
   the same crate tree and already had `mod common;`. The comment promising the mirror agreed is deleted, because
   a comment is not a mechanism. `series.csv` is **byte-identical** before and after.
3. **`tests/village_layout.rs`** pins the canonical table, the 16.4 m span, and that the shared builder matches.
   `tests/` cannot import `examples/`, so the literals are repeated with an instruction: if it fails, fix the copy
   that is wrong, never the table.

## Sources

None. This is a duplication defect, and the numbers are distances between constants.
