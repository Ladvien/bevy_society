# Authoring definitions

Everything the crate simulates is data the host hands in: needs, signals, topics, deeds, factions,
activities, interactions, triggers, conversation rules and node effects. The crate never reads a file
— the host loads the text and passes it in.

```rust,ignore
use bevy_society::{DefsFile, SocietyDefs};

let defs = SocietyDefs::from_ron(include_str!("my.society.ron"))?;
```

## Composing files

`DefsFile::starter()` parses the bundled starter pack, and `DefsFile::merge` adds a second file's
definitions. Merging rejects a **duplicate id** — in any list — and leaves the receiver unchanged, so
a host's file holds only what it adds:

```rust,ignore
let mut file = DefsFile::starter()?;
file.merge(DefsFile::from_ron(include_str!("village.society.ron"))?)?;
let defs = SocietyDefs::compile(file)?;
```

Every list is carried: needs, signals, **skills, goods**, topics, deeds, factions, activities, interactions,
triggers, conversations and node effects. `goods` and `skills` were omitted until 2026-10-26-13, so a pack
that added a good contributed nothing and the error surfaced later as an unknown id on whichever activity
consumed it — a host upgrading from 0.3.0 with a split pack should re-check that its merged file compiles.

`SocietyDefs::compile` interns every id in definition order and compiles every Rhai script, so a
malformed definition or a script with a syntax error is reported **before** the app runs, with a path
like `activity 'eat' consideration 2`.

## What each list defines

| List | Defines | Read by |
|---|---|---|
| `needs` | what decays and what activities restore | mind |
| `signals` | a value the host writes and the crate only reads | behaviour, scripts |
| `topics` | what opinions are about | beliefs, deeds |
| `deeds` | an act one person does to another, and its harm | interaction, reputation |
| `factions` | groups, with an in-group affinity bias | groups, ties |
| `activities` | what a person can choose to do, and how it is scored | behaviour |
| `interactions` | a social exchange between two people | behaviour |
| `triggers` | a Rhai rule that fires on an event | rules |
| `conversations` | which conversation a talk opens | dialogue |
| `node_effects` | a Rhai rule that fires when a dialogue node is entered | dialogue |

## Needs, signals, topics, deeds, factions

```ron
(
    needs: [
        (id: "social", decay_per_day: 1.0),
        (id: "rest", decay_per_day: 0.7),
    ],
    signals: [(id: "day_job"), (id: "on_watch"), (id: "grieving")],
    topics: [(id: "kindness"), (id: "honesty")],
    deeds: [
        (id: "help", topic: "kindness", harm: 0.5),
        (id: "steal", topic: "honesty", harm: -0.7),
    ],
    factions: [(id: "millers"), (id: "watch", ingroup_bias: 0.3)],
)
```

- A **need** level is 1 when satisfied and 0 when desperate. `initial` (default 1.0) is what a new
  person starts at.
- A **signal** is a number the **host** owns: the game writes it on the person's `Signals`
  component and the crate never changes it. A value the host never wrote reads `0.0`, and a person
  spawned before the signal existed reads `0.0` too. Reads through `Signals::get(id)` (an index
  resolved once with `SocietyDefs::signal_id`) or `Signals::set(defs, "name", value)`, and in
  scripts through `ctx.signal(a, id)`. Because it neither decays nor is paid by an activity, it is
  the right carrier for a fact that is true or false today — a day job, a night watch, grief.
- A **deed** has a `harm` in `[-1, 1]`; positive helps the patient. Its `topic` is the norm a witness
  judges it by, so an opinion above zero approves the deed.
- A **faction**'s `ingroup_bias` (default 0.2) is the affinity two co-members start a new tie with,
  and it is what the acceptance draw weights when two people share a faction.

## Activities

An activity is an option a person can choose, scored by considerations. This one says "sleep at home
between 22:00 and 06:00 by your own clock, more so when tired, and rest while there; nobody walks up to
you meanwhile":

