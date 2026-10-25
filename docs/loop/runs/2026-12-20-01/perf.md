# Perf

## The change

`binary_search` (O(log n), wrong) → `contains` (O(n), correct). The question is whether O(n) is affordable.

## Cost

The scan runs **once per person-targeted activity per society tick**, not once per pair. At 5 000 people that is
at most 5 000 comparisons per such activity — against a perception pass that already tests **12.5 million pairs**
per tick at that population. It is not measurable in the tick cost, and it is the same lookup style every other
consumer of this index already uses (`in_perception` does two `position()` scans per call).

A sorted-by-entity vector would restore O(log n) but would have to be maintained alongside `index.sorted`, and a
second ordering to keep in sync is how this bug happened in the first place. The boring option is correct here.

## The real cost of the bug

Not compute — **behaviour**. 1 045 activities per 17 game-days destroyed on their first tick, and a village whose
friendships were suppressed as a side effect (22 → 31 once fixed).

## Scale curve, still unverified

Fourth iteration without a trustworthy measurement: load average ranged 1.8 to 10.2, and consecutive runs of the
same binary at 1 000 people gave 36.8 / 58.1 / 46.2 ms. The guide's `0.6 / 13.5 / 245.1 ms` table remains
unconfirmed and should not be quoted until someone measures it on an idle machine.
