# NeedBelow in a 12-person village, 3 game days, before (300c52b)

seed 7, trigger `NeedBelow(food, 0.7)`, `then: ""`. "crossings" is an independent oracle that arms and disarms each person on their own (disarm when food < 0.7, re-arm above 0.75).

```
people=12 days=3 ticks=768
per-day firings: [0, 1, 0]
total firings: 1
total independent crossings (oracle): 12
ticks with a firing: 1, most firings in one tick: 1
person 00: fired   1  crossings  1  food 0.28..1.00
person 01: fired   0  crossings  1  food 0.28..1.00
person 02: fired   0  crossings  1  food 0.28..1.00
person 03: fired   0  crossings  1  food 0.28..1.00
person 04: fired   0  crossings  1  food 0.28..1.00
person 05: fired   0  crossings  1  food 0.28..1.00
person 06: fired   0  crossings  1  food 0.28..1.00
person 07: fired   0  crossings  1  food 0.28..1.00
person 08: fired   0  crossings  1  food 0.28..1.00
person 09: fired   0  crossings  1  food 0.28..1.00
person 10: fired   0  crossings  1  food 0.28..1.00
person 11: fired   0  crossings  1  food 0.28..1.00
```

Issue case 1 (every-tick firing), from `tests/need_below.rs` run on the same code: two people at 0.2 and 0.9, 10 ticks, `NeedBelow(social, 0.3)`: 10 firings for person 0 (one per tick), expected 1. A trigger with `persons: [1]` and both people at 0.2: 0 firings, expected 1.
