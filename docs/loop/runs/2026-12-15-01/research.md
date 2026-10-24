# Research: the clamp's cost is the opinion model, not mood

## Why this

Item 2 was "recover the clamp's mood cost". Last iteration attributed the cost to fewer affinity-gated contacts
and suggested raising `mere_exposure_per_day`. Testing it found the attribution was **wrong in kind**.

## The mechanism

`compliment` gates on `Affinity` at `Logistic(8.0, 0.7)`. Measured:

| affinity | gate score |
|---|---|
| 0.35 (clamp ceiling) | **0.057** |
| 0.79 (unclamped mean) | **0.673** |

Compliments become **12× less likely** under the clamp. That is a curve/gate mismatch, and it explains the mood
loss exactly — no rate was missing.

## The repair, and it works

Move the gate down to match the ceiling: `Logistic(16.0, 0.25)`.

| configuration | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | **-0.054** |
| clamp 0.35, gate unchanged | **0** | 10 | -0.078 |
| clamp + gate at 0.25 | **0** | 10 | **-0.045** |
| clamp + gate 0.25 + `friend_affinity` 0.30 + trust ≥ 0.5 | **16** | **10** | **-0.045** |
| …with `friend_affinity` 0.25 | 24 | 10 | -0.045 |

A bar at 0.4 is unreachable under a 0.35 ceiling, so `friend_affinity` must move under it — which is why the
first two rows have **0 friends** despite the second input existing. At 0.30 the village has 16 friends and 10
rivals: a third of the shipped rivals, and mood better than baseline.

## And it breaks a different scenario

`tests/opinion_convergence.rs`, which asserts opinion spread falls by at least a third in 12 000 ticks:

| | before | with the clamp |
|---|---|---|
| spread | 0.5106 → **0.24** (53% fall) | 0.5106 → **0.3813** (25% fall) |

Deffuant influence is **weighted by tie strength**, so halving the affinity ceiling roughly halves the
convergence rate. The test's own comment records that it already needed 12 000 ticks rather than the scenario's
6 000 to cross a third; with the clamp it does not cross at all in that time.

That is a coupling across **three models** — ties → opinions — and none of the six earlier attempts found it,
because none of them changed the affinity *scale*. They changed rates, gates, or the accumulation shape.

## What it means

**The affinity scale is shared.** It sets how likeable people are, how quickly they become friends, *and* how
fast opinions spread. Bounding the tie network therefore re-times the opinion model, and a scenario's tick count
is silently a statement about that rate.

Any clamp needs one of:

1. a longer `ticks:` in `opinion-converges-but-not-entirely` — honest but hides the change;
2. a compensating `influence_mu` — keeps the scenario and couples two rates instead;
3. a separate variable for influence weight, so liking and influence stop sharing `affinity` — the clean fix, and
   a schema change.

## Sources

No paper is involved. The coupling is a fact about this crate's data model: `Opinion` influence reads `Ties`,
and `Ties::affinity` is the quantity the clamp bounds.
