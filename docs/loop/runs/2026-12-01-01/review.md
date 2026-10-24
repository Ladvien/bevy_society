# Review

## Pass 1 — Behavior: does the offset do what the paper says?

**GREAT.** Cacioppo et al. 2013 describe the positivity offset as *"the approach motivation at zero input"*
being higher for the positive channel than the negative — the **intercept**, as distinct from the gain
(negativity bias). The crate had the gain and no intercept; this adds the intercept at exactly the point the
model calls "zero input", the resting point `Mood::step` pulls toward.

| check | result |
|---|---|
| idle person, offset 0.0 | pleasure **+0.0000** |
| idle person, offset 0.1 | pleasure **+0.0122** |
| village max pleasure, offset 0.0 | **+0.0000** |
| village max pleasure, offset 0.1 | **+0.0155** |
| village max pleasure, offset 0.3 | **+0.0476** |

Monotonic, and it crosses zero for the first time in the crate's history of this scenario.

## Pass 2 — Zoom: read the plot closely

**GREAT**, and the plot's honesty is the point. `evidence/positivity-offset.png`:

- **Left:** three traces, cleanly separated by the offset, with the axis drawn at zero. All three still
  oscillate around a negative value — the offset moves the whole band up, it does not invert it.
- **Right:** the mean bars stay below the axis at every offset while the **max** bar rises from exactly
  +0.000 through +0.015 to +0.048. That is the ceiling moving and the centre not.

No clipping at ±1, no oscillation at a tick boundary, no dead agents. The dilution is visible: offset 0.3 is
only 3× offset 0.1 in the mean (−0.058 against −0.122), not 3× in effect, because the emotion pull scales
with intensity.

## Pass 3 — Fresh eyes: is the number defensible, and can it be turned off?

**GREAT.** Two things a skeptical host would ask, both answered:

1. **Is 0.1 arbitrary?** Yes, and the doc says so — the paper states the offset's existence and sign, not a
   magnitude. The comparison at 0.3 is shipped as evidence, so a host can see the direction and pick.
2. **Can I have the old behaviour?** Yes: `positivity_offset: 0.0` reproduces it exactly, and
   `tests/mood_offset.rs::a_zero_offset_leaves_the_old_behaviour` pins that.

The narrative to a host:

> An idle person now rests mildly pleased instead of exactly neutral. The affect model you were given had the
> half of Cacioppo's pair that makes bad news hit harder, and not the half that makes an empty moment
> slightly good — so pleasure could never be positive. Set `positivity_offset: 0.0` for the old behaviour.

## Honest limit, stated in the log and the eval

This fixes the **ceiling**, not the **mean**. A saturated negative emotion centre dilutes the offset in
proportion to intensity. The appraisal balance — 3:1 distress to joy — is the remaining lever, and it stays a
decision rather than being tuned here.

## Verdict

Three GREAT. Shipped.
