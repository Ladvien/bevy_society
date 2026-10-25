# Research: why a friendship ends after four hours

## The observation

`village-evening`, seed 7, 24 people, 16.4 game-days. Every `Friend` transition was counted and every
friendship's affinity was traced:

- **1 027 `Friend` messages** — 523 becoming a friend, 504 ceasing to be one.
- Friendships are held for a **median of ~4 game-hours** before the label is lost.
- A single tie's affinity swings a **median of 0.072** over the run (p90 0.240, max 0.528).
- The **friend bar is `friend_affinity` 0.30, the ceiling `affinity_ceiling` 0.35.** The whole band in
  which a tie can be a friend is **0.05 wide** — narrower than a typical tie's own swing.
- **7% of all positive-affinity tie-samples sit inside that 0.05 sliver.** It is not a tail; it is a
  bucket in the histogram, and the histogram has a second peak exactly there.
- The affinity change in the tick a friendship ends is **−0.0500 for the median and the 10th percentile
  alike** — a quantised step, not a gradual decay.

So a village of 24 people creates and destroys a friendship about **63 times a game-day**, and the
mechanism is not that people fall out. It is that the label's only home is a 0.05-wide shelf on a
variable that moves 0.07 at a time.

## What actually ends a friendship

Attributed by looking backwards from each ending (seed 7, 504 endings; seed 42, 507):

| cause, within 6 game-hours | seed 7 | seed 42 |
|---|---|---|
| a witnessed `insult` involving one of the pair | 122 | 166 |
| any exchange with that person (accept or refuse) | 326 | 289 |
| neither | **56** | **52** |

Two things follow, and they point in opposite directions.

1. **Some endings are real events.** An insult is a legitimate reason to stop calling someone a friend,
   and the median is 0.4 game-hours after it — prompt and explainable.
2. **A third of the endings have no interaction behind them at all.** Those 56 and 52 are the label
   crossing the bar from continuous drift: relaxation toward zero (`affinity_relax_per_day` 0.01),
   structural balance, similarity, exposure. That is not a relationship event and a host should not be
   told about it.

## What the literature says a friendship's shape is

- **Altman, I. & Taylor, D. A. (1973).** *Social Penetration: The Development of Interpersonal
  Relationships.* Read via Battle & White (2024), *Intimate Introverts: Influence of Introversion on
  Self-Disclosure and Emotional Intimacy in Close Friendships* (DOI `10.33790/jmhsb1100188`, in the
  corpus):

  > "According to social penetration theory, friendships go through four stages of development:
  > orientation, exploratory affective exchange, affective, and stable exchange ... **Maintenance of
  > friendships is described differently depending on whether the friend is considered 'casual,'
  > 'close,' or 'best.'** 'Casual' friendships rely mostly on proximity, while 'close' and 'best'
  > friendships rely more on increased affection and interaction."

  Two claims are useful here. Friendships are **staged**, so the categories are not merely thresholds on
  one variable; and a friendship is **maintained, not re-earned** — the maintenance behaviours are what
  keep it, and their absence is what ends it. That is the shape the crate lacks: a `Friend` is a
  classification of the current affinity, so it has no memory of having been one.

- **Igarashi, T.** *Longitudinal Changes in Face-to-Face and Text Message–Mediated Friendship Networks*
  (corpus doc `10.1017_cbo9780511894701.022`), read last iteration for the low end and equally relevant
  here:

  > "A 'superficial network' is composed of low intimate and weak ties that are **stable but
  > changeable**."

  A weak tie is stable. A close tie, on the evidence above, is in this crate far *less* stable than a
  weak one — 504 friendships ended in 16 days while the low end was fixed by a band last iteration.

- **Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001).** *Bad is stronger than
  good* (DOI `10.1037/1089-2680.5.4.323`). Already cited in the crate for `snub`. Its second claim is
  the one that matters here: bad endings are not the only endings. A model in which *every* friendship
  ends has the crate honouring the first claim and violating the second.

## What the paper does not supply

- **No number** for how long a friendship should last, how many endings should be event-driven, or how
  wide a band between "becoming" and "ceasing" should be. All three are **design defaults**, and the
  crate's rule is that such a default says so and is chosen by measurement.
- No mechanism for forgetting a *label* as opposed to forgetting a *value*. Social penetration theory
  describes staged, maintained relationships; it does not prescribe a hysteresis band on a scalar.

## Why the previous iteration's answer is not enough

Last iteration added a band to the `stranger_familiarity` bar and found the friend and rival bars were
event-driven, so a band there "held a friendship after the event that ended it" and was rejected — rivals
moved 42 → 48. **That conclusion was drawn from the label counts alone.** This iteration's attribution
shows the split precisely: of 504 endings, 326 follow an exchange and 178 do not. A band that holds *only
when no event happened* is therefore both possible and correct, and the previous measurement could not
distinguish it from a band that holds unconditionally.

## What believable output looks like

- Friendships last **game-days, not game-hours**: the median held duration rises by an order of
  magnitude.
- **Event-driven endings survive**: after a witnessed insult, the friendship still ends, promptly.
- **Drift-driven endings disappear**: the ~11% of endings with no interaction behind them stop being
  announced.
- The friend count does not explode: the band must be narrower than the sliver it guards, or it becomes
  the clique problem the `affinity_ceiling` was introduced to solve.
- Determinism holds, and no draw or iteration order is added.
