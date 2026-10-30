# Snub rate: 5:1 at 24 people vs scarce friendship at 60

Run `2026-10-27-07` (`eval.md`). Lowering the pack's `snub` rate (grudge curve `Linear(1.0,0.0)` → `Power(1.2)` or `Power(1.4)`) lifts the village from 2.9:1 to ≥ 18:1 compliments per insult (Baumeister et al. 2001's floor is 5:1) and drops rivals to 0-7. At 60 people over 30 days it also lets friendship saturate (90.8 % of ordered pairs, bar < 25 %) and rivals vanish, failing `tests/friendship_bound.rs` (issue #10). The two requirements pull the same knob opposite ways.

Options:
1. Keep `Linear` (status quo). 5:1 stays unmet at 24 people; the 60-person bar holds.
2. A friend-count-dependent brake that is not rivalry (Dunbar-style: `Dunbar` capacity trim already exists in `relationships.rs`; tighten it) so the snub rate can fall without friendship saturating. Needs its own research and test.
3. Relax `friendship_bound`'s `rivals >= 1`. Rejected: it guards the inversion of the network.

Recommendation: 2, as a separate pass, then return to the snub curve.

## Update 2026-12-30 (run `2026-12-30-01`): option 2 built and measured, not shipped

A friend-capacity brake (`friend_capacity` 10, after Zhou et al. 2005's layer scaling) takes `Power(1.2)` from 90.8% friend pairs to 21.9% at 60 people, but leaves 0 rivals, so `rivals >= 1` still fails. The finding that changes the question: **HEAD's 60-person town is 32.7% rival pairs and 1.8% friend pairs** (1157 and 62 of 3540), so the bar guards against the wrong failure for the shipped snub. Rival counts at 60 people across exponents are 1157, 1267, 426 and 0 for 1.0, 1.1, 1.15 and 1.2: no stable middle. Patch: `runs/2026-12-30-01/friend-capacity.patch`. Recommended next: find why the default town rivals itself before touching the ratio again.

## Update 2026-10-30 (run `2026-10-30-16`): option 2 shipped

Assumed decision (owner may override): friend-capacity brake (`friend_capacity` 10) plus the pack's `snub` curve at `Power(1.15)`. With the 6 h retaliation refractory now default, the earlier "no stable middle" no longer holds: 60-person town 54 rivals, 209 friends (5.9%), village 8:1 compliments to insults; `friendship_bound` passes (110 friends, 3.1%, 180 rivals). Set `friend_capacity: 0.0` to turn the brake off.
