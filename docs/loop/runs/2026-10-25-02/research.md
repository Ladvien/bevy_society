# Research: why a tie's label should not flip at a single line

## The observation that started this

`village-evening`, seed 7, 24 people, 4.1 game-days. Every `TieLabelChanged` was logged, and for each
one the quantity that caused it was read **at the moment of the change**:

| transition | n | margin from the bar that was crossed |
|---|---|---|
| `Acquaintance -> Friend` | 429 | p10 0.0000, median 0.043 |
| `Friend -> Acquaintance` | 404 | p10 0.0000, median 0.019 |
| `Acquaintance -> Rival` | 112 | p10 0.0024, median 0.016 |
| `Rival -> Acquaintance` | 93 | p10 0.0000, median 0.0001 |
| `Acquaintance -> Stranger` | 648 | p10 0.0000, median 0.0027 |
| `Stranger -> Acquaintance` | 96 | p10 0.0002, median 0.0002 |

**1 782 label changes, and 1 027 of them (58%) were within 0.01 of the bar.** The affinity in the tick
that flipped the tie had moved by a median of **0.00018** — the value was resting *on* the line and the
relaxation step nudged it across. These are not changes of relationship; they are a number crossing a
threshold it happens to sit on.

| | seed 7 | seed 42 | seed 11 |
|---|---|---|---|
| label changes, 4.1 game-days | 1 782 | 1 941 | 2 001 |
| within 0.01 of the bar | 1 027 (58%) | 1 090 (56%) | 1 078 (54%) |
| one-tick affinity step > 0.01 (a real event) | 739 (41%) | 806 (42%) | 873 (44%) |
| median game-hours between changes on a pair | 0.53 | 0.50 | 0.63 |
| pairs that changed more than 10 times | 28 | 21 | 19 |

A host reads `TieLabelChanged` to show "you are now friends", to unlock a conversation, to start a
quest. At 1 782 changes in four days a village of 24 emits one every **3.3 game-minutes**: the message
is unusable as an event, and the label it reports is not a stable property of the relationship.

## What the literature says the shape should be

**Social judgment theory** is the classical account of exactly this. An attitude is not a point; it is an
**anchor** with a **latitude of acceptance** around it and a **latitude of rejection** beyond it, and a
stimulus that lands slightly past the anchor's end point *shifts* the range rather than reversing the
judgement.

> "The 'latitude of acceptance' is the range of possibilities with which an individual is willing to
> agree or to which an individual can adapt. Latitudes are defined by end points, or anchors, that
> determine the extremes of the scale. ... According to social judgment theory, if stimuli continue to
> fall at the end of the continuum or even slightly above the end point, this will produce a shift of the
> range toward that anchor—or assimilation. However, if a stimulus is too far beyond the others, a
> contrast effect will ensue, and the stimulus will be perceived as being even more extreme than it really
> is."

— Fagan & Browne (1994: 219–220), reproducing Sherif & Hovland's (1961) account; corpus doc
`10.1007_0-306-47586-3_8`, *Responses of the Battered*, in the corpus.

Two things follow, and only the first is what this crate needs:

1. **A categorical judgement about a person has a band, not a line.** Between "friend" and "not friend"
   there is a range in which a person's judgement does not change. That is **hysteresis**: the state
   depends on the history, so the path up is not the path down. The band's two ends are the
   *promotion* bar and a *demotion* bar below it.
2. Judgements also **assimilate** — the band itself moves toward repeated experience. That is a model of
   attitude *change* and is deliberately **not** implemented here: it would make the bars themselves
   state, and the crate's bars are `Tuning` fields a host sets.

**Weak ties are the stable ones.** Igarashi, *Longitudinal Changes in Face-to-Face and Text
Message–Mediated Friendship Networks* (corpus doc `10.1017_cbo9780511894701.022`):

> "A 'superficial network' is composed of low intimate and weak ties that are **stable but changeable**."

That is the property the current implementation fails for the *low* end: `Stranger <-> Acquaintance`
churns 744 times in four days because familiarity decays continuously through a single line at 0.1. A
weak tie should be the most stable thing in the graph, not the least.

## What the paper does not supply

**The width of the band.** Neither source gives a number, and no source in the corpus gives an affinity
margin. So `Tuning::label_hysteresis` is a **design default chosen by measurement** (see `plan.md`), and
the code comment says so, which is the crate's rule.

## Why hysteresis rather than the alternatives

| shape | why rejected |
|---|---|
| **Debounce the message** (fire only after the label has held for N game-hours) | Changes when the host *hears* about a change, not whether there was one. The label still flips; a HUD reading the component still flickers. |
| **A single higher bar** | Raises the bar for entering *and* leaving. Measured the previous iteration the other way: it moves the population of friends, not the churn, and the value still rests on whatever line it is given. |
| **Smooth the inputs** (rate-limit affinity) | Changes the dynamics of the model to fix a measurement. Affinity is the thing being measured; it should not be filtered to make its label behave. |
| **Hysteresis** | Two bars for two directions, which is what "a latitude" *is*. Exactly zero behaviour change for a host that sets the field to `0.0`, so no existing pack moves. |

## What believable output looks like

- The number of `TieLabelChanged` messages in `village-evening` falls by **an order of magnitude**, and
  what remains is a real event: the one-tick affinity step is above the band.
- The **friend and rival counts do not move** — hysteresis must not create or destroy relationships, only
  stop them flickering across the line.
- A tie that has settled as a Friend stays a Friend while its affinity rests near the bar.
- `Stranger <-> Acquaintance` churn falls hardest, because that is the pair with the least margin
  (median 0.0027 and 0.0002).
- Determinism holds: no new draw, no new iteration order.
