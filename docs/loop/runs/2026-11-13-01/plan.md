# Plan: add `play`, a leisure activity you can walk to in one step

## Why this over the runner-up

Runner-up was retiring untellable rumors (93% of the book, a 4× cost growth). Lost on one sentence:
**`fun` at zero for a third of every run is the most visible wrong thing when you watch the society, and
last iteration established that no rate can fix it.**

## Design

A new starter activity, `play`:

```ron
id: "play", target: Person, duration_hours: 0.5,
considerations: [Need("fun"), Affinity, Distance],
satisfies: [("fun", 1.7), ("social", 0.3)],
interaction: Some("chat"),
```

Three choices worth stating:

- **`Person`, not `Place`.** That is the whole fix: a `Person` target costs one decision, a `Place`
  target costs a walk plus the competition for the place. `social` demonstrates the difference — it
  needs 2.78 uses a day and never falls short, while `fun` needed 1.33 and got 0.10.
- **No `Hours` gate.** `relax` keeps its evening window, because resting at the well at midday is
  odder than playing at midday; but a need whose *only* source is gated six hours out of twenty-four
  cannot keep up, and now it does not have to.
- **`satisfies 1.7` per game-hour.** `1.7 × 0.5 h = 0.85` per use against `fun`'s 0.8/day, so one session
  covers a day. **Design default** — no paper gives it — stated in the file.

`Affinity Linear(0.0, 0.8)` rather than `compliment`'s `Logistic(8, 0.7)` so play is available with
acquaintances, not only close friends: a village where you can only play with someone you already like
plays very little. `Distance Linear(-1, 1)` matches `chat`, so playing with someone across the village is
disfavoured.

## Also shipped

`SocietyDefs::sources_for(need) -> Vec<(activity, per-use)>`, so a host — and the test — can see every
activity that supplies a need. The last iteration added `best_source`, which hid the fact that `fun` had
exactly one.

## Touched

- `assets/starter.society.ron` — `play`.
- `src/defs.rs` — `sources_for`.
- `tests/leisure.rs` (new) — the shape and the budget.
- `tests/need_budget.rs` — the old assertion said "fun needs > 1 use a day"; it fired correctly when the
  imbalance was fixed, and now asserts the opposite for the three sized needs.
- `docs/research/needs-and-utility.md`, `docs/loop/needs-decision/over-subscribed-evening.md` (resolved),
  `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. The rate lives in the definitions file, where every other activity rate lives.

## Tests

| Test | What it proves |
|---|---|
| `a_cheap_source_of_fun_exists_alongside_relax` | `fun`'s best source needs ≤ 1 use a day, and `fun` has at least two sources |
| `every_need_is_now_covered_by_one_use_a_day` | `rest`, `food` and `fun` each need ≤ 1 use a day; `social` is the documented exception |

Written **red first**: `best source for fun is relax, 0.600/use -> 1.33 uses/day`.
