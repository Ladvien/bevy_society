# Review

## Pass 1 — Behavior: does each lever do what it claims?

**GREAT.** Every option in the decision file was run rather than assumed.

| lever | claimed | measured |
|---|---|---|
| longer `ticks:` | would restore convergence | **fixed point at 0.4036** (18k ≡ 24k) |
| compensating `influence_mu` | would restore convergence | **0.3813 → 0.4337**, worse |
| lower `anchor_pull` | not previously identified | **works at 0.04–0.045** |

The `influence_mu` result is the one that teaches most: it fails *because* convergence has two affinity-scaled
paths (`max(0, affinity)` for Deffuant, `affinity + 1` for conformity), and `mu` scales only one. Raising it skews
the ratio instead of restoring the total.

## Pass 2 — Zoom: is the mechanism the one that was written down?

**GREAT**, after one correction. The first story — "clamping affinity halves influence" — is true but incomplete.
Reading `anchor_step` shows why the *equilibrium* moves:

```
anchor_step = rate · stubbornness · (anchor − x)
```

No affinity term. The equilibrium is `social pull ÷ anchor pull`; the clamp halves the numerator and leaves the
denominator untouched. That predicts a stable non-zero asymptote, which is exactly what 18 000 = 24 000 shows. The
model explains the data rather than the reverse.

The `anchor_pull` sweep is **non-monotonic** — 0.03 fails (0.3720) while 0.04 and 0.045 pass — so the working band
is interior, not "lower is better". A naive sweep to zero would have overshot.

## Pass 3 — Fresh eyes: what does a host developer now know?

**NOT YET for shipping, GREAT for the decision.** The clamp is unambiguously better *as a society*:

| | friends | rivals | mood |
|---|---|---|---|
| shipped | 22 | 33 | -0.054 |
| full config | **16** | **10** | **-0.052** |

Rivals down **70%**, mood unchanged. But making it pass costs four coupled changes, one of which cuts the
Friedkin–Johnsen anchor 2.5× — and that anchor is what makes an opinion *persist*. Cutting a cited term to clear a
test threshold is the precise failure this crate's honesty rule exists to prevent, so it cannot be done silently.

The `trust_rises` failure is the best evidence for the real conclusion: **`affinity` has four readers** (labels,
Deffuant, conformity, acceptance), in three different formulas. Three are invisible from the scenarios the clamp
targets. That is why nine attempts failed — the variable is shared, and only separating it fixes it once.

## Verdict

Pass 1 **GREAT**, Pass 2 **GREAT**, Pass 3 **NOT YET** (for shipping; GREAT for the decision).

**Reverted.** 53 binaries green; only `needs-decision/` modified.

The iteration's value: the decision is now closed by measurement instead of argument, both convenient options are
proven dead, the mechanism is identified and predicts the asymptote, and the host gets three options with numbers
and a recommendation that never tunes a cited model to a test.
