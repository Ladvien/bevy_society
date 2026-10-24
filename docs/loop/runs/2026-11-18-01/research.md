# Research: the row that was not a sample

## The claim under test

`need-cycle`'s observable says *every person's* needs cross below 0.35 and recover above 0.7. Last
iteration marked it `measured` on a **mean across eight people**, citing `[0.86, 0.81, 0.58, 0.79]` and "all
four cycle". A mean is not what the observable says, so this iteration measured per person.

## Per person, at the scenario's own settings

64 s/day, 8 people, 80 000 ticks, seed 7 — each person's lowest level on each need:

| need | lowest any person reached | cycles for all 8? |
|---|---|---|
| social | **0.39** | **no** |
| rest | **0.50** | **no** |
| food | 0.00 | **yes, once a day** |
| fun | 0.12 | most, but 2 of 8 never |

So the observable's first sentence is **false for three of four needs**. Two needs never come near 0.35,
and they are not supposed to: their rates are generous, which is the behaviour the starter pack intends.

## Why the mean said otherwise

The evidence series recorded a row at **tick 0**, before `size_person_state` fills `Needs`. Every need read
`0.000` in that row, once. `"crosses below 0.35"` was therefore true of the series — on the strength of a
state the society never has.

A mean over eight out-of-phase sawtooths cannot fake a 0.000 that none of them reached; the trough of a mean
is never below the lowest individual. So the artefact had to be the *row*, not the averaging, and it was:
exactly one sample at index 0, tick 0, for all four needs.

`evidence/tick-zero.png` shows it — a single vertical line dropping to zero at tick 0 in the left panel, and
a corrected series in the right where `food` (green) is the only need that reaches low.

## What the fix is

The row at tick 0 is no longer recorded. That is a harness fix, not a model fix: the society's first tick
sizes its state, so a row taken before it describes nothing.

`Need` counting also stopped counting a person whose `Needs` is empty — the component exists before it is
sized, and counting it averaged a real 1.0 down toward 0.

## What this means for the scenario

It stays **`pending`**, and the note in the file now says why with the per-person numbers. Three needs
staying above 0.5 is correct behaviour at these rates; the observable assumes starvation that does not
happen. Whether the rates should be tightened, or the observable should ask only that needs *recover*,
is a design question — a `needs-decision` candidate rather than a code defect.

## Literature

No new model. The need cycle is `docs/research/needs-and-utility.md`. This iteration is a measurement
correction: the same class as the `4×` rumor claim and the `RumorId(0)` reading, in a third place.
