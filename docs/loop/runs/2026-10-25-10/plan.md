# Plan: a full place costs something

## Pick

**Pick:** replace the capacity **veto** on `Place(tag)` and `Bound` with a **congestion price** — a full place
stays a candidate at a reduced score — keeping `capacity: 0` as a hard prohibition.

**Why it beats the runner-up** (the price model, deferred twice): this is the defect last iteration proved is
load-bearing for believability — it turns a village of 48 people into one where a fifth of all pairs are hostile —
and it is one scoring rule. The price model is a number nobody can see.

## Design

For a place-targeted option whose place has `used` people aiming at it and `capacity` seats:

```text
fill   = used / capacity                         (capacity > 0)
score *= 1 - crowding_penalty * fill,  clamped to [0, 1]
```

- `crowding_penalty = 0.0` → capacity has no effect on score (full places are free).
- `crowding_penalty = 1.0` → a full place scores 0, which is today's veto **only at exactly full**; above full the
  clamp holds it at 0.
- Shipped default: measured, not guessed.

**`capacity: 0` stays a prohibition.** A place that seats nobody has no room at any price, so it is still dropped.
`tests/bound_places.rs` asserts this and must keep passing.

**Where it is applied.** In target expansion, not in a consideration: the price of a target belongs to the target.
`Place(tag)` and `Bound` already compute `used`/`capacity`; multiplying the returned score there covers every
activity, including ones a host wrote, with no definitions-file change. `Home` and `Itself` are unchanged — a home
has no alternative, so pricing it only pushes people away from sleeping at home.

### Touched

| file | change |
|---|---|
| `src/core.rs` | `Tuning::crowding_penalty` (design default, measured) |
| `src/behavior.rs` | the score multiplier in target expansion; keep `capacity: 0` a veto |
| `tests/crowded_place.rs` | **new**: a full place is cheaper not impossible; capacity 0 still never; an uncrowded village is bit-identical |
| `docs/guide/authoring.md` | `capacity` documented as a price, with the paper |
| `examples/evidence.rs` | none; `--people` is the measurement |

### Tests

1. **A full place is still chosen when it is the only option.** One person, one place with `capacity: 1` already
   occupied: the activity still starts, at a lower rate than with room.
2. **An emptier place wins a tie.** Two places, same tags, one full: the person goes to the one with room.
3. **`capacity: 0` is still never**, exactly as `a_full_bound_place_gives_no_candidate` requires.
4. **`crowding_penalty = 0.0` restores today's ranking** (score unaffected by fill).
5. **An uncrowded village is bit-identical**: `village-evening`, seed 7, 24 people, identical `summary.json` to the
   commit before the change. This is the regression test that protects every existing scenario.
6. Determinism: no draw is added.

### Measurement

`village-evening`, five seeds, populations 24 / 36 / 48, the same harness, before and after:

| what | target |
|---|---|
| `work` share of person-ticks | ~10% at every population |
| rival density | 7-8% of directed pairs at every population |
| 24-person `summary.json` | identical (nothing is full there) |

`--people` scales people and not the village, which is the trap that produced last iteration's false scaling law;
this iteration measures the **oversubscribed** village on purpose, because that is the case the price is for, and
reports it as an overfilled village rather than a bigger one.

## Not doing

- **Scaling the harness's capacity with `--people`.** That is option 1 of the note and it overshoots to 0% hostile;
  it would also delete the very case this change exists to make behave.
- **`Home` pricing.** Left unconditional: a home has no alternative.
- **The price model**, still item 2 in the log.
