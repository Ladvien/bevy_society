# Eval

## Change

**None.** Three levers measured, all rejected. The deliverable is a quantified diagnosis with a citation.

## The finding

The crate cites **Baumeister et al. 2001** (DOI `10.1037/1089-2680.5.4.323`) for `snub`'s grudge mechanism. The
review makes two claims:

1. bad beats good **per event** — "quicker to form and more resistant to disconfirmation";
2. good wins **in aggregate** — "by superior force of numbers: Many good events can overcome the psychological
   effects of a single bad one."

The crate implements (1) and violates (2). Measured:

| event | affinity effect | reach |
|---|---|---|
| `compliment` | **+0.10** | one person |
| `insult` (harm `-0.4`) | **-0.08** per witness | every witness |

Six witnesses in perception means one insult is **-0.48 of total affinity**; bad outweighs good **5:1 per event**.
`compliment` holds **11.3%** of person-ticks against `snub`'s **3.8%** — **3:1 by count**. Outnumbered 3:1 and
outweighed 5:1, good cannot accumulate: the village settles at **18 friend pairs against 42 rival pairs**.

## The three levers

| lever | rivals | friends | verdict |
|---|---|---|---|
| symmetric decay (shipped) | 42 | 18 | — |
| negative affinity fades 2x slower | **82** | 11 | rejected |
| `compliment` +0.10 -> +0.20 | 31 / 44 | 14 / 7 | inconsistent |

The first was my prediction and it is **backwards**: with structural balance disabled the result is still 64, so
balance explains only 18 of the 40. Persisting grudges persist; friendship fades at the same rate, so the
counterweight does not grow.

## Why the paper's asymmetry applied alone fails

The paper separates the two effects deliberately. Making the *bad side* stickier without also raising the
*frequency* of good is implementing half a theory — it amplifies the half that already dominated.

## Gates

| Command | Result |
|---|---|
| `git diff --stat src/ assets/` | **empty** |
| `cargo test` (baseline before any lever) | **71** binaries |
| `cargo build` after each revert | 0 errors |

## What a host gets

Not a fix — a measurement and a citation that says the starter pack contradicts its own literature. Option 3 in
`needs-decision/good-is-outweighed-by-bad.md` is the one that follows the paper; it needs a three-axis
measurement before shipping, and it is new authored content, so it is not a unilateral change.
