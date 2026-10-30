//! A trigger with an `area` fires only for an actor standing inside it, and an event that binds no
//! actor cannot carry one.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{FireTrigger, ScriptEvent, SocietyDefs};
use common::{headless, settings, spawn_person};

const AREA: &str = r#"(
    triggers: [(
        id: "poke",
        on: Custom("poke"),
        then: "fx.emit(\"met\", #{})",
        area: Some((min: (0.0, 0.0), max: (10.0, 10.0))),
    )],
)"#;

fn poke(app: &mut App, who: Entity) -> usize {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(who),
        target: None,
    });
    let mut met = 0;
    for _ in 0..2 {
        app.update();
        met += app
            .world_mut()
            .resource_mut::<Messages<ScriptEvent>>()
            .drain()
            .filter(|e| e.name == "met")
            .count();
    }
    met
}

#[test]
fn an_area_trigger_fires_only_for_an_actor_inside_it() {
    let mut app = headless(SocietyDefs::from_ron(AREA).expect("parses"), settings());
    let inside = spawn_person(&mut app, "In", Vec2::new(5.0, 5.0));
    let outside = spawn_person(&mut app, "Out", Vec2::new(30.0, 5.0));
    let edge = spawn_person(&mut app, "Edge", Vec2::new(10.0, 0.0));
    for _ in 0..2 {
        app.update();
    }

    assert_eq!(poke(&mut app, inside), 1, "inside fires");
    assert_eq!(poke(&mut app, outside), 0, "outside does not");
    assert_eq!(poke(&mut app, edge), 1, "the edge is inside");

    let mut t = app
        .world_mut()
        .get_mut::<Transform>(inside)
        .expect("has a transform");
    t.translation.x = 50.0;
    assert_eq!(poke(&mut app, inside), 0, "walking out stops it");
}

#[test]
fn an_area_on_an_event_with_no_actor_is_refused() {
    let err = SocietyDefs::from_ron(
        r#"(triggers: [(
            id: "t", on: Tick, then: "",
            area: Some((min: (0.0, 0.0), max: (1.0, 1.0))),
        )])"#,
    )
    .err()
    .expect("Tick binds no actor");
    assert!(err.to_string().contains("trigger 't'"), "{err}");

    let err = SocietyDefs::from_ron(
        r#"(triggers: [(
            id: "t", on: Custom("x"), then: "",
            area: Some((min: (5.0, 0.0), max: (1.0, 1.0))),
        )])"#,
    )
    .err()
    .expect("inverted corners");
    assert!(err.to_string().contains("min <= max"), "{err}");
}

#[test]
fn a_fired_trigger_is_reported_with_its_id_and_actor() {
    use bevy_society::{PersonId, TriggerFired};
    let mut app = headless(SocietyDefs::from_ron(AREA).expect("parses"), settings());
    let inside = spawn_person(&mut app, "In", Vec2::new(5.0, 5.0));
    let outside = spawn_person(&mut app, "Out", Vec2::new(30.0, 5.0));
    for _ in 0..2 {
        app.update();
    }
    let mut fired = Vec::new();
    for who in [outside, inside] {
        app.world_mut().write_message(FireTrigger {
            name: "poke".to_owned(),
            actor: Some(who),
            target: None,
        });
        for _ in 0..2 {
            app.update();
            fired.extend(
                app.world_mut()
                    .resource_mut::<Messages<TriggerFired>>()
                    .drain(),
            );
        }
    }
    assert_eq!(fired.len(), 1, "only the person inside the area fired it");
    assert_eq!(fired[0].trigger, "poke");
    assert_eq!(fired[0].actor, Some(PersonId(0)));
    assert_eq!(fired[0].target, None);
}
