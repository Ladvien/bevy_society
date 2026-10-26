//! **A script can read what a villager holds and what it is worth.**
//!
//! `Prices`' own documentation has promised `ctx.price("bread")` since the economy landed, and there was no
//! such function: a price no rule can read is decoration, and the whole point of pricing from the stores is
//! that a host's trigger can react to a shortage. `ctx.stock` is the person half of the same interface.
//!
//! Both read the **tick-start snapshot**, like every other `ctx` call: `update_prices` runs in
//! `SocietySystems::Interact`, after `Perceive` froze the snapshot, so a rule reads the price as of the tick
//! it woke on and acts on the next.
//!
//! `tests/economy.rs` owns the price *model*; this file owns the bridge from the world to a script.
mod common;

use bevy::prelude::*;
use bevy_society::{
    FireTrigger, ScriptError, ScriptEvent, ScriptValue, SocietyDefs, SocietySettings, Stock,
};
use common::{headless, spawn_person};

fn defs(script: &str) -> SocietyDefs {
    // The script is embedded in a RON string, so its own `"bread"` quotes have to be escaped.
    let script = script.replace('"', "\\\"");
    SocietyDefs::from_ron(&format!(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [(
                id: "eat",
                target: Place("table"),
                duration_hours: 6.0,
                consumes: [("bread", 1.0)],
                considerations: [(input: Need("food"), curve: Step(0.0))],
                satisfies: [], interaction: None, barks: [],
            )],
            triggers: [(id: "peek", on: Custom("poke"), then: "{script}")],
        )"#
    ))
    .expect("the test definitions parse")
}

/// A village with one table, so the economy system has a place to work.
fn village(script: &str) -> (App, Entity) {
    let settings = SocietySettings {
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    let mut app = headless(defs(script), settings);
    app.world_mut().spawn((
        Name::new("table"),
        bevy_society::Place { capacity: 8 },
        bevy_society::PlaceTags(vec!["table".to_owned()]),
        Transform::default(),
    ));
    let person = spawn_person(&mut app, "A", Vec2::ZERO);
    // **Prime the price before the trigger fires.** `update_prices` runs in `SocietySystems::Interact`, which
    // is after `Perceive` froze the snapshot, so the very first tick's snapshot still has the empty `Prices`
    // resource. Four updates give the economy a couple of ticks to price the goods, which is the state a rule
    // actually wakes up in on any tick but the first.
    for _ in 0..4 {
        app.update();
    }
    (app, person)
}

fn poke(app: &mut App, person: Entity) -> (Vec<ScriptEvent>, Vec<ScriptError>) {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(person),
        target: None,
    });
    let (mut events, mut errors) = (Vec::new(), Vec::new());
    for _ in 0..2 {
        app.update();
        events.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ScriptEvent>>()
                .drain(),
        );
        errors.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ScriptError>>()
                .drain(),
        );
    }
    (events, errors)
}

fn value(events: &[ScriptEvent], key: &str) -> ScriptValue {
    events
        .iter()
        .find_map(|e| {
            e.data
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_else(|| panic!("no '{key}' in {events:?}"))
}

/// **A person with nothing reports nothing, and the price of a good nobody holds is double.**
///
/// The double is the model, not a coincidence: the corpus's `v(r)` is `2(1 − r)`, and a village that holds
/// none of a good sits at `r = 0`. Asserting it end to end is what proves the script is reading the live
/// `Prices` and not a default.
#[test]
fn a_script_reads_an_empty_persons_store_and_the_scarcity_price() {
    let (mut app, person) = village(
        r#"fx.emit("peek", #{ held: ctx.stock(actor, "bread"), price: ctx.price("bread") })"#,
    );
    let (events, errors) = poke(&mut app, person);
    assert!(errors.is_empty(), "the script must run: {errors:?}");
    assert_eq!(value(&events, "held"), ScriptValue::Float(0.0));
    assert_eq!(
        value(&events, "price"),
        ScriptValue::Float(2.0),
        "an empty village prices the good at double the nominal unit"
    );
}

/// **What a person holds reaches the script, and the price reads it per head.**
///
/// Setting the store directly is the honest way to ask: the question is whether `ctx.stock` reads the
/// component and whether `ctx.price` reads the village total it is part of, not whether an activity can
/// complete in two ticks.
#[test]
fn a_script_reads_the_person_store_and_the_price_falls_as_the_village_fills() {
    let (mut app, person) = village(
        r#"fx.emit("peek", #{ held: ctx.stock(actor, "bread"), price: ctx.price("bread") })"#,
    );
    // Half a loaf a head, in a village of one: exactly half of `Tuning::price_full_store`, which the paper
    // calls the nominal unit.
    app.world_mut().entity_mut(person).insert(Stock(vec![0.5]));
    // One tick to price the store and freeze it into the snapshot; the trigger's own tick then reads it,
    // because a script sees the tick start.
    app.update();
    let (events, errors) = poke(&mut app, person);
    assert!(errors.is_empty(), "the script must run: {errors:?}");
    assert_eq!(value(&events, "held"), ScriptValue::Float(0.5));
    assert_eq!(
        value(&events, "price"),
        ScriptValue::Float(1.0),
        "half a day's supply a head is the nominal unit"
    );
}

/// An unknown good name is an error naming it, not a silent zero: a rule that asks for `"bred"` must be
/// told, which is the same contract `ctx.need` and `ctx.signal` keep.
#[test]
fn an_unknown_good_is_an_error_that_names_it() {
    let (mut app, person) = village(r#"ctx.price("bred")"#);
    let (_, errors) = poke(&mut app, person);
    assert_eq!(errors.len(), 1, "one failed script: {errors:?}");
    assert!(
        errors[0].message.contains("bred"),
        "the error must name the good: {:?}",
        errors[0].message
    );
}
