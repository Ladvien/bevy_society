# Perf

## The change

None to the model. `--friend-affinity` adds one `Option<f32>` to the harness's flags and one assignment.

## What the clamp would have cost

`clamp(-K, K)` instead of `clamp(-1, 1)` — the same single `clamp` call with different bounds. No new work.

The *behavioural* cost is the interesting one: mood -0.054 → -0.078, because fewer pairs reach the affinity
gated activates. That is a model consequence, not a compute cost.
