# Perf

## The change

One extra pass over `Perception.contact` per society tick, inside `Relate`. It reads `Knowledge` per person and
calls `best_telling`, which is a scan of that person's known rumors (`knowledge_capacity` is 64).

## Cost

No measurable change to scenario wall time: the village run stays at ~47 s before and after (`/tmp/base` and
`/tmp/cg3`), and the added work is bounded by `Σ|contact|` which the exposure pass already walks in the same
system set.

Nothing was slowed that the change did not target: the two scenarios it *could* have hurt are measured above,
and rumor reach improved rather than regressed.

## The pacing trade

At `2.0` days the pass fires rarely; at `6.0` it fires so rarely that reach is identical to the system being
absent (0.583), i.e. pure cost. At `0.5` it fires enough to burn spreaders and reach *falls*. The chosen setting
is the one where the compute buys a measured change.
