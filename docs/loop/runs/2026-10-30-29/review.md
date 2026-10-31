# Review: `ActivityDef::urgent` (hunger takes precedence on the provisioning overlay)

Evidence cited is committed: `evidence/urgent.md`, `evidence/plots/urgent.png`, `evidence/plots/zoom-first-days.png`,
`evidence/{before,after}/*/{floor.txt,narrative.txt,summary.json,activities.txt}`, `evidence/sweep/*/{floor.txt,summary.json}`.
Numbers are from those files unless marked.

## 1. Behavior

**Critique.** The claim: a low need should take the choice outright (Maslow's prepotency, the Sims' need buckets, Dill's dual-utility
ranks) and the crate had no rank that followed a need, so a person at `food` 0.05 or less spent half of that time on work, craft and
company with a meal on the shelf (`evidence/sweep/village-s7-default/floor.txt`: 96.8 % of floor ticks with a meal in hand; 49.3 % of
them on something else). Read against the scenario ("needs never met"):

- The share of person-ticks at `food` < 0.05 falls 4-7x at every size and both clocks: village 15.5-19.9 % → 2.6-3.3 % (four seeds), town
  of 60 36.1 / 37.0 % → 7.9 / 8.7 %, town of 40 22.8 → 5.4 %, town of 60 at the library's 240-second day 25.9 → 5.4 %
  (`urgent.md`). The tail of the spell of hunger goes with it: p90 23-47 h → 10-15 h, longest 40-212 h → 24-52 h.
- The mechanism is the claimed one, not a side effect: what remains of the floor in `after/*/floor.txt` is the walk to the table
  (60-71 %) and the night's sleep (18-26 %), with **0.4-0.6 % on work, craft and company** against 49 % before. The mean spell is 6-8 h
  because a spell starts at 0.35 and a person finishes what it is doing, walks and eats; it was 11-20 h.
- The fall-through the Sims describe holds, and it is tested (`tests/urgent.rs::a_meal_with_nowhere_to_eat_it_falls_through`): a hungry
  person with no table plays.
- **The limit is measured, and it is why the starter pack does not take it.** On the default pack the same line makes the floor worse the
  more prepotent it is (19.4 → 18.1 / 26.9 / 52.2 % at 0.15 / 0.25 / 0.35, bread a head 3.2 → 0.7 / 0.3 / 0.0, `eval.md` sweep): its
  `eat` prices an empty shelf and its `bake` is outside the bucket, so the bucket is full of an option that cannot solve the hunger.
  The overlay's `eat` reads the shelf and its `bake` answers a hungry bare household, and there it works. A prepotent option has to
  be able to solve the need; this is in the guide and in the overlay's comments.
- A stranger's reading of one villager's day (`after/v7/narrative.txt`, d10): "06:54 woke and started eat (the sleep finished)", "11:19
  finished eat", "11:31 started work", then company in the afternoon and evening. A person woke hungry, went to eat at once (the walk is
  four game-hours at this clock), then went to work. Over the 18 game-days the tracked villager starts a meal between 05:00 and 09:00 on 15
  of them with `urgent` and on 7 without it (22 and 16 meal starts in all).
- The default pack is untouched: `rank_for` returns `spec.rank` without an `urgent`, and a 17-day run on the pre-change binary and on this
  branch's gives byte-identical `series.csv`, `summary.json`, `narrative.txt`, `activities.txt` and `floor.txt`.

