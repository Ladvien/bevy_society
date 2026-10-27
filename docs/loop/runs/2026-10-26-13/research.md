# A deed seen by a crowd moves each watcher less — including its liking

Iteration `2026-10-26-13`.

## The pick

`src/reputation.rs::witness_reaction` dilutes a witnessed deed's **trust** cost by the size of the crowd
(`Tuning::witness_dilution`, 0.5) but writes **affinity** and **respect** at full magnitude for every witness.
Affinity is the variable that decides the friend and rival labels, so the crowd still multiplies the part of
a deed's cost that shapes the tie network, and only the part that does not shape it is damped.

Why this beats the runner-up (`DefsFile::merge` silently dropping `goods`, the log's #1): that is one line but
it is a schema question with no behavioural surface in the shipped pack, and it changes nothing a host can
watch. This one is the mechanism the last two iterations measured and could not fix, it is already half
implemented and cited, and it is the only change on the list that moves the village's friend:rival ratio —
18 friend pairs against 42 rival pairs at seed 7 — towards what the paper the crate already cites says a
society should look like.

## What the literature says

### Sommerfeld, Krambeck, Semmann & Milinski 2007 — the source of the dilution

DOI `10.1073/pnas.0704598104`, on disk at
`~/.cache/bevy_lights_corpus/markdown/10/10.1073_pnas.0704598104.md`. Verbatim:

- line 74: *"Further analysis documented a **dampening effect of this transmission** in a sense that originally
  negative behavior is not acted on as negatively and vice versa for positive behavior."*
- line 79: *"transmission by gossip had a dampening effect on the cooperation levels compared with acting on
  direct observation. **The more extreme a gossip was, the less impact it had on the reader.**"*
- line 81: *"Such access to multiple sources could reduce the dampening effect. If individuals hear several
  gossip comments about another person, the mean quality of these gossips ... might reflect the original
  behavior much more precisely than a single comment."*

The paper's dampening is about **how much a piece of news moves its recipient**, and it is stated for the whole
response — behaviour toward the person — not for one component of that response. The crate cited this paper for
"believed more, moving each recipient less" and then implemented "moving" as trust alone.

### Baumeister, Bratslavsky, Finkenauer & Vohs 2001 — what the balance should look like

DOI `10.1037/1089-2680.5.4.323`, already cited in the pack for `snub`:

- *"bad impressions and bad stereotypes are quicker to form and **more resistant to disconfirmation** than good
  ones"* — bad wins per event;
- *"**good may prevail over bad by superior force of numbers**: Many good events can overcome the psychological
  effects of a single bad one."*

So a believable society has **more** friendly ties than hostile ones, each individually more fragile. The
village has the opposite, and `needs-decision/good-is-outweighed-by-bad.md` (2026-12-28) established why: the
affinity ledger is **+160.4 from 1 604 accepted compliments against −241.2 from 7 920 witness reactions**,
i.e. bad is ahead 1.5:1 in total and good is outnumbered 4.9:1 in events. Good loses on numbers. The crowd is
where the numbers come from: a compliment writes affinity to **one** person, an insult writes it to **everyone
in perception**.

### The arithmetic, per deed

`witness_reaction` writes `0.15 · approval`, where `approval = norm · sign(harm)`. An `insult` is `harm −0.4`,
so a witness with a `kindness` opinion of `+0.5` writes **−0.075**. Six witnesses → **−0.45** of total affinity
for one deed. A `compliment` writes **+0.10** to one person. Under the shipped code one insult is worth **4.5
compliments** in total affinity; at the measured 4.9:1 event ratio the two sides are within noise of each other
and the tie network settles hostile, exactly as measured.

Diluting by `witnesses^0.5` — the exponent the field already defines — gives each of those six **−0.031**,
a total of **−0.184**, or **1.8 compliments**. Good then wins on the paper's own mechanism.

## The algorithm

`reputation.rs::witness_reaction`, unchanged signature:

```
seen   = witnesses^Tuning::witness_dilution          (already present, for trust)
gain   = trust_gain(tuning, conscientiousness) / seen  (unchanged)
Δaff   = 0.15 · approval / seen                        (NEW: the `/ seen`)
Δresp  = 0.1  · approval / seen                        (NEW: the `/ seen`)
```

Direction and the decision *whether* to object are untouched — the comment already says so for trust, and the
same sentence must now be true of all three: *"The dilution applies to the deed's magnitude, never to its
direction or to whether it is right to disapprove — a witness that objects still objects."*

`witnesses == 1` divides by 1, so a deed with one bystander is bit-identical to today.

## Parameters the paper reports, and the ones it does not

| parameter | value | source |
|---|---|---|
| crowd exponent | 0.5 | design default, already shipped for trust; `witness_dilution` |
| affinity per unit approval | 0.15 | design default, already shipped |
| respect per unit approval | 0.1 | design default, already shipped |
| `witness_gain` | 0.2 | design default, already shipped |

No new number is introduced. The change applies an existing, cited, measured field to the two writes that were
left out of it.

## What believable output looks like

- Friend pairs exceed rival pairs in `village-evening` at three seeds, with each friendship individually more
  fragile than today (the paper's asymmetry), rather than a village that settles at 2.3 rivals a friend.
- A single witnessed insult still moves each witness **negatively**, and `NormViolated` still fires for a
  disapproving witness — the direction is not diluted.
- One bystander is unchanged to the bit: the exponent is 1.0 there.
- Rivals do not fall to zero. A society with no hostility is as wrong as one with too much, and five earlier
  attempts (recorded in `friendship-has-no-diminishing-return.md`) each emptied a different end.

## Sources

- Sommerfeld, R. D., Krambeck, H.-J., Semmann, D. & Milinski, M. (2007). Gossip as an alternative for direct
  observation in games of indirect reciprocity. PNAS. https://doi.org/10.1073/pnas.0704598104 · on disk,
  lines 74, 79, 81 quoted above.
- Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001). Bad is stronger than good. Review of
  General Psychology. https://doi.org/10.1037/1089-2680.5.4.323 · already cited by the pack; the second claim
  quoted above.
- Nowak, M. A. & Sigmund, K. (1998). Evolution of indirect reciprocity by image scoring. Nature.
  https://doi.org/10.1038/31225 · the image-scoring mechanism `witness_gain` implements.
- Ohtsuki, H. & Iwasa, Y. (2004). How should we define goodness? Reputation dynamics in indirect reciprocity.
  J. Theor. Biol. https://doi.org/10.1016/j.jtbi.2004.06.005 · the `Standing` assessment the dilution must not
  disturb.
