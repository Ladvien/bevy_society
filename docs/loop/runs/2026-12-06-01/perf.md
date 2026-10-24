# Perf

## The change

Documentation and one test. No crate code changed, so there is nothing to measure.

## Cost

`tests/need_cycle.rs` runs 80 000 ticks over two 8-person villages, about 56 seconds — the slowest test in the
suite. It earns that: it is the only thing that checks whether a need has a source at all, and it is the test
whose *name* was lying about its own output.
