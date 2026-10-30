# 2026-10-30-26 — perf

The change touches one per-tick path: `evaluate_triggers`' `NeedBelow` block. With no `NeedBelow` trigger
(the starter pack and the village have none) it builds an empty `lines` vector and `retain`s an empty set,
so the benchmark's society does the same work as before. With triggers it is one `BTreeSet` insert or
remove per crossing and one `retain` pass over the (small) set of disarmed people per tick.

Paired runs of `examples/benchmark` (1 000 people, 60 ticks, settle 40, 3 reps, starter pack), alternating
the binary built from `300c52b`'s `rules.rs` and `save.rs` (before) with this change (after), on a host at
load 48 (CI runners and other loops):

| pair | before median ms/tick (spread) | after median ms/tick (spread) |
|---|---|---|
| 1 | 32.6 (47%) | 35.1 (88%) |
| 2 | 37.6 (68%) | 28.2 (124%) |
| 3 | 32.2 (62%) | 34.8 (64%) |

Every spread is larger than any difference between the two columns, and the sign flips between pairs, so
this is "no measurable change", not a speed claim either way. No scenario that does not use `NeedBelow`
can be slower by more than noise on this host; a quiet-host re-measure is still on the backlog.
