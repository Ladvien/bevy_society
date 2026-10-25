# Performance: a friendship is held through a shallow dip

`Tie::refresh_label` already ran once per tie per tick. The change adds one branch: when the promotion test
has failed *and* the tie is currently a `Friend`, two more comparisons against `friend_band`. No allocation,
no draw, no iteration order, no new state.

| persons | before, fastest window (ms/tick) | after, fastest window (ms/tick) | spread after |
|---|---|---|---|
| 100 | 1.125 | **0.737** | 25.1% |
| 5 000 | 527.471 | **495.655** | 12.3% |

**Inside the noise, and in the right direction.** At 100 people the after figure is 34% *lower* than the
before figure; at 5 000 it is 6% lower. Neither is this change: the two runs were on the same shared M1 Max
at different moments, the 100-person spread was 25.1% within its own three reps, and the previous iteration
measured the same 15-35% wobble on an *unchanged* code path. What can be said honestly is that a branch on
two `f32` comparisons beside a 5 000-person neighbour scan costs nothing measurable, and nothing regressed
in the direction that would show a cost.

The 1 000-person point is not re-measured here: the previous iteration recorded 31.954 ms and the machine's
spread at that size was 37.9%, so another sample would say more about the load than about the code.

`village-evening` wall-clock is unchanged at about 3 minutes per 67 200 ticks, which is the run the evidence
came from.
