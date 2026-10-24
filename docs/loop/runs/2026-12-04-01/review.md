# Review

## Pass 1 — Behavior: does the change do what the paper says?

**GREAT.** Sommerfeld et al. 2007 measured that gossip transmission *dampens* — "the more extreme a gossip
was, the less impact it had on the reader" — and cited Hess & Hagen on believability rising with the number of
sources. The change models exactly that: the same information, believed, moving each recipient less.

| check | result |
|---|---|
| 1 witness | costs each **0.0800** — unchanged, correctly |
| 5 witnesses | **0.0431**, not 0.0800 — the diluting |
| controlled trust | accepts **0.499 → 0.558**, trust **0.299 → 0.435** |
| written red first | the test failed at **0.0800 vs 0.0800** — identical, the defect in one line |

The dilution touches magnitude only: affinity, respect and the emotion keep their sign and full weight, so a
witness that objects still objects.

## Pass 2 — Zoom: read the plot closely

**GREAT**, and the plot is the argument. `evidence/dilution.png`:

- **Left:** the flat red line is the old behaviour — every crowd size costs the same per head, which is why the
  total grew with the crowd. The green curve is the dilution. Two lines, one defect, one fix.
- **Middle:** three seeds. Two moved a lot (−0.127 → −0.046 and −0.110 → −0.052), one barely (−0.058 → −0.054).
  I plotted all three rather than the mean.
- **Right:** rivals fall where the effect is large (50 → 34 at seed 42) and **rise slightly at seed 99**
  (45 → 47). That bar is the honest one: a village that trusts more also *insults* more often, because the
  activity is not gated on trust.

No clipping at ±1, no oscillation at a tick boundary, no dead agents.

## Pass 3 — Fresh eyes: three seeds, and what I refused to average

**GREAT.** Three seeds because the last iteration's single-seed number could have been luck, and they disagree
enough to be worth showing individually.

The narrative to a host:

> A bad deed now costs each witness less when more people saw it — the same news, believed by everyone and
> weighing less in each head. Before, a deed seen by five took five times the total trust, while a good deed
> could only ever reward two people, so a village's trust could only fall. You can turn the old behaviour back
> on with `witness_dilution: 0.0`.

## Verdict

Three GREAT. Shipped, with the seed-99 rival increase recorded rather than explained away.
