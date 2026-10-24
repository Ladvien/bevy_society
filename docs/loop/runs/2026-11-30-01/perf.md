# Perf

## The change

One `Option::is_none` check on the reject path; one `Tuning` field lookup. No new iteration, no new
allocation, no new query.

## Behaviour

| scenario | before | after |
|---|---|---|
| `village-evening` mean pleasure | -0.150 | **-0.154** |
| `village-evening` mean needs | 0.599 | 0.596 |
| `village-evening` dominance | +0.069 | **+0.119** |

Dominance rose because a refused greeting no longer stacks a large distress on top of the definition's
small one; the smaller negative emotion leaves the PAD virtual centre slightly less negative.

No scenario slowed: the change is a branch that skips a message write.
