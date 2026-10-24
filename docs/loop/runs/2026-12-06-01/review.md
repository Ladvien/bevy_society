# Review

## Pass 1 — Behavior: is the restated observable true?

**GREAT.** It is true by measurement at two seeds, and it checks something worth checking.

| need | lowest any person reached | cycles, seed 7 | cycles, seed 42 |
|---|---|---|---|
| `social` | 0.45 | 0 of 8 | 2 of 8 |
| `rest` | **0.51** | **0 of 8** | **0 of 8** |
| `food` | 0.00 | **8 of 8** | **8 of 8** |
| `fun` | 0.00 | **8 of 8** | 6 of 8 |

And the assertion that carries the restated claim — every need reaching above 0.7 for every person — passes,
which is what proves no need is being drained with nothing to satisfy it.

## Pass 2 — Zoom: the arithmetic, not the plot

**GREAT**, and here the "zoom" is a division. `rest` cannot cross 0.35 because `sleep` restores `1.20` against
a `0.70/day` decay: **+0.50 net per sleep**. It ratchets up until sleeping stops being chosen. There is no
plot that would show a need running low, because it never does, and the number says why.

`food` is exactly break-even — `1.2 × 0.75 h = 0.90` against `0.9/day` — and that is why it oscillates across
the decision threshold every day while the others do not. The observable's authors wrote about a pack whose
rates were different.

## Pass 3 — Fresh eyes: what a reader would have concluded

**GREAT**, and this is why the iteration was worth doing. A reader of `scenarios.yaml` would have taken
*"`rest` and `food` cross below 0.35"* as a description of the crate, and a reader of the test name
`only_food_cycles_for_every_person_and_the_scenario_says_so` would have taken "only food" as the finding —
while the test's own output printed **`fun cycles for 8 of 8`** underneath. Two documents, three claims, and
none of them agreeing.

The narrative to a host:

> `need-cycle` now checks that every need is met and re-met, rather than that a particular two needs starve.
> The old wording asked `rest` to fall below 0.35, which it cannot: one night's sleep supplies 1.20 against a
> 0.70 decay, so it climbs until sleeping stops paying and never runs low. `food` oscillates daily because
> eating supplies exactly its daily decay, and `fun` usually does too.

## Verdict

Three GREAT. Shipped. No model change: the claim was stale, not the crate.
