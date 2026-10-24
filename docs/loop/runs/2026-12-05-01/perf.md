# Perf

## The change

One test. No crate code changed: `examples/evidence.rs` gained documentation, `scenarios.yaml` gained
documentation.

## Cost

`tests/village_evening.rs` runs seven 4 200-tick villages, about 13 seconds. It is the slowest test in the
suite after `need_cycle` (97 s) and the determinism pair (25 s), and it is worth it: it is the only thing
asserting the scenario the README clip is recorded from.

## Behaviour

Unchanged. Nothing in the library was touched.
