# Research: a term nothing in the pack could raise

## The observation

`AcceptDef` scores every interaction as

```text
base + affinity·a + trust·(t − 0.5) + agreeableness·(g − 0.5) + pleasure·p + ingroup
```

Trust is **centred on 0.5** — `Tie::new` starts every stranger there, and the term is zero at that point.
Measured with 24 people over 20 000 ticks, the trust a target held toward the person approaching it:

| interaction | trust |
|---|---|
| `chat` | 0.27 |
| `compliment` | 0.20 |
| `gossip` | 0.24 |
| `insult` | 0.13 |

Every one below neutral, so the trust term **subtracts on every acceptance draw**, and roughly 47% of
interactions were refused.

The cause is not a rate. `TieDelta` carries a `trust` field and every interaction's `on_accept` and
`on_reject` can set one — and the word `trust` **does not occur in `assets/starter.society.ron` at all**. The
only thing that moved trust was `witness_reaction`:

```rust
tie.trust = (tie.trust + gain * harm).clamp(0.0, 1.0);
```

with `gain = witness_gain · (0.5 + conscientiousness)` ≈ 0.2. So trust could only ever fall, and it fell on
every deed the pack actually uses.

## The literature

**Nowak & Sigmund 1998**, DOI `10.1038/31225` — image scoring, already cited at `Tuning::witness_gain`. The
mechanism is that observers update an image of an actor from what they see it do, in both directions. The crate
implements the update; the pack supplied only acts worth scoring negatively.

**Cacioppo, Cacioppo & Gollan 2013**, DOI `10.1017/S0140525X13002537` — the negativity bias, already the crate's
citation for the positivity offset. A negative deed costing more than a positive one is *correct*. What the
paper does not license is a channel that only ever runs one way.

## What believable output looks like

Two people who interact successfully come to trust each other a little more, so that a village's trust can
recover from its disagreements. Measured as a controlled comparison — the same pack with the deltas stripped —
mean trust **0.261 → 0.345** and accept rate **0.486 → 0.533**.

## What this cannot fix, and why

A trust *gain* reaches the two parties. A trust *loss* reaches **every witness**, at full magnitude. One chat
is `+0.10` in total; one insult seen by three people is `-0.42`. **The loss is multiplied by the crowd and the
gain never is.** That is a design question, recorded in
`needs-decision/a-deed-costs-what-the-crowd-saw.md`, and it is why trust in `village-evening` still declines
over a run despite the deltas.
