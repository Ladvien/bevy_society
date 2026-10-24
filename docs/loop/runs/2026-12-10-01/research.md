# Research: the recommended fix does not work

## Why this

Last iteration's decision file recommended **option 1** — a saturating gain on interaction affinity,
`affinity ← affinity + gain · (1 − |affinity|)`, the form the crate already uses for exposure. Before shipping a
model change that alters how every tie forms, I looked for a citation for the saturating shape.

**There isn't one in the corpus.** The closest hit, Reiter et al. 2023 (DOI `10.31234/osf.io/34vnx`) on
"attenuated gain of trust during adolescence", is about developmental trajectories and an irritability
parameter — not diminishing returns per exchange. So the shape is a design default, and the honest thing was to
test whether it works.

## Measured

`village-evening`, seed 7, 24 people, 17 game-days:

| configuration | friends | rivals |
|---|---|---|
| shipped (no saturation, no contact talk) | **22** | **33** |
| saturation alone | **22** | 38 |
| shipped + contact talk at 3 h | **267** | **0** |
| **saturation + contact talk at 3 h** | **260** | **0** |

Saturation alone is nearly neutral — 22 → 22, rivals 33 → 38 — which is why it looked right. It does not
survive contact: **260 against 267** is the same clique.

## Why it fails, in one line

Both curves pass `0.4`, and both pass it well within 45 exchanges. A saturating gain changes **when** the
threshold is crossed, not **whether**. To stop a clique the model needs something that keeps affinity *below*
the threshold for most pairs, and a saturating gain does not.

## What would

| shape | would it bound friendship? |
|---|---|
| saturating gain, asymptote 1 | **no** — measured, 260 friends |
| lower asymptote `K < 0.4` | untested; would need a second input to ever reach the threshold |
| higher threshold, gated on **trust** | untested; trust moves far more slowly than affinity, so it would bound friendship without touching the affinity curve |
| cap the friend count | `trim` exists at 150; a village of 24 needs far less |

## Sources

No paper supplies the saturating shape; the one candidate read (`10.31234/osf.io/34vnx`) does not. The finding
here is arithmetic, and it is worth recording precisely because the recommendation was mine and it was wrong.
