# Assumed decision: what `Away` does, and how a return ends it

Assumed decision (owner may override), made on issue #35.

1. **Closeness is read live from the tie, not frozen at departure.** Options: freeze affinity toward an away person (a change to relaxation, dyad balance and triads), store closeness at departure in `AwaySince`, or read the live tie. Picked the live tie: smallest, and the tie drifting during a trip is the existing model. Measured: affinity is clamped by the friendship ceiling at the first tick, so the level in the harness reflects the clamped tie.
2. **Removing `Away` leaves `Dormant`.** Options: also remove `Dormant` (wrong if the person is dormant for another reason, e.g. in a conversation), or leave it as Bevy leaves any required component. Picked the second; the guide says to remove both.
3. **The feeling is `Distress` toward the away person, raised to a level each tick; the reunion is `Joy` of a fraction of that level.** Mood only, never a cause of an activity.
4. **`missing_cap = 0.0` is the single off switch**, reunion included.
5. **`SocietySave` goes to version 3** with `away_since_bits` (absolute day, exact bits), because the clock is restored with the people.
