# Plan: a meal is what was on the shelf

## The design, in one paragraph

`arrive_and_satisfy` (src/behavior.rs) pays `satisfies · dt_hours` every tick a person is at their target.
`produce_and_consume` (src/economy.rs) takes an activity's `consumes` when it **completes** and reports the
amount to nobody. The two disagree about what a meal is, and in a famine the economy says "nothing changed
hands" while the needs model says "fed". Scale the `satisfies` payout by the fraction of what the activity
asked for that the stock actually yielded: `factor = min(1, taken / asked)`, `1.0` for an activity that
declares no `consumes`.

## Where the factor is computed, and why there

Two candidate sites:

1. **In `arrive_and_satisfy`, reading `Stock` each tick.** Costs a query on the largest system in the crate,
   and needs `take` to know the *total* the activity will consume, pro-rated over its duration. A meal
   interrupted half way must not have taken a whole loaf.
2. **In `produce_and_consume`, at completion, as the payout's own gate.** The consumption is already there
   and already knows exactly what `Stock::take` returned.

Site 2 is right, and it moves the payout with it: `satisfies` stops being paid per tick in
`arrive_and_satisfy` and is paid at completion, scaled, beside the goods. That is one path, not two, and it
is what the crate already says about an interrupted activity — "an interrupted task produces nothing, the
same rule `satisfies` follows" (`src/economy.rs`, on `produce_and_consume`). The current code pays
`satisfies` per tick for a task that will be interrupted, which contradicts that sentence in the same
crate.

**But moving the payout wholesale is a bigger change than the defect.** Paying at completion means an
8-hour sleep pays nothing until it ends, so a person who wakes early gets nothing for six hours slept, and
every need's series changes shape — a step at completion instead of a ramp. That re-shuffles every
measurement in the log for a defect that lives in exactly one activity. **Rejected.** The payout stays a
per-tick ramp; only the *scale* changes, and it is read from what the stock will actually yield.

So: `arrive_and_satisfy` computes, once per tick per arrived person,
`factor = min(1, available / asked)`, where `available` is the person's `Stock` for every good the activity
declares, and `asked` is what the activity declares. For an activity with no `consumes`, `factor` is `1.0`
and no `Stock` is queried.

**A subtlety that decides correctness:** a partial store must be divided across the meal's ticks, not
multiplied into it. A person holding `0.5` of the `1.0` a meal asks for is fed at `0.5` for the whole meal.
That is what `min(1, available/asked)` gives, and it is stable tick to tick because `Stock` is only
debited at completion — the same number every tick of that meal. Deterministic, no draw, no accumulation.

**What this does not model, and says so:** if two meals overlap with one loaf between them, both read
`1.0` from the same stock and both are fed in full. The crate has no reservation system and inventing one
is a schema-sized change (who holds a claim, and what clears it). `docs/guide/authoring.md` says a `Stock`
is debited when the activity **completes**, which is the honest description of the model; a host that needs
reservations needs a new mechanic, recorded as remaining work rather than guessed at here.

## Tuning fields

**None.** `factor = taken / asked` is authored in the definitions file. A penalty rate would be a number
with no source inventing a curve where the authored quantity already gives one.

## Modules touched

| file | change |
|---|---|
| `src/behavior.rs` | `arrive_and_satisfy` scales its `satisfies` payout by the store's fraction; a `Stock` query added; the doc comment says the rule and its source. |
| `src/economy.rs` | No behavioural change; the doc on `produce_and_consume` gains the sentence that `satisfies` is now the same rule from the read side. |
| `src/defs.rs` | `ActivitySpec` needs the `consumes` list reachable from `arrive_and_satisfy` — it already is (`spec.consumes`), so nothing. |
| `tests/economy.rs` | New: a fed meal pays in full, an empty shelf feeds nobody, half a loaf feeds half. |
| `docs/guide/authoring.md` | The `satisfies` contract gains the new sentence; the current one ("an empty store never stops anyone eating") is now half a truth. |
| `docs/research/needs-and-utility.md` | The need budget table's "per use" column is a *fed* use; say so. |
| `CHANGELOG.md` | One entry under Unreleased/Added, citing the measurement. |

## Tests

1. **`a_meal_without_bread_feeds_nobody`** — a village with `bake: 0` and an `eat` that pays `food`: `food`
   must fall to zero and stay there, where today it holds at the fed level. This is the defect as a test.
2. **`a_fed_meal_pays_in_full`** — the same village with bread available: `food` rises by exactly
   `rate · duration_hours` per meal, unchanged from today. Pins the bit-identity claim.
3. **`half_a_loaf_feeds_half`** — one villager seeded with exactly half the meal's `asked`: the food gained
   is half the full-meal gain. Pins the linear shape rather than a threshold.
4. **`an_activity_that_consumes_nothing_is_unaffected`** — `satisfies` on a consumeless activity is exactly
   what it was. Pins that the change is inside one activity's arithmetic.
5. **Determinism** — two identical runs, identical `food` series. The factor is a pure function of `Stock`
   and the definitions, and `Stock` is debited in a fixed order, so this should hold by construction; the
   test is what makes "should" checkable.

## Verification

- Both feature sets, `clippy --all-targets --all-features -D warnings`, `fmt --check`, `doc -D warnings`.
- `examples/evidence.rs` at seed 23 / 24 people (the price pass's seated run) before and after, on the
  **same seed and pack**, into `evidence/before` and `evidence/after`.
- `perf.md`: the factor is one multiply per arrived person per tick and one `Stock` read for activities that
  consume; measured at 100 / 1 000 / 5 000 people against the pre-change binary.

## The risk, named

**`food` may now starve systematically**, because the village bakes 4.8 loaves against 21 meals and 77% of
meals were being conjured. Feeding the truth may take `food` to the floor and keep it there for a third of a
run. That is the *honest* reading of the pack the village was authored with — but a village where everyone
is permanently starving is as wrong as one where nobody can be. If the measurement shows that, the finding is
that **the pack's supply and demand disagree by 4×** and the fix is a pack change (`bake` makes more, or
`eat` asks for less), which is data and therefore the host's business, not the crate's. That is measured
and reported, not pre-emptively tuned.
