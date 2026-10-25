# Research: every lever on the clamp's opinion coupling, measured

## Why this

The clamp decision had two documented options (longer `ticks:`, compensating `influence_mu`). Both were untested
claims. This iteration measured all of them, and found the actual mechanism.

## The equilibrium is a ratio, and only one side is clamped

Every person's opinion moves under three terms:

| term | formula | affinity-weighted? |
|---|---|---|
| Deffuant | `mu · weight · d`, `weight = 0.5·trust + 0.5·max(0, affinity)` | **yes** |
| Conformity | mean over contacts weighted `max(0.1, affinity + 1)` | **yes** |
| Anchor | `anchor_pull · stubbornness · (anchor − x)` | **no** |

Deffuant et al. 2000 (DOI 10.1142/S0219525900000078); Friedkin & Johnsen 1990 (DOI 10.1080/0022250X.1990.9990069).

Clamping affinity at 0.35 halves the numerator of `social ÷ anchor` and leaves the denominator alone. So the
equilibrium moves toward each person's **anchor** — the village becomes individually stubborn, and spread settles
where the two pulls balance rather than where consensus lies.

Measured weights:

| path | clamped 0.35 | unclamped 0.79 |
|---|---|---|
| Deffuant (trust 0.5) | 0.425 | 0.645 |
| Deffuant (trust 0.9) | 0.625 | 0.845 |
| Conformity | **1.35** | **1.79** |

## Option 1: longer `ticks:` — dead

The settlement is a fixed point, not slow convergence:

| ticks | spread | required |
|---|---|---|
| 12 000 | 0.3813 | ≤ 0.3405 |
| 18 000 | 0.4036 | |
| 24 000 | **0.4036** | |
| 36 000 | 0.4005 | |

18 000 and 24 000 are **identical to four decimals**. No duration can pass the scenario.

## Option 2: compensating `influence_mu` — dead, instructively

`mu` 0.3 → 0.6 made spread **worse** (0.3813 → 0.4337). `mu` scales Deffuant only, so raising it changes the
*ratio* between Deffuant and conformity rather than restoring total social pull. More `mu` means each contact
moves you further toward a *nearer* equilibrium, and the anchor drags the rest back.

## Option 3: lower `anchor_pull` — works, and that is the problem

| `anchor_pull` | spread |
|---|---|
| 0.10 (shipped) | 0.3813 (fail) |
| 0.07 | 0.3611 (fail) |
| 0.06 | 0.3525 (fail) |
| 0.05 | 0.3426 (fail) |
| **0.045** | pass |
| **0.04** | pass |
| 0.03 | 0.3720 (fail) |

Non-monotonic with an interior band. **It passes by cutting a cited model 2.5×.** An anchor pull is what makes an
opinion persist instead of following the last speaker; shrinking it to clear a scenario threshold is tuning the
model to the test, which the crate's honesty rule forbids.

## The village, with everything on

**16 friends / 10 rivals / -0.052**, against shipped **22 / 33 / -0.054**.

Rivals down **70%**, friends down 27%, mood unchanged. As a *society* the clamp is a clear improvement. As a
*scenario* it fails one threshold.

## The cost nobody predicted

`trust_rises` fails: accept rate **0.538** against **0.542**, while mean trust *rises* (0.497 against 0.341). The
trust second input labels more ties as friends, and `AcceptDef` centres on trust 0.5, so a village at 0.497 sits
just under its own midpoint. A fourth model reads the same variable.

## What this establishes

`affinity` is read by **four** models: tie labels, Deffuant influence, conformity, and acceptance. A clamp on it is
a clamp on all four, and three of them are not visible from either scenario the clamp was aimed at.

That is why six rate-level attempts and three lever-level attempts all failed. The variable is shared; only
separating it fixes it once.