What it does not do: the walk is what is left, and at the harness's 64-second day a walk is two to four game-hours (the
`travel-time-in-two-clocks` coupling), so the floor is not zero; a person already doing a four-hour `work` is not pulled off it (activities
end by duration; the decision runs on the end). Friends over four village seeds fall 58 → 47 (the seeds' own sd is 12), insults rise 277 →
313 (+13 %), accepted approaches rise 2-19 % in the villages and fall 3-4 % in the towns: inside the seed range, named, not explained.

**Verdict: GREAT** for the claim made (precedence on a pack whose options can solve the need), with the pack limit and the social drift
named, not hidden.

## 2. Zoom

**Critique.** `urgent.png` (floor per game-day, what the floor was doing, spells) and `zoom-first-days.png` (first four game-days, tick by tick).

- **Lockstep at hour 24.** Every run still jumps from 0 to a spike of villagers at the floor when the whole population (all at `food` 1.0
  on day one) crosses the line together: 8-12 before, 2-7 after (`zoom-first-days.png`), gone within a game-hour or two where it persisted
  at 3-7 for a day. That is pass 27's heterogeneous-start item and is not touched here.
- **No oscillation that grows.** The after series is bursty (village s42's day-6 bump is 10.7 %, the echo of the day-one lockstep, then 1-4 %;
  the town's 2-15 %) with no growing amplitude over 17-30 game-days; the before series wanders 11-48 %.
- **The day has a shape.** Share of villager-ticks at the floor by clock hour (`urgent.md`, last table): before, 10.6-22.4 % in the villages
  with a shallow dip from 09:00 to 14:00; after, 0.8-1.2 % at 06:00-11:00 (after breakfast) rising to 3-5.6 % from 18:00 to 02:00, in every
  run including the towns (6.2 → 10.3 %) and the 240-second day (2.4 → 7.3 %). The hours `eat` is chosen at (`activities.txt`, village s7) peak
  at 06:00-08:00 (56 / 103 / 74 against 22 / 65 / 36 before: people wake hungry and eat first) and taper through the afternoon, with a few at
  night (in the 60-person town 3-31 starts in each night hour over 30 game-days: a hungry person choosing a night snack over sleep, which is the
  same rank). No herd at one hour.
- **Clipping.** Bread a head: mean 1.2-2.0 and max 2.25-5.75 of a 6-loaf ceiling (before: max 3.5-6.0); nobody hoards to the ceiling and
  the price rests at 0.50 on both sides. The floor is not the only edge: `eat` starts rise 10-39 % (village 425 / 401 / 419 / 409 → 469 /
  485 / 475 / 476; town 1 847 / 1 833 → 2 499 / 2 555) and the share that completes holds (village 87 → 87 %, town 65 → 68 %); the walks
  that time out stay 11-14 % (village) and 30-34 % (town), the harness clock.
- **Spikes at the tick boundary.** None: the harness records every tick and the urgent tier only changes who is chosen at an activity's end.
- **Dead agents.** None observed: the scenario has no deaths, and village s7 completes 6 338 activities and interrupts 247 (5 520 and 277
  before).

**Verdict: GREAT.** The residue (the walk, the night, the first-day lockstep) is named.

## 3. Fresh eyes

**Critique.** Regenerated from scratch with seeds and sizes the design was not tuned on: village s99, a 40-person town at seed 3, and a
town of 60 on the library's own 240-second day (the 64-second harness day is what makes walks long, so this is the check that the
result is not a clock artefact):

- Village s99 (`before` 16.2 % → `after` 2.6 %), town 40 s3 (22.8 → 5.4 %), town 60 s99 (37.0 → 8.7 %), town 60 at the 240-second day
  (25.9 → 5.4 %, p90 spell 32.9 → 13.2 h, longest 99.6 → 30.0 h). Same direction, same size of effect, as on the seeds it was swept on.
- Explained to a host developer without the code: *"a pack can say that an activity jumps the queue while one need is under a level;
  here a hungry person eats, or bakes if the shelf is bare, before anything else, and a hungry person with nowhere to eat carries on with
  the day."* The one thing they must know is the limit: put in the bucket every activity that can solve the need, and nothing that cannot
  (`docs/guide/authoring.md`, the overlay's comments).
- The decision trace says why (`tests/urgent.rs::the_decision_trace_records_the_tier`): `chose eat (0.500, rank 1) over work (0.900, rank 0)`.
  A host asking "why did she leave work for lunch when work scored higher" has the answer in one line.
- Seed-to-seed: villages' friends 37-56 after against 48-75 before, town rivals 70-130 after against 119-167 before in the 60-person
  seeds and the reverse in the 40-person town (54 → 78). The seeds disagree on the social effect, which says it is small against the
  noise; the supply and floor numbers agree on every seed.

**Verdict: GREAT.**

## Where the change is not shipped

The starter pack is not edited (measured to make it worse; the overlay is the pack whose `eat` and `bake` can solve the hunger, and it is
itself opt-in, `needs-decision/households-provision-themselves.md`), and `rest` has no `urgent` (sleep is already rank 1 on a night
window). Recorded as an assumed decision in `needs-decision/hunger-takes-precedence.md`.
