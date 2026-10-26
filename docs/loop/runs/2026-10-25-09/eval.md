# Evaluation: what makes a village hostile when it grows

**No model change shipped this iteration.** The change I designed was measured, worked mechanically, and was
rejected because the diagnosis underneath it was wrong. This file records the evidence, because the diagnosis is
the deliverable.

## The item this iteration set out to fix

Last iteration's top remaining item was the price model. The evidence sent the iteration here instead: the same
nine-seed run found **rivalries exploding with population** — 18 friend pairs / 18 rival pairs at 24 people, but
183 rival pairs at 36 (seed 99) — which outranks a price that rests at its floor, because it is behaviour a host
cannot ship.

## Measurement 1: it is real and reproducible

`village-evening`, five seeds, three populations, the shipped binary. `evidence/before/`.

| people | friend pairs | rival pairs | **% of directed pairs hostile** | interrupted endings |
|---|---|---|---|---|
| 24 | 18.4 | 21.6 | **7.83%** | 318 |
| 36 | 20.2 | 145.4 | **23.08%** | 711 |
| 48 | 32.0 | 226.8 | **20.11%** | 1 325 |

Rival density triples between 24 and 36 people, and the interruption count doubles and then doubles again. Both
are reproducible across five seeds (`evidence/before/*/summary.json`).

## Measurement 2: the hypothesis I built on, and why it was wrong

`src/reputation.rs::witness_reaction` divides **trust** by `witnesses ^ Tuning::witness_dilution` (0.5) and
applies **affinity**, **respect** and the memory's **importance** at full magnitude to every witness. Trust was
diluted an iteration ago and the other three were left. Since affinity is what `TieLabel` calls a rivalry, a deed's
total hostility grows linearly with the crowd while its total trust cost grows as a square root — an inconsistency
in one function.

The numbers looked like a textbook confirmation:

| people | mean witnesses per deed |
|---|---|
| 24 | 12.6 |
| 36 | 21.5 |
| 48 | 30.0 |

**So I diluted all four effects.** The mechanism test passed (`tests/witness_dilution.rs`, three tests: every
effect falls per witness; the total still rises; the total is sublinear). And it "worked":

| people | rival % before | rival % at exponent 0.25 | 0.30 | 0.35 | 0.50 |
|---|---|---|---|---|---|
| 24 | 7.83% | 0.87% | 0.29% | 0.22% | 0.22% |
| 36 | 23.08% | 1.75% | 0.98% | 0.51% | 0.22% |
| 48 | 20.11% | 1.42% | 0.82% | 0.43% | 0.28% |

**It flattened the density — and emptied the graph.** At 24 people, rival pairs went 21.6 → 1.0 and `became_rival`
fell from 218 to 11. The plan had named that outcome a regression in advance ("a fix that empties the hostile half
of the graph is a regression, not a fix"), and it is: a village where almost nothing anyone does is resented is not
more believable than a hostile one. The exponent that leaves the 24-person village intact is 0 — i.e. no change.

## Measurement 3: the control that found the real cause

Before tuning the exponent further I asked whether the 36- and 48-person runs were a fair test of a bigger village
at all. `examples/evidence.rs` builds the **same** village for every `--people`, and its four places declare
capacity 6 + 10 + 8 + 6 = **30 seats**. A 36- or 48-person run therefore puts more people in the village than it
has room for. The crate itself says so at startup:

> `'well' is home to 16 people, but an arrival disc of radius 1.5 seats about 4. ... their sleeps are interrupted.`

Scaling the places' capacity with the population and changing nothing else (`evidence/seated/`):

| people | rival % when over capacity | rival % with seats for everyone | friend pairs |
|---|---|---|---|
| 36 | 23.08% | **0.00%** | 67.2 |
| 48 | 20.11% | **0.02%** | 81.6 |

**Rivalry density goes to zero.** The explosion was never about the crowd multiplier: it was a village that could
not seat its people. And the mechanism is visible in where a life goes — an oversubscribed village loses `work`:

| village | work | compliment | snub | gossip |
|---|---|---|---|---|
| 24, as authored | 10.3% | 4.6% | 1.2% | 4.2% |
| 36, over capacity | 5.5% | 5.4% | 1.3% | 4.9% |
| 48, over capacity | 3.6% | 7.4% | 1.3% | 7.2% |
| 36, seated | 10.3% | 3.2% | 0.8% | 2.8% |
| 48, seated | 7.9% | 3.3% | 0.9% | 3.2% |

`work` collapses and social contact roughly replaces it, so the village spends its day in company it did not choose
— and `snub` barely moves (1.2% → 1.3%), while `compliment` and `gossip` rise. The rivalries come from the same
number of hostile events being applied to a much denser contact graph, and the contact graph is dense because
nobody can get to work.

`evidence/plots/capacity.png` — left, the density curve that looked like a scaling law and is a capacity effect;
middle, the activity mix; right, interruptions, which are **not** what changes (see below).

## What this means, and what it does not

- **The dilution change is reverted.** Nothing of it remains: `src/reputation.rs`, `src/core.rs` and
  `tests/witness_dilution.rs` are byte-identical to the commit this iteration started from.
- **The 36- and 48-person numbers in last iteration's log are not a bigger village**; they are an overfilled one.
  The `--people` flag scales the people and not the village, which is a harness trap that produced a false
  "scaling law".
- **It is not a crate bug that a village without capacity warns and then behaves badly.** The crate warns at
  startup, in the words quoted above, and the guide says `capacity` limits how many people may use a place at
  once. The open question is whether a *host* should have to think about it at all, and that is now
  `docs/loop/needs-decision/a-village-with-no-room.md`.
- **Interruptions do not improve when seated** (right panel: 711 → 815 at 36, 1 325 → 1 313 at 48). Capacity buys
  a work life and a friendlier graph, not fewer timed-out walks. Those are a travel-time constant
  (`travel_timeout_hours` vs `seconds_per_day`) and have their own note.

## What shipped

Two things, neither a model change:

1. `examples/evidence.rs` gained `--witness-dilution`, the flag the exponent sweep needed. It is a measurement tool,
   and it is how the next attempt at this question will be measured.
2. `tools/big-cargo.sh`, so compilation and the test suite run on `big` (24 cores) instead of the laptop. Verified
   there: a first `--all-features` release build in **2m19s** and **76 of 76** test binaries green, the same count
   as the laptop. Capture, `gif.py` and the evidence plots stay on the laptop — a PNG rendered on a different GPU is
   not comparable with one rendered here.

## What is not good enough

1. **A `--people` sweep is not a population sweep.** The harness holds the layout fixed, so any future scaling
   claim has to scale the places too. The flag should probably do it, or refuse; that is the decision above.
2. **The dilution inconsistency is real and unfixed.** Three of a deed's four consequences still scale with the
   crowd. It is a defect, it is now measured, and it cannot be fixed alone: at 24 people the crowd is already 12.6
   witnesses, so any positive exponent removes the village's rivalries.
3. **Rivalries are load-bearing.** Whatever replaces this must keep ~20 rival pairs at 24 people.
