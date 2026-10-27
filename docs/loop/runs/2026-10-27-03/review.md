# Review — a victim is moved by what was done to them

## Pass 1 — Behaviour

`evidence/plots/ratio-sweep.png`, `evidence/{before,after,sweep_*}/summary.json`. Seed 7, 390 accepted insults: victims
disliking their insulter **66 → 151 → 350** at ratio 0.0 / 1.0 / 5.0. The defect (a victim liking whoever insulted them,
+0.034 measured) is closed and the response is monotone in the field, so the field is read. Unit tests
(`reputation::patient_tests`) pin the sign in both directions, a benefit unscaled, an injury scaled by exactly the
ratio, ratio 0 off and 1 symmetric, and the ceiling clamp. The paper's direction (reciprocated negativity) and its
magnitude (5:1) are both satisfied.
Critique: the aggregate effect is large (below). The paper's *third* claim, good prevailing by numbers, holds only where
good outnumbers bad; in this pack it does not (compliments:insults ≈ 2.9:1, below Gottman's 5:1), and the model now
reflects that. **Verdict: GREAT** for the mechanism, with the pack-level consequence escalated, not hidden.

## Pass 2 — Zoom

The plot's right panel is the uncomfortable one: rivals 7 → 27, 1 → 35, 8 → 40; at seeds 7 and 42 rivals exceed friends.
Looked for artefacts: affinity pinned at −0.35 (ceiling) is plausible for one −0.30 write per insult at harm −0.4, and
`left_rival` is 41–66, so rivalries still dissolve (no permanent saturation). `became_rival` 121–124 across seeds is
nearly seed-independent, which says the pack's insult rate is driving it, not noise. Ratio 1.0 (uncited, symmetric)
already halves seed 7's friends, so the ratio is not the cause. The earlier version of the victims plot showed
empty "before" bars because the counter did not exist then; it was replaced with the ratio-0.0 sweep as before-state.
**Verdict: GREAT**, conditional on the escalation in `needs-decision/`; the magnitude is Gottman's threshold doing what
the paper says it does to a village below it.

## Pass 3 — Fresh eyes

Seed 23, 16 people (`evidence/fresh_eyes/`): victims disliking insulter 11 → 209 of ~260 insults; friends 18 → 12; rivals
1 → 17. Same direction as seeds 7/11/42, smaller population. Bram's narrative reads as a feud with Corin: compliment
refused, then a snub, then mutual insults in an hour, then Bram compliments Hilda — a stranger can follow it. A host
developer can be told it in a sentence: *the person an insult is done to now resents the insulter, five times as hard as
they would be pleased by the same size of kindness.* **Verdict: GREAT.**
