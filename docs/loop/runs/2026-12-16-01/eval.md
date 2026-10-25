# Eval

## Change

**None shipped.** Both documented options were measured and are dead; a third works by weakening a cited model.

## What the measurement establishes

| claim | measured |
|---|---|
| a longer `ticks:` cannot fix it | 18 000 and 24 000 both give **0.4036** — a fixed point |
| `influence_mu` cannot compensate | doubling it made spread **worse** (0.4337) |
| the mechanism is `social ÷ anchor` | anchor pull is the only term not affinity-weighted |
| lowering `anchor_pull` does work | 0.04–0.045 pass, at 2.5× below shipped |
| the clamp improves the village | **16 / 10 / -0.052** against **22 / 33 / -0.054** |
| a fourth model reads `affinity` | `trust_rises` accept rate 0.538 against 0.542 |

## The finding

**`affinity` has four readers**: tie labels, Deffuant influence, conformity, and acceptance. A clamp on it is a
clamp on all four. Three of them are invisible from the scenarios the clamp was aimed at.

The previous four iterations changed *rates*; the last changed the *range*; this one found that the range is read
in **four places with three different formulas** (`max(0, affinity)`, `affinity + 1`, and a bare comparison).

## Options left for the host decision

1. **Cut `anchor_pull`** — works; tunes a cited model to a test.
2. **Re-state the opinion observable** to match a clamped, anchor-held village (~0.40) — honest.
3. **Separate the influence weight from affinity** — the schema change; one number, one meaning.

Recommendation recorded as 2 or 3, never 1.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **53** binaries |
| working tree | only `needs-decision/` modified |

## Dead-code inventory

Zero.
