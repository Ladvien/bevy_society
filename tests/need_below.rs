//! A `NeedBelow` trigger fires once per person per crossing, and a person's level never arms or
//! disarms it for anyone else.
//!
//! The documented contract: it fires when a person's need *first falls below* the line, and re-arms
//! only once that person's need is back above `below + 0.05`, so a level hovering at the line does
//! not fire every tick.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{DefsFile, Needs, PersonId, SocietyDefs, TriggerFired, reload_defs};
use common::{headless, settings, spawn_person};

const DEFS: &str = r#"(
    needs: [(id: "social", decay_per_day: 0.0)],
    triggers: [(
        id: "lonely", on: NeedBelow(need: "social", below: 0.3),
        then: "fx.emit(\"lonely\", #{})",
    )],
)"#;

/// A society of two people at levels `low` and `high` for the one need.
fn two_people(defs: &str, low: f32, high: f32) -> (App, Entity, Entity) {
    let mut app = headless(SocietyDefs::from_ron(defs).expect("parses"), settings());
    let a = spawn_person(&mut app, "Low", Vec2::ZERO);
    let b = spawn_person(&mut app, "High", Vec2::ZERO);
    for _ in 0..2 {
        app.update();
    }
    set(&mut app, a, low);
    set(&mut app, b, high);
    (app, a, b)
}

fn set(app: &mut App, who: Entity, level: f32) {
    app.world_mut()
        .get_mut::<Needs>(who)
        .expect("has needs")
        .0[0] = level;
}

/// Runs `ticks` updates and returns every `(trigger, actor)` that fired, in order.
fn run(app: &mut App, ticks: usize) -> Vec<(String, Option<PersonId>)> {
    let mut fired = Vec::new();
    for _ in 0..ticks {
        app.update();
        fired.extend(
            app.world_mut()
                .resource_mut::<Messages<TriggerFired>>()
                .drain()
                .map(|f| (f.trigger, f.actor)),
        );
    }
    fired
}

#[test]
fn need_below_fires_once_per_person_per_crossing() {
    let (mut app, _low, high) = two_people(DEFS, 0.2, 0.9);

    // Person 0 is below and person 1 is above the line: one firing, however long it lasts.
    let first = run(&mut app, 10);
    assert_eq!(first, [("lonely".to_owned(), Some(PersonId(0)))], "{first:?}");

    // A second person crossing while the first is still below fires for the second only.
    set(&mut app, high, 0.2);
    let second = run(&mut app, 3);
    assert_eq!(second, [("lonely".to_owned(), Some(PersonId(1)))], "{second:?}");

    // Both are below and disarmed: nothing more, ever.
    assert!(run(&mut app, 20).is_empty());
}

#[test]
fn a_persons_filter_sees_that_persons_own_crossing() {
    // Both are below the line and nobody is above it. The first person's event used to spend the
    // town's one flag, so a trigger for person 1 never saw an event of its own.
    let defs = DEFS.replace(
        "then:",
        "persons: [1],\n        then:",
    );
    let (mut app, _a, _b) = two_people(&defs, 0.2, 0.2);
    let fired = run(&mut app, 10);
    assert_eq!(fired, [("lonely".to_owned(), Some(PersonId(1)))], "{fired:?}");
}

#[test]
fn two_triggers_on_one_threshold_each_fire_once() {
    let defs = r#"(
        needs: [(id: "social", decay_per_day: 0.0)],
        triggers: [
            (id: "first", on: NeedBelow(need: "social", below: 0.3), then: ""),
            (id: "second", on: NeedBelow(need: "social", below: 0.3), then: ""),
        ],
    )"#;
    let (mut app, _a, _b) = two_people(defs, 0.2, 0.9);
    let fired = run(&mut app, 10);
    let ids: Vec<&str> = fired.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["first", "second"], "{fired:?}");
}

#[test]
fn a_person_rearms_only_above_the_band() {
    let (mut app, a, _b) = two_people(DEFS, 0.2, 0.9);
    assert_eq!(run(&mut app, 3).len(), 1, "falls below: fires");

    // Inside the band (0.3 to 0.35) the person is neither armed nor below.
    set(&mut app, a, 0.33);
    assert!(run(&mut app, 3).is_empty());
    set(&mut app, a, 0.2);
    assert!(run(&mut app, 3).is_empty(), "hovering at the line does not re-fire");

    // Back above the band for one tick, then below again: a new crossing.
    set(&mut app, a, 0.4);
    assert!(run(&mut app, 2).is_empty());
    set(&mut app, a, 0.2);
    let again = run(&mut app, 3);
    assert_eq!(again, [("lonely".to_owned(), Some(PersonId(0)))], "{again:?}");
}

#[test]
fn an_edit_that_adds_a_threshold_does_not_rearm_the_old_one() {
    let (mut app, _a, _b) = two_people(DEFS, 0.2, 0.9);
    assert_eq!(run(&mut app, 3).len(), 1);

    // A second trigger on the same need, ahead of the first.
    let edited = r#"(
        needs: [(id: "social", decay_per_day: 0.0)],
        triggers: [
            (id: "very_lonely", on: NeedBelow(need: "social", below: 0.1), then: ""),
            (id: "lonely", on: NeedBelow(need: "social", below: 0.3), then: "fx.emit(\"lonely\", #{})"),
        ],
    )"#;
    reload_defs(app.world_mut(), DefsFile::from_ron(edited).expect("parses")).expect("reloads");
    // The person is still below 0.3 and is still disarmed there; 0.1 is a line the person is not under.
    assert!(run(&mut app, 5).is_empty());

    // Falling under the new line is a crossing of that line alone.
    let a = {
        let mut q = app.world_mut().query::<(Entity, &PersonId)>();
        q.iter(app.world()).find(|(_, id)| id.0 == 0).map(|(e, _)| e).expect("person 0")
    };
    set(&mut app, a, 0.05);
    let fired = run(&mut app, 5);
    assert_eq!(fired, [("very_lonely".to_owned(), Some(PersonId(0)))], "{fired:?}");
}
