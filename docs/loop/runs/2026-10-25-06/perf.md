# Performance: a task is not a skill

`Input::Skill` adds one array index and one trait read per consideration, and the trait read already existed
(`Signal::Trait`). No allocation, no draw, no stored state.

| persons | before, fastest window (ms/tick) | after, fastest window (ms/tick) | spread after |
|---|---|---|---|
| 100 | 0.737 | **0.730** | 10.1% |
| 5 000 | 507.920 | **508.798** | 10.5% |

**Unchanged, to three significant figures.** At 100 people the difference is 1%, at 5 000 it is 0.2%, and the
spreads are 10.1% and 10.5% — smaller than the change being measured. A skill level is `traits.value(group)`
on data already in the decision's cache line; there is nothing here to cost anything.

The 1 000-person point is not re-measured: the machine's spread there has been 24-44% across iterations, so
another sample would describe the load.
