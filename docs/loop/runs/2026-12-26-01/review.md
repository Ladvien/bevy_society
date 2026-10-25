# Review

## Pass 1 — Behavior: does the tool now report something quotable?

**GREAT.** `--reps` produces min / median / max / spread, the three are ordered, and `spread_pct` is derived from the
reported windows rather than being a printed constant — the test recomputes it and fails if they disagree. Reverting
to a single mean fails with *"the benchmark line has no `ms_per_tick_min`"*.

## Pass 2 — Zoom: the three answers disagree by 2x

**GREAT, and uncomfortable.** `evidence/scale.png`, left panel: the guide's retracted line sits **below the measured
minimum at every population**, and the median sits well above it. At 1 000 people the three answers are 13.5, 28.4
and 60.4 ms — a 4.5x range, all of them "the crate's cost".

Right panel: across separate invocations the fastest window repeats to **0.4-0.7%** while the median repeats to
18-22%. So the minimum is a property of the code and the median is a property of the machine's load. The guide was
quoting something in the third category — a single noisy window — as if it were the first.

Nothing here oscillates, saturates or drifts: the growth is monotone in population and the shape is unchanged from
what the guide claimed. What changed is the *scale*, and it changed by 2x.

## Pass 3 — Fresh eyes: why did this stand for nine iterations?

**GREAT, and the answer is a process failure rather than a missing measurement.**

Three things conspired:
1. the tool printed one mean, so a noisy reading was indistinguishable from a measurement;
2. every iteration recorded the doubt instead of replacing the number — nine notes, no fix;
3. nothing asserted the tool's output shape, so the missing spread was never anyone's failing test.

The lesson generalises: **a doubt recorded is not a doubt resolved**, and a standing item that says "the machine
was too busy" is a diagnosis of the tool masquerading as a diagnosis of the environment. Last iteration I wrote the
right instruction; this one followed it in about twenty minutes.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## Not shipped, deliberately

- The 5 000-person figure. It does not complete at `--reps 7 --ticks 300` inside a 30-minute deadline on this
  machine, so it is removed from the guide rather than replaced with a guess.
- Any change to the simulation. This iteration touched the measurement tool and the document that quotes it.
