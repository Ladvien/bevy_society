# Research: a deed's hostility must not grow with the crowd

## Why this

Last iteration's top remaining item was the price model. This iteration was going to take it. The evidence sent
it here instead: the same run found **rivalries exploding with population** — 17 pairs at 24 people, 128 at 36,
219 at 48 — and a price that clips its floor is a cosmetic defect next to a village where a fifth of all pairs
hate each other. "Behaviour that reads as wrong when you watch it" outranks a model that is merely imprecise.

## The measurement that chose this

`village-evening`, seed 7, 17 game-days, three populations, same pack:

| people | directed pairs | friend pairs | rival pairs | mean witnesses per deed |
|---|---|---|---|---|
| 24 | 552 | 22 | 17 (3.1%) | 12.6 |
| 36 | 1 260 | 18 | 128 (10.2%) | 21.5 |
| 48 | 2 256 | 28 | 219 (9.7%) | 30.0 |

Rivalry density triples between 24 and 36 people and then holds, while the rate of deeds per person per day is
roughly flat (35.7 → 59.0 → 81.1 insults a run, i.e. 1.5 · people · days). What rises is the **crowd**: a deed in a
denser village is seen by 1.7x and 2.4x as many people. That is the multiplier.

## What the code does with the crowd

`src/reputation.rs::witness_reaction`, once per witness:

```text
seen  = witnesses ^ Tuning::witness_dilution     (0.5, a square root)
trust = trust + trust_gain(conscientiousness) * harm / seen     <-- diluted
affinity = affinity + 0.15 * approval                          <-- NOT diluted
respect  = respect  + 0.10 * approval                          <-- NOT diluted
memory   = record(importance = |harm|)                         <-- NOT diluted
```

`witness_dilution` was added an iteration ago for exactly this reason, and its own doc says what it is for: "a
deed's total cost grew linearly with the crowd while a trust *gain* from an interaction could only ever reach two
people, so a village's trust could only fall". The author measured **trust** (0.0800 → 0.0431 with five bystanders)
and stopped there. `affinity` is what `TieLabel` calls a rivalry, and it was left undiluted, so **three of the
four effects of witnessing still scale with the crowd.** The 128-rival village is the consequence.

So this is not a tuning problem. It is one effect of an event using a dilution the other three ignore, which the
crate's own comment calls inconsistent.

## The literature

**Latané, B. (1981), "The psychology of social impact", *American Psychologist* 36(4): 343-356, DOI
`10.1037/0003-066X.36.4.343`.** Social impact theory states that the impact of a social force on a person is a
**multiplicative** function of strength, immediacy and **number**:

> Impact = f(S · I · N)

and, of the number of sources, the impact of the *N*th source is smaller than that of the *(N−1)*th — an
**explicitly diminishing** returns law, usually written as a power function `N^α` with `α < 1`. The theory's
own summary is that "the impact of any source is a function of ... its number", and Latané's data put the
exponent near **0.5 for the number of sources**.

That is the shape the crate already uses for trust (`witnesses ^ 0.5`) and it is the shape the other effects
should use, from one paper rather than one effect at a time. `witness_dilution` is therefore not a knob invented
for trust: it is Latané's exponent, and its current doc credits the wrong paper (Sommerfeld et al. 2007 for
gossip *transmission*). Sommerfeld is the right source for believing a message more when more sources tell it;
Latané is the right source for the *impact* of a crowd on a bystander.

**Sommerfeld, Krambeck, Semmann & Milinski (2007), PNAS, DOI `10.1073/pnas.0704598104`** stays as the
reputation-transmission source and is already cited in `tests/witness_dilution.rs`. It does not cover this.

## What a believable output looks like

- **Density, not count.** Rivalry density stays in a band as population rises: the 24-person village's 3-5% of
  directed pairs should hold at 36 and 48, so a host can grow its village without it turning into a feud.
- **Both sides of a deed diluted alike.** A crowd should not make each bystander *angrier* than a lone witness;
  it should make each of them less moved and **more of them** moved (Latané's "more sources, less each").
- **Rivalries still form and still reconcile.** A fix that empties the hostile half of the graph is a regression,
  not a fix: the 24-person village had 17 rival pairs and 69 norm violations, and both must survive.
- **No change to the friend count beyond noise**, and none to rumor reach or interruptions.

## The parameters

- `Tuning::witness_dilution` (0.5) is kept and becomes the **exponent for every per-witness effect**, not just
  trust. Latané reports ~0.5 for number of sources; the crate already ships 0.5.
- No new rate. Every effect keeps its own weight (`0.15` affinity, `0.10` respect); only the divisor changes.