```ron
(
    id: "sleep",
    target: Home,
    duration_hours: 8.0,
    rank: 1,
    unavailable: true,
    considerations: [
        (input: BodyHour, curve: Hours(22.0, 6.0)),
        (input: Need("rest"), curve: Linear(-1.0, 1.0)),
    ],
    satisfies: [("rest", 0.15)],
)
```

- `target` is one of `Itself`, `Wander(radius)`, `Place(tag)`, `Home`, `Bound(name)`, `Person`, or
  `Gradient(field, toward)` with the `fields` feature.
  - `Place(tag)` is the nearest place carrying the tag, the same for everyone.
  - `Home` is the one place a person's `Home` component names.
  - `Bound(name)` is the place *that person* has bound under `name` in their `BoundPlaces` component, so
    each person can be sent to their own site. A person with no such binding simply does not do the
    activity — that is not an error, and it is not warned about. A binding to something that is not a
    `Place` is a wiring mistake and warns once per person and name.
- `rank` groups options into tiers; the highest rank with any viable option wins. Within a tier,
  options below `Tuning::tier_cutoff` × the best score are dropped and one is drawn by weight.
- `urgent` raises the rank **while one need is low**: `urgent: Some((need: "food", below: 0.25, rank: 1))` puts the
  activity in rank 1 whenever the actor's `food` is under `0.25`, and leaves it at its own `rank` otherwise. It is
  Maslow's prepotency in utility-AI form (*The Sims* buckets motives by urgency, so "a starving Sim will never even
  consider watching TV unless they fail to find anything that can solve their hunger"): a hungry person's meal is in
  a tier above the work, the company and the play it would otherwise be drawn against, and when the meal has no
  candidate (no place tagged for it) the lower tiers decide as before. `below` is in `(0, 1]`, the urgent `rank`
  must exceed the activity's own, and an unknown need is an error naming the activity. **Put in the bucket every
  activity that solves the need, not only the one that satisfies it**: a household with a bare shelf solves its
  hunger by baking, so `bake` carries the line too (`examples/assets/provisioning.society.ron`), and a prepotent
  `eat` on a pack that prices an empty shelf instead of reading it empties the shelves and starves the village
  (`docs/loop/runs/2026-10-30-29/research.md`).
- `weight` (default 1.0) multiplies the whole score.
- `satisfies` is a need delta **per game-hour arrived**, not per tick.
- `interaction` runs a social exchange on arrival; it requires a `Person` target.
- `unavailable` (default `false`) makes a person **who has arrived at the activity** impossible to approach: they are in
  nobody's `near` list, so no one picks them as the target of a `Person` activity (and they are not a `Distance`
  neighbour), and an activity still walking toward them ends, `completed: false`, when they arrive. Someone still
  walking to bed is on their way, not asleep. It is the per-activity form of `Dormant`'s "out of perception, no
  exchanges", and it is what `sleep` needs once people keep their own hours (below): without it an owl awake at 01:00
  would compliment the neighbours who are asleep.
- `barks` are thought lines, one per activity start with probability `SocietySettings::bark_chance`.
- `duration_hours` must be above zero.
- A place that is **full** is not unavailable — it costs `Tuning::crowding_penalty` (0.6) of the option's score, so a
  person with another place to go takes it and a person with nowhere else still arrives. `capacity: 0` means
  **closed**: nobody ever goes. A full place used to produce no target at all, which was measured to remove a
  village's working day once it outgrew its seats (`docs/loop/runs/2026-10-25-10/eval.md`).

A consideration's `input` is normalised to `[0, 1]` before its `curve` sees it: `Hour` and `BodyHour` are `h / 24`, `Week` is
`(weekday + h / 24) / 7`, mood axes and `Affinity`/`Respect` are `(x + 1) / 2`, `Distance` is `d / perception_radius`. `Trust`,
`Familiarity` and `SocialProof` are already in `[0, 1]`, and `Need`/`Signal` are read as they stand,
so a signal the host writes outside `[0, 1]` is clamped after the curve. `Signal("x")` names a
signal declared in `signals`; an undeclared name is a `DefsError::UnknownId` at load, not a score of
zero at run time.

A `Signal` consideration scores as written, with no target-dependent arm — it reads the actor's own
component — so it is valid on every target kind. `Step(0.5)` on a signal of `1.0` scores 1 and on
`0.0` scores 0.

