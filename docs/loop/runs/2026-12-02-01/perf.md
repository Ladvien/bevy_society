# Perf

## The change

Two `on_accept` emotion entries in the definitions file. At runtime an accepted `chat` or `gossip` writes two
extra `Appraise` messages instead of none, and `Emotions::appraise` merges them into an existing entry of the
same kind and target, so the emotion list stays as short as it was.

## Behaviour

| | before | after |
|---|---|---|
| `village-evening` mean pleasure | −0.122 | **−0.066** |
| `village-evening` mean needs | 0.603 | 0.607 |
| travelling | unchanged | unchanged |

No scenario slowed: the added work is two message writes on an interaction path that already writes needs,
tie deltas and memories.
