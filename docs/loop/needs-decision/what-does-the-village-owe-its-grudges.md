# What does the village owe its grudges?

> **Measured 2026-12-27. The mechanism works and the rate is a choice. Nothing shipped beyond the metric.**

> **Assumed decision (owner may override), 2026-10-30-25: option 3, in the form the literature supports, shipped as
> the default.** Not a second relaxation rate on negative affinity (measured 2026-12 to *double* rivalries), but the
> grudge that drives `snub` fading at a pace set by the holder's trust in the offender: `Tuning::forgiveness` 12.0,
> `ctx.grudge` reads each memory at `age · (1 + 12 · trust)`. The paper option 3 lacked: McCullough et al. 2010 (DOI
> 10.1037/a0019349, forgiveness tracks relationship value), Burnette et al. 2012 (DOI 10.1177/0146167211424582,
> value and low exploitation risk), Strelan, Karremans & Krieg 2017 (DOI 10.1111/bjso.12173, trust is the mediator).
> Same seeds: village rivals 54/56/59/66 → 28/24/18/36 and friends 46/30/40/35 → 62/65/47/54; town of 60 rivals
> 277/229 → 111/155, friends 49/52 → 101/86 (`runs/2026-10-30-25/eval.md`). Options were (a) this default, (b) ship
> it opt-in at 0.0, (c) 16.0 (warmer, but under the rumor book's pinned 20% tellable line). Picked (a): it moves the
> shipped society, `0.0` restores the old curve bit for bit, and it touches no protected check. Override by setting the
> default back to `0.0` or up to `16.0` with `tests/rumor_book.rs` re-measured.

## The state

On `village-evening` (24 people, 17 game-days) the village settles at **42 rival pairs against 18 friend pairs** —
15% of all 276 pairs are rivals, 6.5% are friends.

## Established

`TieLabelChanged` is now counted both ways in `summary.json`:

| seed | became rival | left rival | ratio | final rivals | friends |
|---|---|---|---|---|---|
| 7 | 272 | 188 | 1.45 | 42 | 18 |
| 42 | 308 | 238 | 1.29 | 35 | 19 |
| 11 | 377 | 297 | 1.27 | 40 | 14 |

So reconciliation **exists** — a quarter of rivalries end — and the village forms them 1.3-1.45x faster than it
dissolves them. The count decelerates (second-half growth 0.68x the first), so it saturates.

## The lever, measured

`Tuning::affinity_relax_per_day` is the reconciliation rate:

| relax | rivals | friends | ratio |
|---|---|---|---|
| 0.01 (shipped) | 42 | 18 | 1.45 |
| 0.015 | 28 | 7 | 1.22 |
| 0.02 | 25 | 7 | 1.20 |
| 0.03 | **10** | 5 | **1.08** |

Monotone on rivalry, and **wrong on friendship**: relaxation pulls all affinity toward zero, so it cannot
distinguish a grudge from a fondness. At 0.03 the village is nearly friendless.

## Options

### 1. Leave it. A village with more rivals than friends is a tense village, not a broken one.

- **For:** the mechanism is right, it saturates, and 15% rival pairs is defensible for a small closed village
  where everyone competes for one well.
- **Against:** 2.3 rivals per friendship is a strong flavour, and a host expecting a warm village will be
  surprised.

### 2. Raise `affinity_relax_per_day` to about 0.02 and accept fewer friendships.

- **For:** one number, rivalries down 40%, ratio near parity.
- **Against:** friends fall 18 -> 7. It trades one imbalance for another, and it is the *same* variable, so a
  host cannot tune one without the other.

### 3. Separate forgetting-for-grudge from forgetting-for-fondness.

- **For:** the only option that can fix the ratio without emptying the village. Negative affinity relaxes on its
  own rate; positive on another.
- **Against:** two `Tuning` fields where there was one, and a claim that the two processes differ which no paper
  in the corpus supports — negative affect does appear to persist longer than positive (negativity bias), but
  that would be a **new** cited default, not a reparameterisation.

## Recommendation

**Option 1 now, option 3 if a host wants a warmer village.** The measurement is the deliverable; the crate's job
is to report 1.27-1.45 and let a designer choose. Option 2 is the trap — it looks like a fix and is a trade.
