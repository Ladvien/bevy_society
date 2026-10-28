# One host-walked person, destination never reached (Locomotion::Host, travel_timeout_hours None)

A host that never moves Bram; tavern 8 m away; 64 s/day, 1/64 s ticks (1 game-hour = 171 ticks); 1200 ticks = 7.0 game-hours.

## Before / `stall_timeout_hours: None` (today's behaviour)
- 08.01 h  decided `eat`, travelling
- (nothing for the next 7.0 game-hours: no arrival, no ActivityEnded, the need is never met)

## After / `stall_timeout_hours: Some(1.0)`
- 08.01 h  decided `eat`, travelling
- 09.01 h  `eat` ended, completed=false (no progress for 1 h)  -> re-decides
- 10.01 h  `eat` ended, completed=false
- 11.01 h  ... 12.01, 13.02, 14.02, 15.02: one interrupted ending per game-hour

The host now hears of every stuck walk (`ActivityEnded { completed: false }`) once an hour instead of never. The person re-chooses `eat` because its need is still unmet and no other option scores higher; steering it away from a place that cannot be reached is the host's `TravelFailed`-side knowledge, not this change.