Curves turn that number into utility:

| Curve | Value |
|---|---|
| `Linear(m, b)` | `clamp(m·x + b, 0, 1)` |
| `Power(k)` | `x^k` |
| `Logistic(k, x0)` | `1 / (1 + e^(−k(x − x0)))` |
| `Step(at)` | `1` when `x ≥ at` |
| `Hours(from, to)` | `1` inside the wrapping hour window; only with `input: Hour` or `input: BodyHour` |
| `Days(from, to)` | `1` inside the wrapping week window, in days from the start of the week (`Days(5.0, 7.0)` is the last two); only with `input: Week` |

`Affinity`, `Trust`, `Respect`, `Familiarity` and `SocialProof` are only valid on a `Person` target,
and `Distance` is valid on `Place`, `Home` and `Person`. A definition that breaks that rule is
rejected with the offending consideration named.

`Field(input)` reads a social field, and needs the `fields` feature. `Commotion` is the one to reach for
when a town should drift toward trouble: it is written by a witnessed deed's `|harm|` and by any host
that writes a `Commotion { at, amount }` message, and a `Gradient(Commotion, true)` target walks a
person up the slope toward it.

### Two clocks: `Hour` and `BodyHour`

`Hour` is the village's clock, the same for everyone: shop hours, the baker's dawn, the working day. `BodyHour` is the
actor's own: the clock hour less their `Chronotype` (hours; positive is an owl, negative a lark; the default `0.0` makes
it equal to `Hour`). Both are `h / 24` and both take the `Hours(from, to)` curve. Write what the body decides (when to
sleep) on `BodyHour` and what the village decides (when the workshop opens) on `Hour`; the gap between the two is what
chronobiology calls social jetlag (Wittmann et al. 2006, DOI `10.1080/07420520500545979`), and an owl who sleeps
until 09:00 beside a workshop that opens at 08:00 is a story the pack's author can use or leave out.

The crate never draws a chronotype; the host writes one the way it writes a `Personality`:
`Chronotype::sample(&mut rng, Chronotype::POPULATION_SD_HOURS)` for a population like the measured ones (a normal of
1.5 h, cut at three deviations), or `chronotype_hours: -2.0` on a `PersonDef` in a placement file. A save keeps it
(`PersonSave::chronotype`; a file from before it restores as `0.0`). Without one a person keeps the village's clock,
so `BodyHour` reads what `Hour` does. `docs/research/chronotype.md`.

### A week: `Week` and `Days`

`Input::Week` is how far through the week it is, `(weekday + hour / 24) / 7`, and `Days(from, to)` is a window over it, in days
from the start of the week, the way `Hours` is over the day: `Days(0.0, 5.0)` is the first five days, `Days(5.0, 7.0)` the last
two, `Days(6.0, 1.0)` wraps (the last day and the first), and `Days(4.75, 6.0)` opens at 18:00 on day 4 because the input
carries the hour. Both ends must be in `0..=7`, and a `Days` curve on any input but `Week` is refused at load.

```ron
(id: "work", target: Place("work"), duration_hours: 4.0,
 considerations: [
     (input: Hour, curve: Hours(8.0, 17.0)),
     (input: Week, curve: Days(0.0, 5.0)),   // shut on days 5 and 6
 ],
 satisfies: [], interaction: None, barks: [])
```

Day 0 is whatever the host calls the first day of its week, and `SocietySettings::start_weekday` (default `0`, values of 7 and
over wrap) is the weekday of game-day 0; `ctx.weekday()` in a script reads the same calendar. A week is seven days
(`model::DAYS_PER_WEEK`); a host with another calendar writes it as a script. Like `Hour`, `Week` is the village's clock: a
`Chronotype` shifts `BodyHour` and never `Week`, because a rest day is a social fact. The starter pack has no week, so a society
that never writes `Week` is the society the crate always had, bit for bit. `examples/assets/weekly.society.ron` is an opt-in
overlay with a rest day: `work` shut on days 5 and 6 and a `laze` that takes its place. Shutting `work` alone is not enough: the
hours it frees go to whatever else is open, which in the starter pack is company, and the village's rivalries grow
(`docs/research/weekly-rhythm.md`). The settings are not saved: a host restoring a save passes the same `start_weekday` it
started with.

