# Review

## Pass 1 — Behavior: is the observable true, and is it now checked?

**GREAT**, and the answer was not the one the flake suggested.

| checked | result |
|---|---|
| the observable asserted | `tests/village_evening.rs`, first time — `grep` found none before |
| planted at the file's 300 | **7 of 7 seeds**, reaching 2–8 villagers |
| planted at the harness's 8 960 | 8 of 10 seeds |
| holders counted | **villagers only** — the player cannot gossip |

The gossip model is sound. What was broken was that nothing checked it and two documents disagreed about
what was being run.

## Pass 2 — Zoom: read the numbers closely

**GREAT**, and the numbers are the whole review. Ten seeds at the harness's tick split **8 pass / 2 fail**;
seven seeds at the file's tick are **7 of 7**. That is not a model that sometimes works — it is one measured
at two different points in a run, one of which is past the village's busiest period.

And the two "failures" turn out to be **1 villager + the player**: at seeds 8 and 99 the harness reports
`known_by: 1` while the count over `PersonIndex` is 2. The player is a `Person` without `Autonomous` — it
never gossips, so a rumor resting there has died. Counting it would make the observable true while the
mechanism failed, which is the exact false pass this file prevents.

## Pass 3 — Fresh eyes: what a reader of `scenarios.yaml` would get wrong

**GREAT**, and this is the pass that justifies the iteration. A host or a future iteration reading
*"a rumor planted at tick 300"* and then running the harness would be measuring a different experiment, in
the same file, under the same `status: measured`. Both ticks are now stated, with the conversion and the
measurement, so the next reader cannot repeat it.

The narrative to a host:

> The `village-evening` scenario claims a planted rumor gets told to someone else, and now a test checks it —
> seven seeds, reaching two to eight villagers. It also turned out the file said the rumor is planted at tick
> 300 while the harness planted at 8 960, thirty times later; at that later point it only spreads in eight of
> ten runs. Both numbers are now written down.

## Verdict

Three GREAT. Shipped. No model change: this iteration is about the crate no longer being able to claim a
scenario passes on the strength of a number nobody verified.
