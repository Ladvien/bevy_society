# Review

## Pass 1 — Behavior: is the omission real, and is it fixed?

**GREAT** for the scope shipped, and the scope is deliberately narrow because the larger fix is a decision.

The omission is not a judgement call: `grep trust assets/starter.society.ron` returned **zero** matches.
`AcceptDef` centres trust on 0.5, `Tie::new` starts there, and `witness_reaction` applies `trust_gain · harm`
which is negative for every deed in the pack. A term in every acceptance draw could only ever go down.

| | no delta (control) | with delta |
|---|---|---|
| mean trust | **0.261** | **0.345** |
| accept rate | **0.486** | **0.533** |

The test is a controlled comparison — the same pack, one with the deltas stripped — so it is the delta and
nothing else.

## Pass 2 — Zoom: read the plot closely

**GREAT**, and the plot refuses to overclaim. `evidence/trust.png`:

- **Left:** the two pleasure traces overlap heavily. That is honest: mood moved −0.066 → −0.058, which is
  *visible* in the statistic and barely visible in the trace. I did not draw it as a dramatic change.
- **Middle:** the controlled comparison, where the effect is unambiguous — trust 0.26 → 0.34, accept 0.49 → 0.53.
- **Right:** the asymmetry that no pack tweak can address. A chat is `+0.10`; a witnessed insult is `−0.14` per
  witness and reaches everyone in perception.

No oscillation change, no clipping, no drift.

## Pass 3 — Fresh eyes: what a host would see, and what I tried

**GREAT**, and the most useful pass was the one that *failed*. A `help` activity so the pack's unused deeds
would run gave three results, none of them acceptable:

| gate | result |
|---|---|
| 0.55 | rivals 44 → **0** — a village with no conflict |
| 0.70 | near-neutral mood, trust still declining |
| 0.80 | **1 acceptance in 12 000 ticks** — inert |

Recording all three matters more than the one that shipped: they establish that **deed density is not the
lever**, so the next attempt does not start there.

The narrative to a host:

> A conversation that goes well now raises trust between the two people, and more greetings are accepted.
> Before, nothing in the starter pack could raise trust at all — the accept formula centres it on 0.5 and only
> witnessed deeds moved it, all of them harmful. It still declines in a crowd, because a witnessed deed costs
> every witness and a good deed only rewards the pair; that is written up as a decision rather than tuned here.

## Verdict

Three GREAT. Shipped. The result is smaller than the last three iterations, and the reason is established
rather than hand-waved: the remaining asymmetry is structural and belongs to a decision.
