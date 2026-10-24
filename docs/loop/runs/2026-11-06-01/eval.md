# Eval

## The cap

`tests/knowledge_bound.rs`, capacity 32, 12 people, 3 000 ticks.

| | max rumors one person held |
|---|---|
| before | **287** |
| after | **32** (exactly the cap) |

And in the long benchmark, `knowledge mean` holds at 64.0 from tick 500 through 5 000 while the book
grows 848 → 8 357. `Ties` settles at Dunbar's 150. Two bounds hold; the book does not, and that is the
filed decision.

## Behaviour at the defaults is unchanged

`knowledge_capacity` is 64 and the village's book never exceeds 34 rumors, so the cap cannot bind.
Verified rather than assumed:

| | |
|---|---|
| `series.csv` rows identical | **4 200 / 4 200** |
| `narrative.txt` first 30 lines | **identical** |

So this is a memory and cost change, not a behaviour change, and the plot is a straight line because
nothing moved.

## The harness column

| scenario | column before | column after |
|---|---|---|
| `rumor-crosses-a-cluster` | 1 change, flat 0.333 | **6 changes**, 0 → 0.333 over ~350 ticks |
| `village-evening` | flat 0.417 | 0.042 = 1/24, flat |

`evidence/harness-fix.png`. The cluster diff is the whole point: the old column was a stranger's deed
from tick 0, and the planted rumor's real curve is a diffusion over 350 ticks.

The village figure is *worse* than previously reported and is a separate finding: its origin gossiped
twice in 3 600 ticks, so the scenario asks an arbitrary villager to be the town crier. Documented at
the plant site; not fixed here.

## Perf

| | before | after |
|---|---|---|
| 1 000 people, 200 ticks | 14.96 ms | **14.96 ms** |
| 1 000 people, 3 000 settle + 2 000 ticks | **63.36 ms** | **61.91 ms** (−2.3%) |
| 100 people, 200 ticks | 0.556 ms | 0.556 ms |

The 2.3% is what the `Knowledge` cap is worth on its own. The remaining 4× growth is the rumor book,
and the probe shows it: 8 357 rumors at tick 5 000, every one of them walked for `RumorReach` on every
tick.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **22** binaries (21 + `knowledge_bound`) |
| `cargo test --no-default-features` | 22 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero, unchanged. One public field added (`knowledge_capacity`); no item removed.
