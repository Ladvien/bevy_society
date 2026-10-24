# Research: leisure needs a companion, not a bigger number

## Where this starts

Last iteration closed with a measured negative result: raising `relax`'s rate to cover a day of `fun`
decay halves `fun`'s zero fraction (33.8% → 18.8%) but **lowers** the mean of all four needs
(0.127 → 0.118). The village is over-subscribed — 24 people, four places, one evening — so a bigger
number only moves starvation around. The decision file said the fix is not a rate.

## Why a second, cheaper source is the right shape

`fun` and `social` sit at opposite ends of the same budget:

| need | uses/day needed | actual supply | cost of one use |
|---|---|---|---|
| social | 2.78 (`chat`) | **ample** | 3 ticks; the other person walks to you |
| fun | 1.33 (`relax`) | **0.10** | a walk plus 16 ticks, inside a 6-hour window |

`social` needs *more* uses a day than `fun` and is never short, and the difference is not the rate — it
is that a `chat` targets a `Person`. A `Person` target costs one decision; a `Place` target costs a
decision, a walk, and the competition of everyone else wanting the same well.

So the fix is a `Person`-targeted leisure activity, not a larger `relax`.

## Literature

Self-determination theory is the crate's frame for needs (already cited at `Tuning` and in
`docs/research/needs-and-utility.md`), and it says where intrinsic motivation comes from:

> "the satisfaction of the psychological needs for autonomy, competence, and relatedness contributes to
> pro-activity, integration, and well-being"
> — Vansteenkiste & Ryan 2013, DOI 10.1037/a0032359

Relatedness is one of the three needs, and it is *satisfied through other people*. "Individuals can be
vital, open, curious and caring" (same paper) is the state a society is in when its needs are met, and
the paper's mechanism for reaching it is need-supportive *social* context.

A leisure activity that is done **with someone** is therefore the reading of the paper that makes `fun`
a social need rather than a solitary one. `relax` at a well alone is the solitary form and stays; `play`
with another person is the form the theory describes.

No paper gives the rate. `1.7 per game-hour × 0.5 h = 0.85` per use against `fun`'s 0.8/day is a **design
default**, stated as such in the file, sized the same way `relax`'s rate would have to be — one session
covering a day — but attached to a source that costs a `chat`'s three ticks.

## What believable output looks like

`fun` should stop sitting at zero and should oscillate like every other need does: peaks from a session,
troughs from decay, never a flatline. Measured after the change, at the same seed and layout:

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **1.3%** |
| `fun` mean | 0.061 | **0.112** |
| mean of all four needs | 0.127 | **0.128** |

The total is unchanged, which is the point last iteration's rate change missed: this reallocates
nothing, it adds a way to meet the need that does not consume the evening.

## The three other needs pay a little

`social` 0.124 → 0.107, `rest` 0.221 → 0.212, `food` 0.101 → 0.080, because `play` competes with `chat`
and `eat` for the same hours. That is a real cost and it is reported rather than hidden; the total is
flat, and the need that was at zero for a third of every run is now at zero for 1%.
