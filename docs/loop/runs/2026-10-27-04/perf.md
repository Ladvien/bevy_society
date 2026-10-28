# perf
One `BTreeMap` pass over persons per tick, replacing an O(persons) distance scan *per un-arrived person with a place target*: complexity falls from O(walkers x persons) to O(persons log places). Not timed: the host is loaded (measurement spreads 37-57%); no speedup claimed.
