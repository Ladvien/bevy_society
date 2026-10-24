# Perf

## The change

Documentation and three tests that call `SocietyDefs::from_ron` on strings. No runtime path is involved.

## Cost

`tests/guide_blocks.rs` parses four small definitions files — microseconds. Nothing to measure, and nothing was
slowed.
