# The loop log

One entry per iteration: the change, why, evidence, before/after numbers, the dead-code inventory delta, the three verdicts, the commit, and the top 5 remaining items.

The loop itself is `/.omp/commands/build-loop.md`; the scenarios it runs are `scenarios.yaml`.

## How to read this file

- **Dead-code inventory** is `cargo +1.98.1 clippy --all-targets --all-features -- -D warnings`: the count of unused items. It should only ever shrink as in-flight systems land, and each entry should name the system that will consume it. Unused items that nothing will consume are dead weight, not work in progress.
- **Verdicts** are always three, in order: behaviour, zoom, fresh eyes. Only three GREATs ship.
- **Evidence** paths are relative to the iteration's `runs/` directory.

## Iterations

_none yet_
