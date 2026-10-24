# Research: a coin flip decided which tavern you walked to

## The defect

`targets_for` returns **every** `Place` carrying the activity's tag:

```rust
ActivityTarget::Place(tag) => places
    .iter()
    .filter(|place| place.used < place.capacity && place.tags.iter().any(|t| t == tag))
    .map(|place| DoingTarget::Place(place.entity))
    .collect(),
```

A `Place` target has **no `Distance` term** in the decision, so every candidate scores identically and they
all tie. The tie was broken with a uniform pull from the seeded RNG:

```rust
let index = rng.0.below(chosen.targets.len().max(1));
let target = *chosen.targets.get(index)?;
```

**Measured, in isolation**: two taverns serving `food`, one **1 m** away and one **9 m**, 6 000 decisions —
**3 020 near, 2 979 far**. A coin flip, in a village where the far tavern costs fifteen times the walk.

## What it costs the village

Last iteration measured **62.5% of all person-ticks spent travelling**. Part of that is short activities
(the decision file `travel-cost-is-not-scored.md`), and part is this: choosing a far place when a near one
serves the same need.

## The fix

**Nearest wins for a `Place`, the uniform pull stays for a `Person`.**

- A `Place` is a fixed destination: the tavern will be in the same place tomorrow, so there is nothing to
  randomise for variety. Preferring the closer one is what a person does.
- A `Person` is not: chatting with whichever neighbour comes up is variety, and forcing the nearest would
  make a villager always talk to the same person.

Ties on distance fall back to the seeded draw, so the result stays deterministic — `tests/determinism.rs`
passes 2/2.

## Measured

| | random tie | **nearest tie** |
|---|---|---|
| 6 000 decisions, 1 m vs 9 m | 3 020 / 2 979 | **5 999 / 0** |
| `village-evening` travelling, seed 7 | 62.5% | **58.9%** |
| need mean, seed 7 | 0.526 | **0.545** |
| travelling, seed 42 | — | 58.8% |
| need mean, seed 42 | — | 0.550 |

**The village effect is small and that is worth stating plainly**: its four places sit within about 13 m of
each other, so "near" and "far" rarely differ by much. The defect is unambiguous in isolation and modest in
this layout; a village with a far market and a near one would feel it much more.

## Literature

No new model. `Signal::Distance` is already an input of the utility decision (`docs/research/needs-and-utility.md`,
Game AI Pro 3 ch. 13 on response curves over normalised inputs). What was missing is that a `Place` target
was never scored on it at all — the signal exists, the tie did not consult it.