## Interactions

```ron
(
    id: "chat",
    accept: (base: 0.5, affinity: 0.3, trust: 0.2, agreeableness: 0.2, pleasure: 0.1, ingroup: 0.1),
    on_accept: (actor_needs: [("social", 0.2)], target_needs: [("social", 0.2)]),
    influence: ["kindness", "honesty"],
    on_reject: (actor_emotion: Some((Distress, 0.3))),
    barks: (
        actor: ["Morning, {target}!"],
        accepted: ["Good to see you, {target}."],
        rejected: ["Not now."],
    ),
)
```

The acceptance draw is
`clamp(base + affinity·aff(t→a) + trust·(trust(t→a) − 0.5) + agreeableness·(A_t − 0.5) + pleasure·P_t + ingroup·[shared faction], 0.02, 0.98)`
with one draw from `SocietyRng`. A `requires` expression is evaluated first; when it is false nothing
else happens and the exchange is reported as refused by rule.

An accepted exchange can also do each of these, in this order: `influence` the listed topics both
ways, `gossip`, run a `contest`, apply a `deed` (which witnesses judge and which can become a rumor),
introduce the pair to each other's close ties, and add intergroup contact.

Bark templates may use `{self}`, `{target}`, `{place}` and `{subject}` (rumor subject, gossip only).
Any other `{...}` is rejected at compile time rather than printed literally.

## Goods, stores and trade

