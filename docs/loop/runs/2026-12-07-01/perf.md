# Perf

## The change

One test. No crate code changed, so there is nothing to measure.

## Cost

`tests/cluster_reach.rs` runs two 12-person clusters over equal game-days at two clocks, about 44 seconds.
It is the slowest test after `need_cycle` (56 s), and it earns it: it is the only thing that checks the last
`pending` scenario, and it asserts a **failure**, which is the only way a failure stays visible.
