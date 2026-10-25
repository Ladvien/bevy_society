# Plan: give the asymmetry a cited default

## Why this over the runner-up

Last iteration's top item: separate forgetting rates, "needs a cited default". Going to the corpus for that default
found something better — the paper says the aggregate outcome is decided by **count of events**, not by decay, and
the starter pack does not satisfy its condition.

## What was tried

1. **Asymmetric decay** — negative affinity fades half as fast (Baumeister 2001, "harder to shed"). Measured:
   rivals **42 -> 82**, friends 18 -> 11. **Rejected**, and backwards from my prediction.
2. **Isolating the mechanism** — with `balance_rate_per_day = 0` the result is still 64 rivals, so structural
   balance explains 18 of the 40. The rest is direct: persisting grudges persist while friendship keeps fading.
3. **Stronger `compliment`** (+0.10 -> +0.20). Seed 7 gives 31 rivals / 14 friends, seed 42 gives 44 / 7.
   **Inconsistent**, so not a trend.

## The diagnosis that survived

The asymmetry is in the **event weights and counts**, and it is measurable:

| event | affinity effect | reach |
|---|---|---|
| `compliment` | +0.10 | one person |
| `insult` (harm `-0.4`) | -0.08 per witness | every witness |

~6 witnesses means one insult is **-0.48 of total affinity** against **+0.10** for a compliment — bad outweighs
good **5:1 per event**. And `compliment` holds **11.3%** of person-ticks against `snub`'s **3.8%**, only **3:1**.
Good is outnumbered 3:1 and outweighed 5:1, so friendship cannot accumulate.

## Touched

- `docs/loop/needs-decision/good-is-outweighed-by-bad.md` — new, with three options.
- `docs/research/relationships.md` — the citation, both of its claims, and the measurement.
- `CHANGELOG.md`, `docs/loop/log.md`, this run record.
- **No crate changes**: `git diff` on `src/` and `assets/` is empty.

## Why nothing shipped

Three levers, three rejections. The one that follows the paper — raise the *rate* of friendly interaction rather
than its weight — is authored content and needs a three-axis measurement (ties, needs, interruptions) before it
can be chosen. The measurement is this iteration's deliverable.
