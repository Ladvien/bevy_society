# Research: an activity aimed at a person was cancelled on the tick it began

## Why this

Last iteration's top item was "`ActivityTarget::Home` ignores `Place::capacity`". Before building it I measured
*which* activities actually fail, by adding an `interrupted_by_activity` breakdown to `summary.json`. That one
diagnostic redirected the whole iteration: the biggest failure was not `sleep` at all.

## The breakdown, `village-evening`, seed 7, 17 game-days

| activity | interrupted |
|---|---|
| **compliment** | **1 045** |
| snub | 598 |
| work | 164 |
| sleep | 162 |
| eat | 118 |
| gossip | 112 |
| relax | 29 |
| play | 5 |
| chat | 4 |

`compliment` — a **person**-targeted activity — was **47% of all failures**. `sleep` was 7%.

## The cause, instrumented rather than guessed

Every exit path in `end_activities` was instrumented at once (suspension, timeout, target-gone, lost-perception).
All **1 045** took the same branch:

```
1045 WHY compliment pre-arrival timed_out=false gone=true
```

So the target was reported **gone** while standing in a list of 24 people. The check was

```rust
perception.persons.binary_search(&target).is_err()
```

`binary_search` requires the slice sorted by the key being searched. `Perception::persons` is filled in
`PersonIndex.sorted` order — by `PersonId` — and `PersonId` comes from an `On<Add<Person>>` observer, whose order
need not match spawn order. Printing the raw entity ids settled it:

```
FULL [4294967234, 4294967233, 4294967232, 4294967231, ...]
```

**Strictly descending.** A binary search over a descending vector finds almost nothing, so nearly every
person-targeted activity was cancelled.

## The fix

`!perception.persons.contains(&target)` — a linear scan, matching how every other lookup in this file handles the
index (`in_perception` uses `position()`). At 5 000 people it is 5 000 comparisons once per person-targeted
activity per tick, against a perception pass that already walks 12.5 million pairs.

## Measured

| | interrupted | share | `compliment` failures | friends | rivals |
|---|---|---|---|---|---|
| before | 2 237 | **22.8%** | 1 045 | 22 | 46 |
| after | **400** | **5.9%** | **11** | **31** | **38** |

Interruptions down **82%**, friends up **41%**, rivals down **17%**. Reverting the one line restores 2 237 and
1 045 exactly, so the measurement is causally tied to the change.

Total endings fall 9 808 → 6 768 because a compliment that died instantly and was re-chosen is now one completed
activity — fewer, longer things, which is the right shape.

## Sources

None. This is a data-structure bug: `binary_search` over a vector that is not sorted by the search key. The
observable it corrupted — a village whose people mostly complete what they start — is ordinary behaviour, not a
model claim.
