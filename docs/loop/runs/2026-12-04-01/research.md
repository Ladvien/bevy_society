# Research: a crowd multiplied a penalty and could not multiply a reward

## The state this picks up

Last iteration shipped `TieDelta::trust` on successful interactions and measured that it was **not enough**:
mean trust at an exchange went 0.261 → 0.345 against a neutral of 0.5, and trust in `village-evening` still
declined over a run. The reason was identified and escalated:

> A trust *gain* from an interaction goes to the **two parties**. A trust *loss* from a witnessed deed goes to
> **every witness within perception**, at full magnitude.

`witness_reaction` applies `trust_gain · harm` per witness, with `witness_gain = 0.2`. One `steal`
(`harm -0.7`) is **-0.14 per witness**. So a chat is `+0.10` in total while one insult seen by three people is
`-0.42`.

## The decision file said "recommend dilution" — so I looked for a source

The recommendation was mine, from symmetry. Whether the literature agrees is a different question, and it does.

**Sommerfeld, Krambeck, Semmann & Milinski 2007**, *Gossip as an alternative for direct observation in games of
indirect reciprocity*, PNAS, DOI `10.1073/pnas.0704598104` — already cited by this crate for the gossip
mechanism. Measured directly:

> "transmission by gossip had a **dampening effect** on the cooperation levels compared with acting on direct
> observation. The more extreme a gossip was, the less impact it had on the reader."

and, of how many sources an individual hears from:

> "Hess and Hagen found an increase in **believability** with an increasing number of sources."

**Believed more, moving each recipient less.** That is precisely a per-witness dilution: information from a
crowd is more credible in aggregate and weights less in each head. The paper does not say a deed's *total*
reputational cost is constant — it says the transmission dampens, which is what a per-recipient divisor models.

So dilution is not a symmetry argument. It is the measured behaviour of the mechanism this crate already cites.

## What believable output looks like

A village whose trust can recover from its disagreements: rivals falling as trust rises, and a mean trust that
is not pinned below the neutral point `AcceptDef` uses.

## The design

`Tuning::witness_dilution` is the **exponent** of the witness count:

```
gain = trust_gain(tuning, conscientiousness) / (witnesses ^ witness_dilution)
```

| value | behaviour |
|---|---|
| `0.0` | the old behaviour: full cost per witness, total grows linearly with the crowd |
| **`0.5`** | square-root dilution — the default |
| `1.0` | the total cost is constant however many saw it |

`0.5` rather than `1.0` because the paper's dampening is partial: many sources make a claim *more* believable,
so the total should still grow, just sub-linearly.
