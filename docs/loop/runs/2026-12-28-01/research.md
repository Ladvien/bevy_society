# Research: good is outweighed by bad

## Why this

Last iteration's top item: "separate forgetting rates for positive and negative affinity — the only shape that
fixes the rival/friend ratio without emptying the village. Needs a cited default." I went to the corpus for the
default. What I found changed the diagnosis.

## The citation

**Baumeister, Bratslavsky, Finkenauer & Vohs 2001**, *Bad Is Stronger Than Good*, DOI
`10.1037/1089-2680.5.4.323` — already cited in the crate, in `snub`'s consideration. Two findings, and the crate
implements the first while contradicting the second:

> "bad impressions and bad stereotypes are quicker to form and **more resistant to disconfirmation** than good
> ones"

> "**good may prevail over bad by superior force of numbers**: Many good events can overcome the psychological
> effects of a single bad one."

The paper explicitly separates **per-event weight** (bad wins) from **aggregate outcome** (good wins by count).
A believable village therefore has *more* friendly ties than hostile ones, each individually more fragile.

## What the crate does

`village-evening`: **18 friend pairs, 42 rival pairs**. Fewer friendly ties, and symmetric decay. Both halves of
the paper's account are inverted.

## The arithmetic

| event | affinity effect | reach |
|---|---|---|
| `compliment` | **+0.10** | one person |
| `insult` (deed, harm `-0.4`) | **-0.08** per witness (`witness_gain 0.2 x -0.4`) | **every witness** |

With roughly six people in perception, one insult moves **-0.48** of total affinity against **+0.10** for a
compliment — bad outweighs good **5:1 per event**. Meanwhile `compliment` occupies **11.3%** of person-ticks
against `snub`'s **3.8%**, about **3:1**. Good is outnumbered 3:1 and outweighed 5:1, so it cannot accumulate.

**That is the whole mechanism**, and it is a data fact in `assets/starter.society.ron`, not a rate in `Tuning`.

## The three levers, measured

| lever | rivals | friends | verdict |
|---|---|---|---|
| symmetric decay (shipped) | 42 | 18 | — |
| grudge fades 2x slower | **82** | 11 | **rejected** |
| `affinity_relax_per_day` 0.03 | 10 | 5 | rejected last iteration |
| `compliment` +0.10 -> +0.20 | 31 / 44 across seeds | 14 / 7 | **inconsistent** |

The asymmetric decay was my prediction and it is **backwards**. Isolating it: with `balance_rate_per_day = 0` the
result is still 64 rivals, so structural balance explains 18 of the 40 — the rest is direct. Persisting grudges
persist, and friendship decays at the same rate, so the counterweight is unchanged.

## Conclusion

The fix is not decay and not per-event weight. It is the **ratio of events**, which is what the paper says decides
the aggregate. Recorded with three options in
`docs/loop/needs-decision/good-is-outweighed-by-bad.md`; option 3 (raise the *rate* of friendly interaction, not
its weight) is the one that follows the paper rather than fighting it, and it needs a three-axis measurement
before shipping.

## Sources

- Baumeister, Bratslavsky, Finkenauer & Vohs 2001, *Bad Is Stronger Than Good*, DOI `10.1037/1089-2680.5.4.323`.
  Corpus doc `10.1037_1089-2680.5.4.323`, pp. 1, 33, 38.
