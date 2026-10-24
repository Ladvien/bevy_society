# Research: the bar is not the lever; the asymptote is

## Why this

Item 1 was "a bound below the friendship threshold". Last iteration put the five label thresholds on `Tuning`,
which made the question measurable **without a code change** — so this iteration could sweep the bar and then
test the shape, rather than guess at either.

## The bar is not the lever

`village-evening`, seed 7, 24 people, 17 game-days:

| `friend_affinity` | friends | rivals | mood |
|---|---|---|---|
| 0.4 (shipped) | **22** | 33 | -0.054 |
| 0.55 | 18 | 33 | -0.054 |
| 0.70 | 18 | 33 | -0.054 |
| 0.80 | 16 | 33 | -0.054 |

Doubling the bar removes six pairs. Because affinity reaches **1.0** for anyone in contact, every bar below 1.0
is eventually passed.

## It is not a runaway

A four-times-longer run, same seed:

| game-day | friend-ties |
|---|---|
| 8.2 | 77 |
| 16.4 | 115 |
| 24.6 | **122** peak |
| 32.8 | 121 |
| 41.0 | 120 |
| 57.4 | 109 |

It **saturates and then decays**. So the problem is a saturating system with a **high asymptote** — 122 of 276
pairs, 44% of the village — not an unbounded one. The asymptote is wherever affinity settles, and affinity
settles at 1.0. My prediction that a longer run would give a bigger clique was **wrong**, and the measurement
corrected it.

## The shape that works

Clamp affinity to `[-K, K]` with `K = 0.35` — **below** the bar — so contact alone cannot make a friend, and a
second input is required.

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | **22** | **33** | -0.054 |
| clamp only (bar unchanged at 0.4) | **0** | 18 | -0.078 |
| clamp + trust ≥ 0.55 + affinity bar 0.25 | **15** | **18** | -0.078 |
| clamp + trust ≥ 0.55 + affinity bar 0.30 | **12** | **18** | -0.078 |

**Both tie counts fall** — the first shape in six attempts to do that without inverting the village. Holds at
three seeds (4–6 friends at the strictest setting).

## The cost, and where it comes from

Mood **-0.054 → -0.078**. The two bar settings give *identical* mood, so the cost is entirely the **clamp**.
Affinity-gated activities (`chat` at `Logistic(8.0, 0.4)`, `compliment` at `Logistic(8.0, 0.7)`) see lower
affinity, so fewer contacts, so fewer `Liking` exchanges, so lower mood. The chain is visible in the numbers,
not assumed.

## The bug worth not repeating

Scaling by the asymptote, `delta · (1 − |a|/K)`, is **asymmetric**: a negative delta steps past `-K` to the
outer `-1` clamp while a positive one decays smoothly. Measured, rivals rose **33 → 48** instead of falling. A
plain **clamp** to `[-K, K]` bounds both directions, and is what the table above uses.

## Sources

No paper supplies the clamp; it is a design default. The two mechanisms `classify` cites — mere exposure
(Zajonc 1968, DOI `10.1037/h0025848`) and similarity (Byrne 1961, DOI `10.1037/h0044721`) — are unchanged.
