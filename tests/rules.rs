//! Triggers and scripts (plan D8): `once`, cooldowns, script errors, the operation limit, snapshot
//! isolation and rumor reach.

mod common;

use bevy::prelude::*;
use bevy_society::{
    FireTrigger, Knowledge, PlantRumor, RulesPlugin, ScriptError, ScriptEvent, ScriptValue,
    SocietyDefs,
};
use common::{headless, settings, spawn_person};

/// Runs the first updates, which prime `Time<Fixed>`, and returns what they reported.
fn first_tick(app: &mut App) -> (Vec<ScriptEvent>, Vec<ScriptError>) {
    step(app, 2)
}

fn defs(text: &str) -> SocietyDefs {
    SocietyDefs::from_ron(text).expect("the test definitions parse")
}

fn fire(app: &mut App, name: &str, actor: Option<Entity>, target: Option<Entity>) {
    app.world_mut().write_message(FireTrigger {
        name: name.to_owned(),
        actor,
        target,
    });
}

/// Advances the app, collecting the events and errors written on every update.
///
/// Draining only at the end would lose everything but the last two updates: Bevy keeps a message for
/// two buffer swaps.
fn step(app: &mut App, updates: usize) -> (Vec<ScriptEvent>, Vec<ScriptError>) {
    let (mut events, mut errors) = (Vec::new(), Vec::new());
    for _ in 0..updates {
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

fn named(events: Vec<ScriptEvent>, name: &str) -> Vec<ScriptEvent> {
    events.into_iter().filter(|e| e.name == name).collect()
}

#[test]
fn a_once_trigger_fires_once_per_actor() {
    let mut app = headless(
        defs(
            r#"(
                triggers: [(
                    id: "poke",
                    on: Custom("poke"),
                    then: "fx.emit(\"met\", #{ n: 1 })",
                    once: true,
                )],
            )"#,
        ),
        settings(),
    );
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(2.0, 0.0));
    first_tick(&mut app);

    let mut met = Vec::new();
    for _ in 0..10 {
        fire(&mut app, "poke", Some(a), None);
        met.extend(named(step(&mut app, 1).0, "met"));
    }
    fire(&mut app, "poke", Some(b), None);
    met.extend(named(step(&mut app, 2).0, "met"));

    assert_eq!(met.len(), 2, "once per actor: A once, B once");
    assert_eq!(met[0].actor, Some(a), "the event carries A");
    assert_eq!(
        met[0].data,
        vec![("n".to_owned(), ScriptValue::Int(1))],
        "the payload survives as an integer"
    );
    assert_eq!(met[1].actor, Some(b));
}

#[test]
fn a_cooldown_trigger_fires_then_holds() {
    // One game-day is 8 ticks, so one game-hour is 8/24 ticks: 12 ticks cover two hours and a half.
    // One game-day is 240 ticks here, so one tick is six game-minutes: a one-hour cooldown covers
    // ten ticks.
    let mut settings = settings();
    settings.seconds_per_day = 240.0 / 64.0;
    let mut app = headless(
        defs(
            r#"(
                triggers: [(
                    id: "clocked",
                    on: Tick,
                    then: "fx.emit(\"met\", #{})",
                    cooldown_hours: 1.0,
                )],
            )"#,
        ),
        settings,
    );
    let (opening, _) = first_tick(&mut app);
    assert_eq!(
        named(opening, "met").len(),
        1,
        "the first society tick fires the trigger once"
    );

    // A one-hour cooldown is ten ticks long here, so the following six ticks stay gated.
    let (later, _) = step(&mut app, 6);
    assert!(
        named(later, "met").is_empty(),
        "the one-hour cooldown still holds on the next ticks"
    );
}

