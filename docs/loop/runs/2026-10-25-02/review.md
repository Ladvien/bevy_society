# Review: a label is a latitude

Three passes, each with a written critique, before the verdict. Evidence is under `evidence/`.

## Pass 1 — Behaviour: does the change do what the paper and the scenario say?

**GREAT, for what shipped.** Social judgment theory says a judge about a person is a band, and
`tests/label_hysteresis.rs` holds the band at both ends of the stranger bar: promotion at
`stranger_familiarity`, demotion only at `stranger_familiarity - 0.02`. `evidence/{before,after}/summary.json`
at seed 7, same 67 200 ticks:

| | before (band 0.00) | after (band 0.02) |
|---|---|---|
| label changes | 2 291 | **1 953** (−15%) |
| friends / rivals | 18 / 42 | **18 / 42** |
| rumor reach | 0.542 | 0.542 |
| interrupted endings | 329 | 329 |

Nothing but the message stream moved, which is what a change to *when a message is sent* should do.

**But the honest headline is smaller than the first plan said.** The plan claimed "an order of magnitude".
The measured reduction is 15% on this instrument (2 867 → 2 315 on the longer probe at the scenario clock,
17%). The one place it is decisive is the low end, and there it is total: `Stranger -> Acquaintance` 828 →
**552** (276 ordered pairs, once each) and back 96 → **1**.

## Pass 2 — Zoom: is the band in the right place?

**GREAT, and this is the iteration's finding — found by zooming, not by planning.**

The first implementation put the band on every bar. It looked strictly better on the headline — 2 291 →
1 486 messages, −35% — and it was wrong:

| seed | band everywhere: friends / rivals | band on the stranger bar: friends / rivals |
|---|---|---|
| 7 | **20 / 48** | 18 / 42 |
| 42 | **16 / 42** | 14 / 35 |
| 11 | **10 / 49** | 9 / 40 |

The extra cut held friendships past the event that ended them. Reading the cause table is what caught it:
at the friend and rival bars the one-tick affinity step is a median **0.013** — one accepted `compliment`,
one witnessed deed — so those flips are events. At the stranger bar the step is **0.0002** — decay.
A band on an event is a lie; a band on decay is a fix. `evidence/plots/where.png` draws both.

## Pass 3 — Fresh eyes: does it read, and is the narrative explainable?

**GREAT.** `evidence/narratives/after.md` is Bram's day exactly as before — two sleeps, compliments, four
gossips in a row at 12:12–13:07, three snubs at 13:55–14:26 — and a host reading it can say why each step
happened. The message a host acts on now arrives 15% less often and no longer reports a value a thousandth
from a line as a relationship event.

**What a stranger would still complain about, correctly:** 1 953 messages in 17 game-days is one every
12.6 game-minutes, and 85 pairs still change more than ten times. That remainder is real dynamics — the
village's friend count is genuinely unstable — and it is item 1 on the list below, not something this
change should have hidden.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## A test deleted, not kept

The first version of `a_held_category_still_promotes` asserted that a tie at rival affinity, held as a
Friend, becomes an *Acquaintance*. It failed, correctly: it becomes a **Rival**, in one tick. The test was
asserting the old single-line implementation's shape rather than the intent. Replaced with the assertion
that matters — promotion to another category is never held — and the failure is what found the wording.

The village test's first version asserted **zero** demotions with a band. Measured, it is **one**, at tick
6 016 of 6 400, for a pair that genuinely went unseen long enough to decay past the band. Pinning zero
would have been a lie; the assertion is now "fewer than a twentieth of the baseline, and at most one".
