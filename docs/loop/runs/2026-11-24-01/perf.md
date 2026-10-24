# Perf

## The change

One pass over `defs.activities` at startup. No per-tick cost.

## Behaviour

Bit-identical. With the shipped 240 s/day pack the warning is silent, so nothing changed: travelling 46.8%,
needs 0.621 at the default.

## Standing benchmark

Not re-run: a startup-only loop over ten activities cannot move a per-tick benchmark. Recorded rather than
measured, and honestly so.
