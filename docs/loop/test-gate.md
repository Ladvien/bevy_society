# The test gate: what it costs, and why it is run the way it is

Measured 2026-10-28 on `big` (24 cores, 31 GB), for the change that folded 74 of the 81 `tests/*.rs` binaries into `tests/it/` and moved the gate to cargo-nextest. Every number below was taken on a shared machine: the run happened while society's own loop was in its gate, HomeStill was converting papers, and the unit was niced and given a low CPU weight. Read them as upper bounds, from one run each.

## The gate, as a pass runs it

```sh
RAYON_NUM_THREADS=1 cargo nextest run --profile gate
RAYON_NUM_THREADS=1 cargo nextest run --profile gate --no-default-features
cargo test --doc
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

nextest does not run doctests, which is why `cargo test --doc` is its own line. The CI release-only modules run as `cargo test --release --test it rumor_book::` and `friendship_bound::`.

## What it cost, before and after

Before, from the pass-20 gate logs of 2026-10-27: `cargo test` 47.1 minutes of test running, `--no-default-features` 21.9 minutes of compile and link for 78 test binaries at `CARGO_BUILD_JOBS=3` and 29.5 minutes of tests, then clippy and rustdoc still queued after more than two hours. After, one cold run on a fresh target directory, each step timed in its own cgroup:

| step | result | wall | CPU |
|---|---|---|---|
| nextest, default features | 327 passed, 2 skipped | 24.6 min | 9,177 s |
| clippy `-D warnings` | clean | 10 s (dependencies cached) | 18 s |
| nextest, `--no-default-features` | 306 passed, 2 skipped | 51.1 min, of which about 32 min was a cold build of the dependencies for that feature set and 19.2 min was tests | 7,744 s |
| `cargo test --doc`, both feature sets | pass | 68 s and 84 s | about 35 s each |
| `cargo doc -D warnings` | clean | 62 s | 144 s |
| `cargo check --release --test it` | compiles | 111 s | 277 s |
| plain `cargo test --test it`, skipping the 18 tests over 60 s | 233 passed | 243 s | 2,128 s |

The whole sequence took 1 h 25 min including the cold `--no-default-features` build. **A warm run, with that feature set's dependencies already built as they are in the loop's target directory, should take about 55 to 60 minutes. That figure is an estimate from these steps, not a measurement.** Test running alone went from 47.1 to 24.6 minutes (default features) and from 29.5 to 19.2 minutes (`--no-default-features`).

Not measured: the release-only modules `rumor_book` and `friendship_bound` were only compiled (`cargo check --release`), not run; the CI workflow edit has not run on GitHub's runners.

## Where the time goes

Per-test times from the first default-features run, before the rayon change: **18 tests over 60 seconds held 96% of the 12,350 seconds of per-test time**, and two modules, `label_hysteresis` and `reciprocity`, held 80%. nextest runs each test in its own process across all binaries at once, so the wall time is bounded below by the slowest single test. That was 34 minutes in the first run and 20 minutes (then 19) in the run with `RAYON_NUM_THREADS=1`, on this loaded machine. More parallelism cannot go below that; a cheaper test can.

## Why `RAYON_NUM_THREADS=1`

The first nextest run burned **7 h 11 min of CPU in 40 minutes of wall**, about ten cores busy, with the machine's load average above 150. Pinning one test to fewer CPUs showed why: `reciprocity::a_zero_rate_is_the_old_society`, same binary, same 24-core machine:

| rayon threads | wall | CPU |
|---|---|---|
| default (24, one per core) | 156 s | 494 s |
| 2 | 96 s | 108 s |
| 1 | 118 s | 104 s |

`bevy_stigmergy` runs a rayon pass every tick and rayon sizes its global pool from the core count. With one thread the same test costs about a fifth of the CPU and is no slower. With the variable set, the default-features run used 9,177 CPU-seconds in 24.6 minutes (about six cores busy) and passed 327 of 327.

Two things that were tried and did not work: a one-thread Bevy task pool in the test harness (`TaskPoolPlugin` with `TaskPoolOptions::with_num_threads(1)`) changed nothing and was reverted, because the threads were rayon's, not Bevy's. They show up in `/proc` named "Compute Task Pool" because a Linux thread inherits its creator's name and rayon creates its pool from inside a Bevy compute thread, while `ComputeTaskPool::thread_num()` still says 1. nextest 0.9.150 can set the variable from its config only through wrapper or setup scripts, which are still experimental there (the config fails to parse without enabling them), so it is on the command line.

## Memory is not the constraint

Eight tests running at once held 197 MB of anonymous memory in total, about 80 MB each; the cgroup's `memory.peak` of 7 GB in the table's runs is page cache from compiling. So `test-threads = 8` in `.config/nextest.toml` is a CPU choice, sized to leave cores for the other loops on a shared machine. It was not tuned beyond that.

## Why seven files are still their own binaries

`bound_places`, `crowded_home`, `save`, `unmade_goods`, `unreachable`, `unreachable_places` and `warnings` install a process-global tracing subscriber, directly or through `LogPlugin`. In one process the first installer wins and a second gets nothing, so they cannot share a binary under plain `cargo test`. `tests/it/registration.rs` names them in `SEPARATE`, and fails if a file is added to `tests/it/` without a `mod` line (it would never be compiled) or directly to `tests/` without an entry (it would become another binary). Both guards were shown to fail on a throwaway file and to pass again when it was removed.
