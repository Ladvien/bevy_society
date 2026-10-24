# Perf

No simulation change shipped — the rate experiment was reverted. The warning is one pass over
`defs.needs × defs.activities` in `finish`, which runs once when the plugin is built.

The two new `SocietyDefs` accessors are `O(activities)` and `O(1)` respectively; the tests use them and
a host reading a budget would.

Standing benchmark, unchanged and not re-run because nothing in the tick path changed:

| Persons | ms per society tick |
|---|---|
| 100 | 0.564 |
| 1 000 | 15.02 |
| 5 000 | 275.5 |