`goods: [(id: "bread")]` declares a **quantity**, not a currency. Every person carries a `Stock` with one level
per good, in `[0, Tuning::stock_ceiling]`. A person starts holding the good's `initial` (`goods: [(id: "bread",
initial: 1.0)]`; `0.0` when omitted, held at the ceiling), the way a need starts at its `initial`: a society
whose larders all begin empty has nothing to eat for its first game-day and is starving on its second, because
nothing has yet been made to eat
(the starter pack starts every person with one loaf, `docs/loop/runs/2026-10-30-27/`). A `Stock` the host
spawned with the right length is left as it is. An activity can `produces: [("bread", 3.0)]` and `consumes:
[("bread", 1.0)]`, applied once, when it **completes** (an interrupted one pays nothing, like `satisfies`). Each
good has a price in `Prices`, read off **how full the village's stores are, per head**:
`price = 2 · (1 − r)` below half full and `1.5 − r` above it, where `r` is
`total held / ( villagers · Tuning::price_full_store )`. An empty store is worth **double** the nominal unit, a
full one **half**, and half full is nominal — the shape and the numbers from the corpus's own game-economy
Model A (`eth-cgl-ar_games-Zun16a` §3.1.1: "the price modifier term v(r) doubles the price if the storage is
empty and halves the price if the storage is full"). It is clamped to
`[Tuning::price_floor, Tuning::price_ceiling]`. The same six loaves therefore price differently in a village
of three and one of thirty, which a count of units could not say. **Nothing gates**: an empty store never
stops anyone eating — but **it stops the meal feeding them**. An activity's `satisfies` is paid in
proportion to what its `consumes` actually took, `min(1, held / asked)`, so `eat` from an empty larder still
walks to the table, still spends the ticks, and restores no `food`. A partial store feeds partially. An
activity with no `consumes` is unaffected; its payout is exactly what it always was. This follows
Zandstra 2000 (DOI `10.18174/197257`), whose *satiation* — the term that "influences the size of meals" —
is what `satisfies` is modelling.

A good changes hands on an accepted interaction:

```ron
(
    id: "share",
    on_accept: (
        actor_gives: [("bread", 1.0)],     // `target_gives: [...]` too, and both is a swap
        actor_to_target: (trust: 0.05),    // the giver is paid in goodwill
        target_to_actor: (trust: 0.05),
    ),
)
```

Each side gives what it has, up to the amount, so the pair cannot create a good. An activity that offers such
an exchange is scored by the **joint surplus** of it:

```ron
(
    id: "share",
    target: Person,
    duration_hours: 0.2,
    considerations: [(input: Surplus("bread"), curve: Linear(2.0, 0.0))],
    interaction: Some("share"),
)
```

`Surplus(good)` is `value(target's store) − value(own store)` with `value(held) = 1 / (1 + held)`, in `[0, 1]`:
zero between equal stores, 0.5 for a loaf to someone with none, and zero unless the giver holds **more than**
`Tuning::trade_reserve` (so a giver of a whole loaf is left with at least one) and the gap is at least `Tuning::trade_margin`. It needs a `Person` target. The
neighbours' stores are copied for scoring only when some activity uses `Surplus`, so a pack without one pays
nothing.

### A household that provisions itself: `Stock(good)`

`Stock(good)` is how much of a good the **actor itself** holds, in units, clamped to `[0, 1]` like every input, so
a curve describes the first loaf and "a day's supply", not a bin of six. It needs no target and costs one read of
the actor's own `Stock`. It is what lets a pack close the loop from "my shelf is bare" to "make some":

```ron
// make bread when hungry and holding none ...
(id: "bake", target: Place("food"), duration_hours: 1.0, produces: [("bread", 4.0)],
 considerations: [
     (input: Hour, curve: Hours(6.0, 20.0)),
     (input: Stock("bread"), curve: Linear(-1.0, 1.0)),        // 1.0 empty, 0.0 from one loaf up
     (input: Need("food"), curve: Logistic(-10.0, 0.5)),       // only a hungry household bakes
 ],
 satisfies: [], interaction: None, barks: [])
// ... and eat where there is something to eat
(id: "eat", target: Place("food"), duration_hours: 0.75, consumes: [("bread", 0.25)],
 considerations: [
     (input: Need("food"), curve: Logistic(-10.0, 0.35)),
     (input: Stock("bread"), curve: Linear(4.0, 0.05)),        // a quarter loaf, one meal, reads as 1.0
 ],
 satisfies: [("food", 1.2)], interaction: None, barks: [])
```

**Why a pack wants this.** A producer that works by the clock is an open loop: stable only if its output happens to
equal what the village eats. The starter pack's `bake` is a one-hour dawn errand drawn against nine other options, and
on the evidence village scaled to sixty people it makes almost no bread (3 bakes in ten game-days at the harness's
64-second day, 0-5 a day at the 240-second one), the stores run down, and the town is hungry for good by about
game-day 12 (`docs/loop/runs/2026-10-30-28/`). A household that tops
up **its own** shelf is a balancing loop on a stock (Meadows, *Thinking in Systems*), so production is the sum of what
the shelves ask for and scales with the population by construction. A person with an empty shelf that is not hungry
is not made to bake at once, and a person at a table with nothing on it is priced, not forbidden: the floor of
`0.05` keeps "nothing gates" true. `examples/assets/provisioning.society.ron` is this recipe as the two replacement
activities; the starter pack keeps its clock-driven `bake` until the owner decides otherwise
(`docs/loop/needs-decision/households-provision-themselves.md`).

The overlay's `eat` and `bake` also carry `urgent: Some((need: "food", below: 0.25, rank: 1))` (see `urgent` under
Activities): a hungry household eats, or bakes when the shelf is bare, before the work, company and play it would
otherwise be drawn against. The floor on `food` falls from 15-20 % of person-ticks to under 3 % in the village and from
36-37 % to 8-9 % in the town of sixty (`docs/loop/runs/2026-10-30-29/`). It needs both lines: with `urgent` on the
starter's `eat` alone, which prices an empty shelf and leaves `bake` out of the bucket, a hungry household keeps choosing
a meal that feeds nothing and the floor rises.

## Triggers

```ron
(
    id: "village_turns",
    on: RumorReach(deed: "steal", fraction: 0.5),
    then: "fx.emit(\"village_turns\", #{})",
    once: true,
)
```

`on` takes one of these forms:

