//! `ctx.position` and `ctx.distance` read where persons stood at the tick start, on the society plane.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{FireTrigger, ScriptEvent, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Fires only when the two are within 10 m and the actor is east of x = 0.
const NEAR: &str = r#"(
    triggers: [(
        id: "near",
        on: Custom("poke"),
        if: Some("ctx.distance(actor, target) <= 10.0 && ctx.position(actor).x > 0.0"),
        then: "fx.emit(\"met\", #{})",
    )],
)"#;

fn poke(app: &mut App, actor: Entity, target: Entity) -> usize {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(actor),
        target: Some(target),
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
fn a_condition_can_ask_how_far_apart_and_where() {
    let mut app = headless(SocietyDefs::from_ron(NEAR).expect("parses"), settings());
    let a = spawn_person(&mut app, "A", Vec2::new(5.0, 5.0));
    let close = spawn_person(&mut app, "Close", Vec2::new(8.0, 9.0)); // 5 m away
    let far = spawn_person(&mut app, "Far", Vec2::new(25.0, 5.0)); // 20 m away
    let west = spawn_person(&mut app, "West", Vec2::new(-4.0, 5.0)); // 9 m from A
    for _ in 0..2 {
        app.update();
    }

    assert_eq!(poke(&mut app, a, close), 1, "5 m apart, actor east of 0");
    assert_eq!(poke(&mut app, a, far), 0, "20 m apart");
    assert_eq!(
        poke(&mut app, west, a),
        0,
        "within range but the actor is west of 0"
    );

    // The read is the tick-start position, so moving someone changes it from the next tick.
    app.world_mut()
        .get_mut::<Transform>(far)
        .expect("has a transform")
        .translation
        .x = 9.0;
    assert_eq!(poke(&mut app, a, far), 1, "now 4 m away");
}
