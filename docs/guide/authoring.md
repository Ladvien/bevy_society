# Authoring definitions

Everything the crate simulates is data the host hands in: needs, topics, deeds, factions, activities,
interactions, triggers, conversation rules and node effects. The crate never reads a file — the host
loads the text and passes it in.

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

`SocietyDefs::compile` interns every id in definition order and compiles every Rhai script, so a
malformed definition or a script with a syntax error is reported **before** the app runs, with a path
like `activity 'eat' consideration 2`.

## What each list defines

| List | Defines | Read by |
|---|---|---|
| `needs` | what decays and what activities restore | mind |
| `topics` | what opinions are about | beliefs, deeds |
| `deeds` | an act one person does to another, and its harm | interaction, reputation |
| `factions` | groups, with an in-group affinity bias | groups, ties |
| `activities` | what a person can choose to do, and how it is scored | behaviour |
| `interactions` | a social exchange between two people | behaviour |
| `triggers` | a Rhai rule that fires on an event | rules |
| `conversations` | which conversation a talk opens | dialogue |
| `node_effects` | a Rhai rule that fires when a dialogue node is entered | dialogue |

## Needs, topics, deeds, factions

```ron
(
    needs: [
        (id: "social", decay_per_day: 1.0),
        (id: "rest", decay_per_day: 0.7),
    ],
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
- A **deed** has a `harm` in `[-1, 1]`; positive helps the patient. Its `topic` is the norm a witness
  judges it by, so an opinion above zero approves the deed.
- A **faction**'s `ingroup_bias` (default 0.2) is the affinity two co-members start a new tie with,
  and it is what the acceptance draw weights when two people share a faction.

## Activities

An activity is an option a person can choose, scored by considerations. This one says "sleep at home
between 22:00 and 06:00, more so when tired, and rest while there":

```ron
(
    id: "sleep",
    target: Home,
    duration_hours: 8.0,
    rank: 1,
    considerations: [
        (input: Hour, curve: Hours(22.0, 6.0)),
        (input: Need("rest"), curve: Linear(-1.0, 1.0)),
    ],
    satisfies: [("rest", 0.15)],
)
```

- `target` is one of `Itself`, `Wander(radius)`, `Place(tag)`, `Home`, `Person`, or `Gradient(field,
  toward)` with the `fields` feature.
- `rank` groups options into tiers; the highest rank with any viable option wins. Within a tier,
  options below `Tuning::tier_cutoff` × the best score are dropped and one is drawn by weight.
- `weight` (default 1.0) multiplies the whole score.
- `satisfies` is a need delta **per game-hour arrived**, not per tick.
- `interaction` runs a social exchange on arrival; it requires a `Person` target.
- `barks` are thought lines, one per activity start with probability `SocietySettings::bark_chance`.
- `duration_hours` must be above zero.

A consideration's `input` is normalised to `[0, 1]` before its `curve` sees it: `Hour` is `h / 24`,
mood axes and `Affinity`/`Respect` are `(x + 1) / 2`, `Distance` is `d / perception_radius`. `Trust`,
`Familiarity` and `SocialProof` are already in `[0, 1]`.

Curves turn that number into utility:

| Curve | Value |
|---|---|
| `Linear(m, b)` | `clamp(m·x + b, 0, 1)` |
| `Power(k)` | `x^k` |
| `Logistic(k, x0)` | `1 / (1 + e^(−k(x − x0)))` |
| `Step(at)` | `1` when `x ≥ at` |
| `Hours(from, to)` | `1` inside the wrapping hour window; only with `input: Hour` |

`Affinity`, `Trust`, `Respect`, `Familiarity` and `SocialProof` are only valid on a `Person` target,
and `Distance` is valid on `Place`, `Home` and `Person`. A definition that breaks that rule is
rejected with the offending consideration named.

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
| `Hour(h)` | the clock crosses hour `h` |
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
actor. See [`scripting.md`](scripting.md).

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
