# Perf

Not measured: the changes are one arithmetic expression in `witness_reaction`, one per-candidate
binding in `decide_activities`, and one authored consideration. The binding does one additional
`PersonId` lookup per Person-target candidate, which is a binary search over the person index.

`scale-1000` remains `pending`.
