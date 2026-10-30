# Plan

`Tuning::friend_capacity` (10, 0 = off) and `Tuning::bond_scale(friends, already_friend)`; applied to positive affinity gain in `apply_effects` (behavior.rs), `Tie::expose` / `expose_toward` / `expose_contacts` (relationships.rs) and dialogue exposure. Starter pack `snub` grudge curve `Linear(1,0)` -> `Power(1.15)`. Tests: `tests/friend_capacity.rs`, `tests/friendship_bound.rs` (existing, protected, unchanged).
