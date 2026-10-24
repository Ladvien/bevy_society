# Perf

## The change

`grudge` replaces `.sum()` with `.fold(0.0, f32::max)`. Same pass over the same memories, one comparison
instead of one addition. The refusal fix removes an index into `defs.deeds` on the reject path.

## Behaviour

| scenario | before | after |
|---|---|---|
| `village-evening` travelling | 58.9% | 59.0% |
| `village-evening` need mean | 0.545 | **0.542** |
| `rumor-crosses-a-cluster` snub share | 50.3% | **26.0%** |

The cluster's snub reduction means *fewer* interaction resolutions per tick, so if anything the scenario is
cheaper.

## Standing benchmark

Not re-run: the change is one `f32::max` per memory in a function that already iterated them. Recorded
rather than measured.
