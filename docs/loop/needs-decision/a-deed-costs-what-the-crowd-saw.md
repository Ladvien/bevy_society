# Should a witnessed deed cost the same for every witness?

> **Resolved 2026-12-04: no, and the literature settles it.** Sommerfeld, Krambeck, Semmann & Milinski 2007
> (DOI `10.1073/pnas.0704598104`) measured that gossip transmission *dampens* — *"the more extreme a gossip
> was, the less impact it had on the reader"* — and cited Hess & Hagen on believability rising with the number
> of sources. Believed more, moving each recipient less: a per-witness dilution. `Tuning::witness_dilution`
> (default 0.5, a square root) implements it. Measured, one witnessed insult now costs each witness **0.0800**
> with one bystander and **0.0431** with five, where before it was 0.0800 in both. The analysis that prompted
> the question follows.

## The state, measured

A trust *gain* from an interaction goes to the **two parties**. A trust *loss* from a witnessed deed goes to
**every witness within perception**, at full magnitude, with no dilution.

```rust
// reputation.rs, witness_reaction
tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);   // gain = witness_gain * (0.5 + conscientiousness)
```

`witness_gain` is 0.2 and a `steal` is `harm: -0.7`, so **one witnessed steal is -0.14 per witness**. Measured
interaction mix over 12 000 ticks with 24 people:

| interaction | accepted | trust effect |
|---|---|---|
| `chat` | 78 | **+0.05** to each of two people |
| `compliment` | 258 | +0.02 to the target |
| `gossip` | 225 | **+0.03** to each of two people |
| `insult` | 189 | **-0.14 per witness** |

A chat is `+0.10` in total. One insult seen by three people is `-0.42`. **The loss is multiplied by the size
of the crowd; the gain never is**, so a village's trust has a permanent downward pressure that scales with
density — and density is what makes a village a village.

This is why the pack's trust still falls even after every accepted interaction gained a `TieDelta::trust`:
measured, mean trust at the moment of an exchange went **0.261 → 0.345** and the accept rate **0.486 → 0.533**,
which is a real improvement and not enough. Trust in `village-evening` still declines over a run.

## Why it is a decision

Three defensible answers:

### 1. Dilute a witnessed deed by the number of witnesses

`gain * harm / sqrt(witnesses)`, or a fixed budget split among them. Reputation damage is partly about *how
many people know*, so a deed seen by ten is worse in total than one seen by two — but not ten times worse for
each individual.

- **For:** makes trust survivable in a crowd without weakening the deed itself; one number.
- **For:** matches how gossip actually scales — the *news* spreads, the individual outrage does not multiply.
- **Against:** changes `witness_reaction`'s contract, which `docs/research/rumor-and-reputation.md` documents.

### 2. Cap the total trust a single deed can remove village-wide

`harm` is applied to each witness until a total budget is spent, then further witnesses get less.

- **For:** bounded damage per deed.
- **Against:** order-dependent — which witnesses are "first" is the iteration order, and that is exactly the
  kind of hidden dependence the determinism rules exist to prevent.

### 3. It is correct, and the pack should make good deeds witnessed too

The pack uses `insult` and never uses `help` or `gift`; a deed is only costly because nothing offsets it.

- **For:** no model change; the asymmetry is the negativity bias working as Cacioppo et al. describe.
- **Against:** measured, an attempt to use `help` made it either inert (1 accepted in 12 000 ticks at
  `Affinity > 0.8`) or erased all conflict (rivals **44 -> 0**, friends **21 -> 81** at `Affinity > 0.55`).
  There is no gate between those that produces a believable village, which suggests the deed *density* is not
  the lever.

## Recommendation

**Option 1**, because it is the only one that addresses the actual asymmetry — that a crowd multiplies a
penalty and cannot multiply a reward. It is one number and it stays deterministic.

## What ships now

`chat`, `gossip` and `compliment` carry `TieDelta::trust` so a **successful** exchange can raise trust at all;
before this, the starter pack set no trust delta anywhere and the only mover was the witnessed deed. Measured
with a controlled comparison (same pack, deltas stripped): mean trust **0.261 → 0.345**, accept rate
**0.486 → 0.533**. That is the part that was unambiguously a defect — a term in `AcceptDef` centred on 0.5
that nothing in the pack could raise.