#[test]
fn a_script_error_is_reported_and_the_app_keeps_running() {
    let mut app = headless(
        defs(
            r#"(
                needs: [(id: "rest", decay_per_day: 0.5)],
                triggers: [(
                    id: "bad",
                    on: Custom("poke"),
                    then: "ctx.need(actor, \"thirst\")",
                )],
            )"#,
        ),
        settings(),
    );
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    first_tick(&mut app);

    fire(&mut app, "poke", Some(a), None);
    let (_, reported) = step(&mut app, 2);

    assert_eq!(reported.len(), 1, "one failed script: {reported:?}");
    assert!(
        reported[0].message.contains("thirst"),
        "the message names the unknown need: {}",
        reported[0].message
    );

    // The app keeps updating: the clock still advances.
    let before = app.world().resource::<bevy_society::SocietyClock>().tick;
    step(&mut app, 3);
    let after = app.world().resource::<bevy_society::SocietyClock>().tick;
    assert!(after > before, "the society kept ticking after the error");
}

#[test]
fn an_infinite_loop_hits_the_operation_limit() {
    let mut app = headless(
        defs(
            r#"(
                triggers: [(id: "spin", on: Custom("poke"), then: "loop {}")],
            )"#,
        ),
        settings(),
    );
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    first_tick(&mut app);

    fire(&mut app, "poke", Some(a), None);
    let (_, reported) = step(&mut app, 2);

    assert_eq!(reported.len(), 1, "the operation limit stops the loop");
    assert!(
        reported[0].message.to_lowercase().contains("operations")
            || reported[0].message.to_lowercase().contains("limit"),
        "the message names the limit: {}",
        reported[0].message
    );
}

#[test]
fn a_trigger_sees_the_tick_start_snapshot_not_its_own_writes() {
    let mut app = headless(
        defs(
            r##"(
                triggers: [
                    (id: "raise", on: Tick, then: "let p = ctx.persons(); fx.adjust(p[0], p[1], \"affinity\", 0.9)"),
                    (id: "look", on: Tick, then: "let p = ctx.persons(); if ctx.affinity(p[0], p[1]) > 0.5 { fx.emit(\"saw\", #{}) }"),
                ],
            )"##,
        ),
        settings(),
    );
    spawn_person(&mut app, "A", Vec2::ZERO);
    spawn_person(&mut app, "B", Vec2::new(2.0, 0.0));

    // The first tick's snapshot predates its own write, so its second trigger sees no affinity.
    let (opening, _) = first_tick(&mut app);
    assert!(
        named(opening, "saw").is_empty(),
        "the raise of a tick is not visible to a trigger of the same tick"
    );
    assert_eq!(
        named(step(&mut app, 1).0, "saw").len(),
        1,
        "the next tick sees the affinity the previous tick applied"
    );
}

#[test]
fn a_rumor_reach_trigger_fires_once_when_the_rumor_reaches_its_fraction() {
    let mut app = headless(
        defs(
            r#"(
                topics: [(id: "honesty")],
                deeds: [(id: "steal", topic: "honesty", harm: -0.7)],
                triggers: [(
                    id: "rumor",
                    on: RumorReach(deed: "steal", fraction: 0.5),
                    then: "fx.emit(\"spread\", #{})",
                    once: true,
                )],
            )"#,
        ),
        settings(),
    );
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(2.0, 0.0));
    first_tick(&mut app);

    app.world_mut().write_message(PlantRumor {
        origin: a,
        subject: b,
        patient: None,
        deed: "steal".to_owned(),
        truth: false,
    });
    let spread = named(step(&mut app, 3).0, "spread");
    assert_eq!(spread.len(), 1, "one rumor, one crossing, one event");
    assert_eq!(spread[0].actor, None, "the event has no actor");
    assert!(
        app.world().resource::<bevy_society::RumorBook>().rumors[0].alive,
        "the rumor is alive"
    );
    let _ = app.world().get::<Knowledge>(a);
}

#[test]
fn the_plugin_works_without_any_trigger_defined() {
    let mut app = headless(SocietyDefs::default(), settings());
    spawn_person(&mut app, "A", Vec2::ZERO);
    let (events, errors) = step(&mut app, 5);
    assert!(named(events, "met").is_empty());
    assert!(errors.is_empty());
    assert!(
        app.is_plugin_added::<RulesPlugin>(),
        "the group added the plugin"
    );
}
