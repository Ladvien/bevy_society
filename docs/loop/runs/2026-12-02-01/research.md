# Research: pleasant conversation was affectively silent

## The finding this picks up

Last iteration added the positivity **offset** and measured that it fixes the *ceiling* and not the *mean*:
mean pleasure stayed at -0.122. The remaining lever, named in
`a-village-that-is-never-pleased.md`, is the appraisal **balance**: 1 302 distress appraisals against 427 joy.

Measuring where that ratio comes from gave an exact answer, and it is not a rate:

| | count |
|---|---|
| refusals | 1 216 |
| **distress appraisals** | **1 216** |
| accepted interactions | 1 338 |
| positive appraisals | **443** |

Every refusal produces exactly one distress appraisal. Acceptances do not: **1 338 of them produce 443
positive appraisals**, because only `compliment` pays an emotion and it pays only its *target*.

## The mechanism, in the pack

The pack's four interactions, with what each pays on **accept**:

| interaction | accepted | refused | emotion on accept |
|---|---|---|---|
| `gossip` | 420 | 528 | **none** |
| `insult` | 324 | 395 | target `Anger 0.5` |
| `compliment` | 375 | 231 | target `Joy 0.3` |
| `chat` | 106 | 163 | **none** |

**43% of accepted interactions pay nothing to either party's mood**, and they are the two most common. The
actor who initiates a pleasant exchange gains no affect from it in any interaction at all — `compliment`
pays only the person receiving it.

## What the literature does and does not ask for

**Cacioppo, Cacioppo & Gollan 2013**, DOI `10.1017/S0140525X13002537` — already the crate's citation for the
positivity offset — give the negative channel the **steeper gain** (negativity bias) and the positive channel
the **higher intercept** (positivity offset). So a negative event outweighing a comparable positive one is
*correct*: `Anger 0.5` against `Joy 0.3` is a bias, and should stay.

What the paper does not give either channel is **silence**. A pleasant conversation is an appraisal in OCC
terms — the crate maps `Liking` to the pleasure axis at `+0.40` — and a pack where 43% of them produce no
emotion is not modelling a bias. It is declining to model positive affect at all.

## Why `Liking` and not `Joy`

OCC's *joy* is a prospect emotion about an event; what a conversation gives is liking for the person one had
it with. `Liking` is already in the crate's `EmotionKind` set and already feeds the pleasure axis.

## What believable output looks like

A society where pleasant interactions register: positive affect present in the steady state, and mood able
to sit near neutral rather than pinned negative. A bias, not a silence.
