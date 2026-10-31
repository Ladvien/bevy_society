# A tour

The pieces of a society, one at a time. [`getting-started.md`](getting-started.md) builds the first village; this page is the vocabulary for the rest.

### People

```rust,ignore
app.world_mut().spawn((
    Person,            // brings personality, mood, emotions, needs, signals, ties, opinions, knowledge,
    Autonomous,        // memories, memberships and status
    Name::new("Bram"),
    Home(well),
    Transform::from_xyz(2.0, 0.0, 0.0),
));
```

A player character is a `Person` without `Autonomous`: it does only what the game tells it.

### Needs and daily life

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

Options are scored by compensated considerations, tiered by `rank`, and drawn by weight. A rank can follow a need: `urgent: Some((need: "food", below: 0.25, rank: 1))` puts an activity in the higher tier while that need is low. The provisioning overlay (`examples/assets/provisioning.society.ron`) uses it on `eat` and `bake`, so a hungry villager eats, or bakes when the shelf is bare, before work, company and play; the starter pack leaves it off (`docs/guide/authoring.md`).

### Relationships

`Ties` holds one directed edge per person with affinity, trust, respect and familiarity. Labels are recomputed each tick and a change writes `TieLabelChanged`. **A label is a latitude, not a line**: entering a category and leaving it are different tests (`Tuning::label_hysteresis` at the low end, `Tuning::friend_band` for a friendship, `Tuning::rival_band` for a rivalry), so a value resting on a bar does not announce a relationship change every time the model nudges it. Measured on `village-evening` at seed 7, a friendship now lasts a median **1.01 game-days** instead of **0.36** (`CHANGELOG.md`, 0.3.0), and a rivalry **2.64** instead of **0.29** — before the rival band a `Rival` was released when its affinity climbed **0.0001** above the bar, so the village made rivalries it could not keep ([the sweep](https://github.com/bevy-coven/bevy_society/blob/main/docs/loop/runs/2026-10-27-02/eval.md)).

### Opinions

An accepted chat pulls both sides' opinions toward each other inside their confidence bounds, and a confident group majority pulls an agreeable person further, one tick at a time.

### Rumors and reputation

```rust,ignore
app.world_mut().write_message(PlantRumor {
    origin: watchman,
    subject: bram,
    patient: None,
    deed: "steal".to_owned(),
    truth: false,
});
```

Every witness who knows it can pass it on; a listener who already knows it stifles the teller.

### Factions and status

```ron
(factions: [(id: "millers"), (id: "weavers"), (id: "watch", ingroup_bias: 0.3)])
```

Two co-members start a tie with in-group affinity, and standing is the respect conferred by everyone who knows a person well enough.

### Triggers and scripting

```ron
(
    id: "watch_stops_theft",
    on: NormViolated("steal"),
    if: Some("ctx.member(actor, \"watch\")"),
    then: "fx.bark(actor, \"Stop, thief!\")",
)
```

### Dialogue

```rust,ignore
app.add_plugins(bevy_speech_bubbles::SpeechBubblesPlugin::<MyCamera>::default());
app.world_mut().write_message(TalkTo { npc: bram, player });
```

