# Perf

## The change

One `powf` per witness per deed. A deed fires on an accepted interaction with a `deed` field, which in the
starter pack is one interaction; the witness list is already built and already iterated.

## Behaviour

| | before | after |
|---|---|---|
| mean trust at an exchange | 0.345 | **0.435** |
| accept rate | 0.533 | **0.558** |
| `village-evening` mean pleasure, seed 42 | −0.1268 | **−0.0456** |

More interactions are accepted, so more exchange work runs. The cost moves with the behaviour the change
intends, and a `powf` against a decision pass of millions of consideration evaluations is not measurable.
