# Research: a rivalry is released at 0.0001, a friendship is held through 0.05

Iteration `2026-10-27-02`. The pick is the asymmetry between how the tie label **holds** a friendship and how it
**releases** a rivalry.

## The measurement that chose it

The loop log's #1 was "raise the rate of friendly interaction" (run `2026-12-28-01`). Measuring the current tree
first, on `village-evening` at 17.62 game-days, three seeds, showed that premise is stale again — the village is
already friend-dominated:

| seed | friends | rivals | became rival | left rival | refused / accepted approaches |
|---|---|---|---|---|---|
| 7 | 25 | 2 | 47 | 42 | 1 533 / 2 659 |
| 42 | 19 | 2 | 43 | 38 | 1 633 / 2 748 |
| 11 | 21 | **0** | 19 | 19 | 1 416 / 2 707 |

`became_rival` 47 against `left_rival` 42 means rivalry is **formed and dissolved at nearly the same rate**. Rival
pairs exist but never persist; at every seed the count is 0–4 across the whole village. So the question is not why
the village has too few rivals, it is why the rivalries it makes evaporate.

Pair-level measurement (`tests/zz_scratch_rival_spells.rs`, deleted after this run; 12 000 ticks, 2.93 game-days,
three seeds):

| seed | rival spells | median spell | **margin above the −0.30 bar at release** |
|---|---|---|---|
| 7 | 41 | 0.289 game-days | median **0.0002** |
| 42 | 92 | 0.375 game-days | median **0.0001** |
| 11 | 16 | 0.141 game-days | median **0.0001** |

Released with `familiarity_peak < 0.1`: **0 of 149**. So a rivalry does not end because the parties forget each
other. It ends because affinity rises by **one tick's worth** past the bar.

## What the code does

`TieLabel::classify_from` (`src/relationships.rs`) holds a **friend** through a band:

```rust
if promoted != Friend && current == Friend
    && affinity >= tuning.friend_affinity - tuning.friend_band  // 0.30 - 0.05
    && trust >= tuning.friend_trust
    && familiarity >= tuning.friend_familiarity
{ return Friend; }
```

There is no matching clause for **Rival**. A rival is released the instant `affinity > -rival_affinity`, which
`Tuning::label_hysteresis` (0.02) only delays through the `stranger_familiarity` clause — and that clause is about
falling to `Stranger`, not about leaving `Rival`.

So the crate has a **0.05 hold band on the positive side and a 0.02 one on the negative side that does not apply
to the negative side's own category**, and affinity relaxation (`affinity_relax_per_day` 0.01) plus one accepted
compliment (+0.10) is worth far more than either.

### The asymmetry is the opposite of the one the crate cites

`assets/starter.society.ron`, `snub`'s consideration, already cites **Baumeister, Bratslavsky, Finkenauer & Vohs
2001**, *Bad Is Stronger Than Good*, DOI `10.1037/1089-2680.5.4.323`. Its first finding:

> "bad impressions and bad stereotypes are quicker to form and **more resistant to disconfirmation** than good
> ones"

"Quicker to form, more resistant to disconfirmation" is precisely a **hold band on the bad side wider than the
one on the good side**. The crate implements neither half of it: a rivalry is quicker to form (it needs one
insult, at −0.08 per witness across a crowd) and *less* resistant to disconfirmation (it dies on the next tick's
relaxation step).

The second finding — "good may prevail over bad by **superior force of numbers**" — is why the rival band must be
a *band* and not a floor: good still wins by count, which the village already demonstrates (19–25 friends against
0–2 rivals at every seed). Holding a rival for longer does not make the village hostile; it makes the hostility it
does express persist long enough for a host to see it.

## The formula

One clause in `TieLabel::classify_from`, symmetric with the friend clause:

```rust
if promoted != Rival && current == Rival
    && affinity <= -(tuning.rival_affinity - tuning.rival_band)
    && familiarity >= tuning.rival_familiarity
{ return Rival; }
```

`rival_band` is a new `Tuning` field. It is a **design default** — Baumeister supplies the direction ("more
resistant to disconfirmation") and no number, exactly as it supplies the direction for `snub`'s curve and not its
slope. The number is chosen by measurement below.

Note what the clause is *not*: it is not unconditional. A rivalry that climbs all the way past
`-(rival_affinity - rival_band)` is released, so a reconciliation still happens; it just needs a real change
rather than a rounding. And it does not apply to leaving a friendship, which keeps its own `friend_band`
clause — the two bands guard two categories and a tie can only be in one of them.

## What believable output looks like

From the paper: bad is quicker to form and slower to release, good prevails by count.

- **Rival spells lengthen** from a median of 0.14–0.38 game-days to something a host can narrate — and the
  margin at release moves off 0.0001.
- **Rival pairs at the end rise** or hold, and `became_rival` falls (fewer re-formations, because the ones that
  form are still rivals).
- **Friends must not collapse.** The village is friend-dominated; if friends fall by anything like the rivals
  gain, the change has moved the whole network instead of the asymmetry, and a narrower band is the answer.
- **The reconciliation rate must stay above zero.** `left_rival` > 0 at every seed, or the change has made
  grudges permanent rather than resistant.

## Sources

- Baumeister, R. F., Bratslavsky, E., Finkenauer, C. & Vohs, K. D. (2001). Bad is stronger than good. *Review of
  General Psychology* 5(4), 323–370. DOI `10.1037/1089-2680.5.4.323`. Already cited in `assets/starter.society.ron`
  for `snub`; supplies the direction of the asymmetry and not its size.
- Sherif, M. & Hovland, C. I. (1961), as reproduced in Fagan & Browne (1994: 219–220), corpus doc
  `10.1007_0-306-47586-3_8` — the **latitude of acceptance**: a judgement about a person is a band around an
  anchor, and a stimulus just past the anchor's end shifts the band rather than reversing the judgement. This is
  the shape `label_hysteresis` and `friend_band` already use, cited at those fields; the rival band is the same
  shape applied to the category that had none.
- Bondü, R. & Richter, P. (2016). DOI `10.3389/fpsyg.2016.00795` — rejection and provocation sensitivity predict
  reactive aggression; the mechanism behind `snub` and therefore behind which pairs become rivals at all. Cited
  for mechanism only.
