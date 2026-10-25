# "Why did they do that?"

A villager walks away from a conversation to sleep, and your player asks why. `Doing` says *what* a person is doing. `DecisionTrace` says what else they were weighing and by how much they lost.

It is a bounded record of the scores behind every daily-life decision. The design follows Game AI Pro 3 ch. 6, *Debugging AI with Instant In-Game Scrubbing*: recording must have minimal cost, **fixed** memory, and "**no impact on the outcome of simulating objects**". The last is tested: `tests/trace.rs` runs the same seed with and without a trace and asserts the same 3 000-tick sequence of chosen activities.

## Turn it on

The trace is a `Resource` the crate records into **only if you insert it**. No resource, no recording, no cost.

```rust,ignore
// 64 decisions of up to 8 options each: the default.
app.insert_resource(DecisionTrace::default());

// Or pick your own ring: 256 decisions, the best 4 options of each.
app.insert_resource(DecisionTrace::with_capacity(256, 4));
```

The ring has a fixed capacity. When it is full the **oldest** decision is dropped, which is what a host that only looks at the last few minutes wants, and what keeps memory flat over a long run. `with_capacity(0, _)` records nothing; `options_per_decision` is clamped to at least 1.

## Read it

```rust,ignore
fn why(trace: Res<DecisionTrace>, names: Query<(Entity, &Name)>) {
    let names: Vec<(Entity, String)> = names.iter().map(|(e, n)| (e, n.to_string())).collect();
    for decision in trace.recent().take(5) {
        info!("{}", decision.describe(&names));
    }
}
```

The lines read like this (the numbers here are illustrative):

```
tick 412: Bram chose sleep (0.912), the best option
tick 413: Alys chose chat (0.480), 21% below work (0.608)
```

| on `DecisionTrace` | returns |
|---|---|
| `recent()` | every `Decision` held, **newest first**; double-ended. |
| `last_for(person)` | the newest `Decision` for one person, if the ring still holds one. This is the "why did *this* person do that" call. |
| `len()`, `is_empty()` | how many decisions are held. |

## A `Decision`

| field | holds |
|---|---|
| `tick` | the society tick it ran on |
| `person` | the `Entity` |
| `name` | the person's `Name`, copied at the time, because the entity may despawn before you read the trace |
| `signals` | the host-written signal values the decision scored with, indexed by `SignalId`; empty if the person had no `Signals` |
| `options` | every option that scored above zero, **best first**, cut to `options_per_decision`; the idle fallback is included when nothing else scored |

| method | answers |
|---|---|
| `chosen()` | the `ScoredOption` taken, or `None` if no options were recorded |
| `best()` | the highest-scoring option. **Not necessarily the chosen one**: selection is a weighted random draw over the viable tier, so a lower scorer can win. |
| `took_the_best()` | whether the chosen option was also the best |
| `shortfall()` | how far the chosen option fell short of the best, as a fraction of the best score: `0.0` when it was the best, `1.0` when it scored nothing. |
| `describe(&[(Entity, String)])` | a one-line account for a log or HUD. The name comes from `name`, then from the list you pass, then falls back to "someone". |

`shortfall` is deliberately not "chosen minus runner-up": a weighted draw can legitimately pick the fifth-best option, which made the first version of this report a negative "margin" about a score that had nothing to do with the choice.

## A `ScoredOption`

| field | holds |
|---|---|
| `activity` | the activity's name as authored |
| `score` | `weight · Π compensated consideration`, in `[0, 1]` |
| `chosen` | whether it was the one taken |

A score is what `docs/guide/authoring.md` describes: the activity's `weight` times the product of its considerations, each passed through its curve. A small `shortfall` says the choice was close; a large one says the person took something unlikely, which is what the draw is for.

## Next

- [`authoring.md`](authoring.md): the considerations and curves that make up a score.
- [`concepts.md`](concepts.md): ticks, and why the draw is deterministic for a seed.
