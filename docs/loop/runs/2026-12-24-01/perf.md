# Perf

## The change

One `perception.pos` scan per arriving person, counting bodies inside `interaction_radius` of their goal, plus one
`sqrt`. It runs only while `arrived_day` is `None` — i.e. while someone is still walking — so it is bounded by the
number of travellers, not by the population.

## Cost

`O(contact-density)` per travelling person per tick. The perception pass in the same tick already walks the grid
and builds `near`/`contact`; this reads the same `pos` array. Not measurable in the scenario wall time.

## Scale curve, still unverified

Eighth iteration without a trustworthy absolute number. The guide's `0.6 / 13.5 / 245.1 ms` table cannot be
checked on this machine: load has sat between 6 and 12 throughout, and consecutive runs of the same binary at
1 000 people have spread by 7%.

What can be said: the scenario runs used here (`village-evening`, 67 200 ticks, 24 people) take the same wall time
before and after, and the change is an `O(near)` scan on top of one that already exists.
