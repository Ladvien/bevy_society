# Plan: name a need whose sources cannot meet its decay

## Why this over the runner-up

Runner-up was retiring untellable rumors (93% of the book, a 4× cost growth). Lost on one sentence:
**the log's #1 was `fun` at 35% and the honest answer turned out to be "the crate cannot fix this from a
rate", which is itself the finding the next iteration needs.** Naming it costs one warning and a test;
guessing at it costs a shipped default that makes the society worse.

## The failed fix, recorded rather than hidden

Attempted first: `relax`'s `satisfies` 0.4 → 0.55, so one evening session pays 0.825 against `fun`'s
0.8/day decay.

| | before | after |
|---|---|---|
| `fun` at zero | 33.8% | **18.8%** |
| `fun` mean | 0.061 | **0.051** |
| mean of all four needs | **0.127** | **0.118** |

The zero fraction halves — the metric the log named — but every other need falls and the total goes
**down**. The village is over-subscribed: 24 people, four places, one evening. Raising one supply moves
starvation around rather than removing it. Reverted.

## Design

`BehaviorPlugin::finish` gains `warn_unbalanced_needs(defs)`, beside `warn_short_activities`,
`warn_unscored_fields` and `warn_short_travel_reach`. For every need it computes the best
`satisfies · duration_hours` over all activities and warns when `decay_per_day / best > 4`.

`MAX_USES_PER_DAY = 4.0`, not 1.0, because the crate cannot see a source's *cost*: `social` needs 2.78
chats a day and is served, since a `chat` targets a `Person` and costs three ticks. The check catches
only what no activity shape makes plausible.

Two read-only accessors on `SocietyDefs` make the same arithmetic available to a host:
`need_decay(name)` and `best_source(name) -> (activity, per-use)`.

## Touched

- `src/behavior.rs` — `warn_unbalanced_needs`, `MAX_USES_PER_DAY`.
- `src/defs.rs` — `need_decay`, `best_source`.
- `tests/need_budget.rs` (new) — two tests.
- `docs/loop/needs-decision/` gains the over-subscription note.
- `CHANGELOG.md`, `docs/loop/log.md`, `docs/research/needs-and-utility.md`.

## New `Tuning` fields

None. Nothing about the simulation changed; this iteration is a diagnostic and a record of a negative
result.

## Tests

| Test | What it proves |
|---|---|
| `every_need_has_a_source` | each starter need has an activity that supplies it, and the budget is printed |
| `the_budget_exposes_the_one_need_that_starves` | `fun` needs > 1 use a day, which is the state the crate now warns about; the test fails the day that stops being true |

Both are arithmetic over compiled definitions, so they are instant and cannot flake.
