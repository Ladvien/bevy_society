# Perf

## The change

Documentation and one arithmetic test. No runtime path changed; the village is bit-identical at friends 22 /
rivals 33.

## Cost

`tests/exposure_equilibrium.rs` does no simulation — it reads `Tuning::default()` and asserts a fixed point.
Effectively free, which is why it is worth having: a rate change that moves the equilibrium becomes a test
failure with the margin printed.
