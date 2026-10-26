# A village with no room

> **Measured 2026-10-25-09. The diagnosis is the deliverable; nothing shipped.** A model change was designed,
> tested and reverted because this control showed the problem was elsewhere.

## The state

`examples/evidence.rs --scenario village-evening --people N` builds the **same four places** for every `N`, and
those places declare capacity 6 + 10 + 8 + 6 = **30 seats**. At `--people 24` that is room to spare. At 36 and 48
it is not, and the crate says so at startup:

> `bevy_society: 'well' is home to 16 people, but an arrival disc of radius 1.5 seats about 4. ... their sleeps are
> interrupted. Give them more room: raise interaction_radius to about 2.8 or spread the residents over more homes.`

What an overfilled village looks like, five seeds, 17 game-days:

| people | % of pairs hostile | work share of a life | friend pairs |
|---|---|---|---|
| 24 (30 seats) | 7.8% | 10.3% | 18.4 |
| 36 (30 seats) | 23.1% | 5.5% | 20.2 |
| 48 (30 seats) | 20.1% | 3.6% | 32.0 |
| 36, places scaled to fit | **0.00%** | 10.3% | 67.2 |
| 48, places scaled to fit | **0.02%** | 7.9% | 81.6 |

**`work` collapses because the workshop is full**, and social contact roughly replaces it. The rivalries follow from
the same number of hostile events landing on a far denser contact graph. Scale the seats and the hostility goes to
zero.

Note that **zero is also wrong**: a village with no rivalries at all is a different failure from one where a fifth of
pairs are hostile, and the 24-person village's 21.6 rival pairs are the behaviour the README's clip and the
scenarios assert. Both extremes are broken; the honest target is a density that holds roughly flat as a village
grows.

## Why this is a decision and not a fix

Three ways to make a host's growing village behave, and they are not equivalent:

### 1. `--people` scales the places too (harness only)

- **For:** one line in `examples/evidence.rs`, and every future population claim becomes true instead of measuring
  a queue. The evidence harness exists to answer "what would a bigger village look like", and today it cannot.
- **Against:** fixes the instrument, not the library. A host that adds people and not places still gets a village
  that cannot work — which may be exactly what it wants (a refugee camp, a besieged town).
- **Risk:** the scaled rows above are **0% hostile**, so scaling seats alone overshoots. Scaling capacity *and*
  keeping the density flat needs a second lever, which is option 2.

### 2. Make `capacity` a soft limit with a cost, not a hard gate

- **For:** a full workshop should push people elsewhere, not make `work` unselectable. Today `Place(tag)` filters
  `used < capacity` before scoring, so a full place simply **has no candidate** and the activity cannot be chosen
  at all. A penalty in the score would let a crowded workshop still be worked at, less often.
- **Against:** it changes the activity mix of every village at every population, so it needs the full nine-seed
  measurement; and `Home` already bypasses the gate entirely (only `Place(tag)` and `Bound` check it), so "full"
  currently means two different things in one function.

### 3. Leave it: capacity is a host's design tool

- **For:** the crate warns at startup, names the place and the radius that would fix it, and a host that models a
  crowded village may want exactly this outcome.
- **Against:** the failure mode is silent in the numbers a host looks at (friend/rival counts, not "my villagers
  stopped working"), and the evidence harness itself fell into it and produced a false scaling law that reached an
  iteration's log.

## Recommendation

**Option 1 now, option 2 as its own iteration, option 3 never as the whole answer.**

Option 1 is a correction to an instrument that already misled one iteration. Option 2 is the library change worth
making — a full place should be a price, not a prohibition — but it re-shuffles every village's day and must be
measured on nine seeds across three populations before it ships, with the target stated up front: **rival density
roughly flat at about 7-8% of pairs from 24 to 48 people**, and `work` holding at about 10% of a life.

## Not doing yet, and why

The `witness_dilution` inconsistency (`src/reputation.rs` dilutes trust and not affinity, respect or memory
importance — see `docs/loop/runs/2026-10-25-09/eval.md`) is real, and it was the first thing tried. Every positive
exponent emptied the 24-person village's rivalries, because that village's crowd is already 12.6 witnesses per deed.
It cannot be fixed before the density question above is settled, or the fix will be judged against a baseline that
is itself an artefact.
