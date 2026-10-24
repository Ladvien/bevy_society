# Perf

No library code changed. The only runtime change is in the evidence harness (`summary` reads the
planted rumor's id instead of `RumorId(0)`), which runs once at the end of a scenario.

The three new tests add wall-clock to `cargo test`, not to any scenario:

| test | time |
|---|---|
| `deed_response` | 0.04 s |
| `opinion_convergence` | 8.8 s (12 000 ticks, required by the corrected observable) |
| `need_cycle` | 20.0 s (three runs of 20 000 ticks) |

`need_cycle` is the slowest test in the suite. It is slow because a need cycle needs four game-days to
show at all and the crate's day is 4 096 ticks; cutting it to 5 000 ticks (the scenario's original
figure, 1.2 days) would make it pass or fail on nothing.

Standing benchmark, unchanged by this iteration:

| Persons | ms per society tick |
|---|---|
| 100 | 0.556 |
| 1 000 | 14.96 |
| 5 000 | 277.0 |
