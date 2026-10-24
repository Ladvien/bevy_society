# Research: the number that steered eight iterations and was never measured

## What the log said

Since iteration 2026-11-06-01, the top-5 list has carried the same line:

> **Retire untellable rumors** — 93% of the book cannot spread; a 4× cost growth over 3 000 ticks.

Eight iterations. It was the #1 item twice. The decision file
`needs-decision/retiring-untellable-rumors.md` rests on it: the reason to change `alive`'s meaning was
that the per-tick walk over a growing book was **four times** more expensive after 3 000 ticks.

That number was never measured. It came from a run that compared a short benchmark against a long one
and attributed the whole difference to the book.

## Measured

200 people, four places, a four-second game-day. The per-tick cost is sampled over 100 ticks at five
points of a 6 000-tick run, once with no `RumorReach` trigger and once with one:

| tick | rumors in book | ms/tick, no trigger | ms/tick, with a trigger |
|---|---|---|---|
| 1 000 | 765 | 5.76 | 5.95 |
| 2 000 | 1 454 | 5.64 | 4.99 |
| 3 000 | 2 412 | 3.51 | 3.07 |
| 4 000 | 3 374 | 2.97 | 2.79 |
| 5 000 | 4 299 | 4.80 | 3.93 |

**The per-tick cost does not grow.** It drifts *down*, because the village settles: fewer people
re-decide as activities lengthen. A `RumorReach` trigger does not change that, so the walk over 4 299
rumors is not what a tick spends its time on — 4 299 pointer-chases beside 200 people scored against
every neighbour within 8 m is nothing.

The second half of the claim holds: on a 3 000-tick run the book holds **2 250 rumors, all `alive`,
with 301 still tellable** — **87% untellable**. That is real, and it is what a retirement rule would fix.

## So what does the book cost?

Memory, and only memory. A `Rumor` is about 48 bytes, so:

| run | rumors | book |
|---|---|---|
| 200 people, 3 000 ticks (12 game-days) | 2 250 | ~105 KiB |
| 200 people, 6 000 ticks (22 game-days) | 4 865 | ~228 KiB |
| 5 000 people, extrapolated | ~25× | ~5 MiB |

Real, linear, predictable, and not a tick cost.

## What this changes

The decision file recommends retiring a rumor once no living spreader holds it. That remains the right
fix **for memory**, and it remains a decision — `alive` is documented as "the subject is still in the
world", three places read it, and a host reading `RumorBook` sees it. What changes is the *urgency*: a
4× tick blowup would justify a breaking semantic change; 5 MiB over a long large run does not.

## Literature

No new model. The stifling rule is Daley & Kendall 1964 (DOI 10.1038/2041118a0), already cited at
`rumor_dk::on_contact`; this iteration is about a measurement the loop made of its own code, not about a
model.
