# Plan: close the clamp's decision with measurements

## Why this over the runner-up

Item 1 was the clamp decision. The previous iteration recorded two options as if they were plausible. **They had
never been run.** Before a structural decision goes to the host, its options should be measured — otherwise the
decision is a guess wearing a table.

## What was done

1. Measured **option 1** (longer `ticks:`) across 12 000 / 18 000 / 24 000 / 36 000.
2. Measured **option 2** (compensating `influence_mu`).
3. Found the mechanism both options missed: the equilibrium is `social pull ÷ anchor pull`, and only social pull is
   affinity-weighted.
4. Measured **option 3** (lower `anchor_pull`) across 0.03–0.07.
5. Ran the full suite with the complete configuration, and found a fourth reader of `affinity`.

## Measured

| lever | result |
|---|---|
| `ticks:` 18 000 / 24 000 | **0.4036 / 0.4036** — a fixed point, not slow convergence |
| `influence_mu` 0.3 → 0.6 | 0.3813 → **0.4337** — worse |
| `anchor_pull` 0.045 / 0.04 | **passes**, at 2.5× below the shipped value |
| full config, village | **16 friends / 10 rivals / -0.052** vs **22 / 33 / -0.054** |

## Why nothing shipped

The clamp is **good for the village** — rivals down 70%, mood unchanged — and it needs **four** coupled changes,
one of which cuts a cited model 2.5× purely to clear a threshold. That is exactly the choice the loop's honesty
rule reserves for `needs-decision/`.

## Touched

- `docs/loop/needs-decision/friendship-has-no-diminishing-return.md` — the three-lever map, the mechanism, the
  fourth reader, the three remaining options.
- `CHANGELOG.md`, `docs/loop/log.md`.
- **Reverted**: all source, assets, and test changes.

## New `Tuning` fields

None shipped. Option 3 would change the meaning of `anchor_pull`, which is why it is a decision.