| `on` | fires when |
|---|---|
| `Tick` | every society tick |
| `Hour(h)` | the clock crosses hour `h`; `h` may be fractional (`5.5` is 05:30) |
| `PersonDied` | a person dies; the deceased is bound as the actor |
| `Interaction("name")` | that interaction resolves |
| `TieLabel(Friend)` | a tie's label changes to it |
| `RumorReach(deed: "steal", fraction: 0.5)` | a rumor of that deed reaches that fraction of the living |
| `NormViolated("steal")` | a witness judges that deed a norm violation |
| `NeedBelow(need: "social", below: 0.3)` | a need first falls below that level |
| `Activity("name")` | that activity starts |
| `DialogueNode(conversation: "greet", node: "n")` | that node is entered |
| `Custom("name")` | the host fires it with `FireTrigger` |

Two syntax traps, both measured: **struct variants take parentheses**, so `RumorReach { deed, fraction }` fails
with `Expected opening (`; and **`TieLabel` variants are capitalised**, so `friend` fails with a list of the four
valid names. Every row above is asserted by `tests/guide_blocks.rs` against the loader, so the table cannot drift
from the crate without a test failing.

`if` is an optional Rhai condition; `once` fires at most once per (trigger, actor) and `cooldown_hours` gates by
actor. `area: Some((min: (x, y), max: (x, y)))` limits a trigger to an actor standing inside that rectangle on the society plane (edges inclusive); `Tick`, `Hour` and `RumorReach` bind no actor, so the loader refuses an area on them. `factions: ["millers"]` limits a trigger to actors who belong to any listed faction, and `persons: [3]` to actors with those `PersonId`s (a trigger listing both takes either); an unknown faction is refused, and so are both lists on the actorless events. Empty (the default) means any actor. See [`scripting.md`](scripting.md).

## Conversations and node effects

```ron
(
    conversations: [
        (conversation: "greet_friend", priority: 2, if: Some("ctx.label(actor, target) == \"friend\"")),
        (conversation: "greet", priority: 0),
    ],
    node_effects: [
        (conversation: "greet", node: "compliment", then: "fx.adjust(actor, target, \"affinity\", 0.1)"),
    ],
)
```

These name conversations and nodes that live in the host's `bevy_speech_bubbles::DialogueScript`. They
are inert data without the `dialogue` feature, so one definitions file serves every feature set; with
the feature on, a name the script does not define is an `error!`.

## Placing persons

A definitions file says what kinds of things exist (`DefsFile::from_ron(text)` and `.to_ron()` round-trip, so an editor can save what it edited); a `PlacementFile` says who and where. It is RON too, and an editor can save it and a game load it: `PlacementFile::from_ron(text)` and `.to_ron()` round-trip.

```text
(
    places: [(id: "mill", at: (10.0, 0.0), capacity: 3, tags: ["work"]), (id: "cottage", at: (-4.0, 2.0), capacity: 2)],
    persons: [(name: "Ada", home: Some("cottage"), bound: [("work", "mill")], factions: ["millers"], needs: [("hunger", 0.4)], personality: (extraversion: 0.9), ties: [(with: "Bram", affinity: 0.2)])],
)
```

`placement.spawn(world)` checks every name against the world's `SocietyDefs`, spawns nothing when a line is wrong, and returns a `Placed` that maps names to entities. A person starts at `at`, else at their home. A trait left out is `0.5`, a tie's `trust` left out is `0.5`, and `autonomous` is `true` unless given. Spawning draws no random numbers. `PlacementFile::capture(world)` reads a live world back into a file (positions as they are now, opinions as their anchors), so what a user moved can be saved with `to_ron()`; every place and person must have a `Name`.

## Errors

`DefsError` is `Display` and implements `std::error::Error`:

| Variant | Means |
|---|---|
| `Ron(String)` | the text is not RON |
| `Duplicate { kind, id }` | an id appears twice in one list, or a merge collides |
| `UnknownId { kind, id, owner }` | a name refers to nothing defined |
| `Invalid { owner, message }` | a value out of range or a field used on the wrong target |
| `Script { owner, message }` | Rhai syntax error |

`owner` is a path such as `activity 'eat' consideration 2`, so the message points at the definition.
