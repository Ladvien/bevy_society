# Perf

## The change

None shipped.

## What it would have cost

`talk_in_contact` walks `Perception.contact`, which is already built and already walked by `expose_contacts`,
`expose`-based contagion and conformity. It adds one `Ties` lookup and one message write per pair per cooldown
— 13 742 requests over 17 game-days at a 3-hour cooldown, about 0.2 per person per tick.

The cost is not the problem. The rate is.
