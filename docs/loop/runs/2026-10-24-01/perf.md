# Perf

Not measured this iteration: the change adds one `eval_f32` call per script consideration per
decision, and a decision runs only for a person with no activity, so the cost is bounded by the number
of `Script` considerations a definition declares. The starter pack declares none.

`scale-1000` in `scenarios.yaml` remains `pending` and is the fifth candidate for a later iteration.
