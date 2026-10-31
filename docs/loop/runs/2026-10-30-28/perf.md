# Perf: `Input::Stock` and the provisioning overlay

**Verdict: no cost on the default path, and none was measurable.** The default pack never names `Stock`, so the only code on its
path is one match arm in `signal_value` that is never taken, and the scoring loop is otherwise unchanged. A run of the default
pack is bit-identical to HEAD's (`series.csv` compared byte for byte over three game-days, `eval.md`), so whatever a tick costs
on HEAD it costs here. `Decision::stock` is the copy of the actor's `Stock` that `Surplus` already makes every decision; the
new input reads it and allocates nothing.

The overlay adds two considerations to `bake` (now four) and one to `eat` (now two), each a read of a `Vec<f32>` or a need level.
Against the other activities of the pack that is a handful of floating-point operations per person per decision, and the
overlay's town is cheaper in what it does (1 847 `eat` walks in thirty game-days against 3 307).

Command (release, this branch's `examples/benchmark`, the starter pack, `--ticks 200`, five reps, 24 threads, host load
average 10-17 when it started, other jobs of mine running for the first runs):

```sh
./target/release/examples/benchmark --people N --ticks 200
```

| people | fastest window (ms/tick) | median (ms/tick) | spread |
|---|---|---|---|
| 100 | 0.63 / 0.74 / 0.79 | 0.86 / 0.98 / 1.01 | 55-87 % |
| 1 000 | 32.5 / 34.6 / 43.3 | 55.5 / 64.4 / 277.6 | 59-202 % |

Every spread is far above the 20 % line, so these say little beyond the fastest window: **0.6-0.8 ms at 100 and 33-43 ms at
1 000**, which is at or under the 43.5 ms the v0.6.0 table records for 1 000 people (`docs/guide/concepts.md`). No before/after
binary pair was built: with identical code on the measured path, a pair on this host would have measured the host. No 5 000-person
run: the path is identical and the host's other jobs would dominate it.
