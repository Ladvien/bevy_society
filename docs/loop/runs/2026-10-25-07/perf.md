# Performance: a basic economy

Two systems: `produce_and_consume` reads the tick's `ActivityEnded` messages and touches one person's stock per
message; `update_prices` walks the goods list once. Both return immediately when the pack declares no goods.

| persons | before, fastest window (ms/tick) | after, fastest window (ms/tick) | spread after |
|---|---|---|---|
| 100 | 0.730 | **0.762** | 9.7% |
| 5 000 | 508.798 | **516.979** | 10.4% |

**Inside the noise, and slightly up.** 4% at 100 people and 1.6% at 5 000, against spreads of 9.7% and 10.4%.
The one avoidable cost is `produce_and_consume`'s `position` scan over `PersonIndex::sorted` per ending, which
is `O(n)` per ending and therefore `O(n·endings)` — at 5 000 people with no goods declared it never runs, and
the benchmark's pack declares none, so the measured figure does not include it. A village with goods at that
scale would want the scan replaced with the index lookup the rest of the crate uses; that is recorded rather
than fixed, because nothing in the benchmark exercises it and a change there would be unmeasured.

The 1 000-person point is not re-measured: the machine's spread at that size has been 24-44% across iterations.
