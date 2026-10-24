# Research: an observable that assumed starvation the model does not produce

## How it surfaced

Last iteration found one `measured` scenario with nothing asserting it. Auditing the rest, `need-cycle` has
been `pending` since November with a test that prints its numbers but says in its own header that the
observable **does not hold**. Worth reading what it claims.

> Every person's `rest` and `food` needs fall and rise again over a game-day: each crosses below 0.35 at
> least once and recovers above 0.7 at least once.

## Measured, per person, at the scenario's own settings

64 s/day, 80 000 ticks, 8 people, two seeds. The thresholds are the observable's own.

| need | lowest any person reached | cycles 8 people | seed 7 | seed 42 |
|---|---|---|---|---|
| `social` | 0.45 | no | 0 of 8 | **2 of 8** |
| `rest` | **0.51** | **no** | 0 of 8 | **0 of 8** |
| `food` | 0.00 | yes | **8 of 8** | **8 of 8** |
| `fun` | 0.00 | yes | **8 of 8** | **6 of 8** |

**`rest` never crosses 0.35 at any seed.** `food` always does. `fun` usually does — which the observable's own
correction (*"only `food` cycles"*) denies.

## Why `rest` cannot cycle

It is arithmetic, not a defect:

```
sleep:  0.15 rest per game-hour × 8 h  =  1.20 restored per sleep
rest:   decay_per_day                  =  0.70 lost per day
```

**A net `+0.50` per sleep.** So `rest` ratchets *up* until sleeping stops being chosen, and never needs to run
low. The observable assumed a need that starves; `rest` is over-supplied by a factor of 1.7.

`food` cycles because `eat` supplies `1.2 × 0.75 h = 0.90` against a `0.9/day` decay — exactly break-even, so
it oscillates around the decision threshold. `fun` is close behind.

## Why the observable is the thing that is wrong

The decays are cited (Maslow 1943) and the rates are **data** — a host sets them. Re-tuning `rest`'s decay to
make the observable true would be fitting the model to a claim written about a different pack, and five
iterations have changed that pack since. The claim is stale; the model is right.

So the observable is **restated** to a property that is true and worth checking:

> Every person's needs are **met and re-met**: over the run each need both falls below 0.7 and recovers above
> it, and no need is ever left falling with no activity to satisfy it.

That catches the defect the original was reaching for — a need drained with nothing that satisfies it — without
asserting a starvation that a generous rate legitimately prevents.

## Sources

No new model. Need decay is `Needs::decay` (Maslow 1943, cited at the formula); the rates are the
definitions file's. `docs/research/needs-and-utility.md` records the decision model.
