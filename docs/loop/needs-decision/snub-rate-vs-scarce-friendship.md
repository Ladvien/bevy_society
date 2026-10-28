# Snub rate: 5:1 at 24 people vs scarce friendship at 60

Run `2026-10-27-07` (`eval.md`). Lowering the pack's `snub` rate (grudge curve `Linear(1.0,0.0)` → `Power(1.2)` or `Power(1.4)`) lifts the village from 2.9:1 to ≥ 18:1 compliments per insult (Baumeister et al. 2001's floor is 5:1) and drops rivals to 0-7. At 60 people over 30 days it also lets friendship saturate (90.8 % of ordered pairs, bar < 25 %) and rivals vanish, failing `tests/friendship_bound.rs` (issue #10). The two requirements pull the same knob opposite ways.

Options:
1. Keep `Linear` (status quo). 5:1 stays unmet at 24 people; the 60-person bar holds.
2. A friend-count-dependent brake that is not rivalry (Dunbar-style: `Dunbar` capacity trim already exists in `relationships.rs`; tighten it) so the snub rate can fall without friendship saturating. Needs its own research and test.
3. Relax `friendship_bound`'s `rivals >= 1`. Rejected: it guards the inversion of the network.

Recommendation: 2, as a separate pass, then return to the snub curve.
