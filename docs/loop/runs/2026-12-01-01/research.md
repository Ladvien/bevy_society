# Research: the affect system had a gain and no intercept

## The finding this picks up

Last iteration measured it and escalated it: in `village-evening` the **maximum pleasure over every person
and every tick is exactly `+0.000`**, stdev 0.031, for 17 game-days. Nobody is ever pleased.

`Mood::step` pulls toward the emotion centre at `pull_per_day = 4.0` and returns to the personality baseline
at `return_per_day = 1.0`. `pad::baseline` is **zero** for a sampled personality, because
`Personality::sample` averages three draws and traits centre on 0.5, and Mehrabian's mapping is linear with
its zero at 0.5. So pleasure's ceiling is exactly the baseline: **zero**.

## The model that names the missing term

**Cacioppo, Cacioppo & Gollan 2013**, *The negativity bias: Conceptualization, quantification, and individual
differences*, DOI `10.1017/S0140525X13002537`, in the corpus:

> "the intercept for the positive activation function (i.e., the approach motivation **at zero input**) is
> higher than the intercept for the negative activation function (producing the **positivity offset**), and
> the gain for the negative activation function is higher than for the positivity activation function
> (producing the **negativity bias**). The consequence of the positivity offset is that the motivation to
> approach is stronger than the motivation to withdraw at very low levels of evaluative activation (thereby
> motivating exploratory behavior), whereas the consequence of the negativity bias is that the motivation to
> withdraw is stronger than the motivation to approach at high levels of evaluative activation."

Two mechanisms, and the crate had **only one of them**:

| mechanism | in the crate |
|---|---|
| negativity **bias** (steeper gain for the negative channel) | yes — a landed insult pays Anger 0.5 against a compliment's Joy 0.3, and distress outnumbers joy 3:1 by count |
| positivity **offset** (higher intercept for the positive channel) | **no** |

The offset is exactly "the approach motivation at zero input", which is the point `Mood::step`'s return term
pulls toward. So the fix is one term on that resting point, not a new model.

## What believable output looks like

A person with no emotions at all rests **mildly pleased** rather than neutral, and a village's pleasure
crosses zero sometimes rather than never.

## What the offset can and cannot do

Steady state of `x' = x + (c − x)·pull·I + (rest − x)·back` is

```
x = (pull·I·c + back·rest) / (pull·I + back)
```

so the offset **tilts the resting point** but is diluted by the emotion pull in proportion to intensity:

| intensity `I` | centre `c` | offset 0.1 | offset 0.3 |
|---|---|---|---|
| 0.0 | — | +0.100 | +0.300 |
| 0.2 | −0.5 | −0.167 | −0.056 |
| 0.5 | −1.0 | −0.633 | −0.567 |

A saturated negative centre pins pleasure near −0.6 whatever the offset. So the offset fixes the **ceiling**
and not the mean, and the mean needs the appraisal balance — which is `a-village-that-is-never-pleased.md`'s
option 1, and remains open.
