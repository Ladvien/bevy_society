//! **A `Field` consideration must score, and a `Gradient` target must produce a target.**
//!
//! `Signal::Field` returned a constant `0.0` and `ActivityTarget::Gradient` produced no targets, so a
//! definition using either was accepted by the definitions file and could **never be chosen**. The field
//! grids themselves work — measured, at a cell holding six people `field_value(Crowd, ..)` returns
//! **1.0** — so this was a wiring gap, and the crate's answer to it was a startup warning.
//!
//! A warning is the right thing for a known gap; it is the wrong thing for a *documented feature*. This
//! file is the minimal case the decision file asked for: **one person, one activity, one consideration**,
//! with a crowd already in one cell. If the score is still zero here, the bug is in the scoring path and
//! not in a crowd hiding it.
#![cfg(feature = "fields")]

use crate::common;

use bevy::prelude::*;
use bevy_society::{ActivityStarted, Autonomous, FieldInput, Person, SocietyDefs};
use common::{headless, settings, spawn_person};

/// An activity whose only consideration is the crowd field, targeting the crowd.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            activities: [(
                id: "join",
                target: Gradient(Crowd, true),
                duration_hours: 0.5,
                considerations: [(input: Field(Crowd), curve: Linear(1.0, 0.0))],
                satisfies: [("social", 0.5)],
            )],
        )"#,
    )
    .expect("the definitions parse")
}

/// A crowd in one place and an empty place beside it, so the field has a **slope**.
///
/// The first version of this test packed nine people into a single cell and expected a gradient. That
/// was wrong, and it is why the case matters: `field_value` divides a cell by the channel's **peak**, a
/// global maximum, so any cell in the hottest part of the grid reads 1.0 — a plateau, with
/// `gradient_target` correctly returning `None` because there is no direction to walk. A gradient needs
/// high cells *and* low ones next to each other.
fn crowd_field() -> (App, Entity) {
    let mut day = settings();
    day.seconds_per_day = 240.0;
    let mut app = headless(defs(), day);
    // A knot of people at x = 0 and nothing at x = 4, so the field falls off along +X.
    for i in 0..9 {
        let spot = Vec2::new(0.1 * (i % 3) as f32, 0.1 * (i / 3) as f32);
        let person = spawn_person(&mut app, &format!("P{i}"), spot);
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person));
    }
    // The chooser stands two metres away, where the slope toward the crowd is real.
    let chooser = spawn_person(&mut app, "Me", Vec2::new(2.0, 0.0));
    app.world_mut()
        .entity_mut(chooser)
        .insert((Autonomous, Person));
    // Two updates: the first builds `SocialFields` by deferred command.
    app.update();
    app.update();
    (app, chooser)
}

#[test]
fn a_field_consideration_scores_above_zero() {
    let (mut app, chooser) = crowd_field();

    // The field itself, read the same way the decision should read it.
    let value = app
        .world()
        .get_resource::<bevy_society::SocialFields>()
        .map(|fields| {
            let cell = fields.world_to_cell(Vec2::new(2.0, 0.0));
            fields.field_value(FieldInput::Crowd, &[], cell)
        })
        .expect("the fields plugin built its resource");
    println!("field_value(Crowd) at the chooser = {value:.3}");
    // The chooser is outside the knot, so its own cell is *empty* — that is what makes this a field
    // reading rather than a crowd count. The slope is what the activity walks along.
    assert!(
        value <= 0.5,
        "the chooser must stand off the crowd so its cell is not the peak: {value:.3}"
    );
    let gradient = app
        .world()
        .get_resource::<bevy_society::SocialFields>()
        .and_then(|fields| {
            fields.gradient_target(FieldInput::Crowd, &[], Vec2::new(2.0, 0.0), true)
        });
    println!("gradient target = {gradient:?}");
    assert!(
        gradient.is_some(),
        "a populated field with an empty cell beside the chooser must give a direction to walk"
    );

    // **Long enough for the fallback to expire.** `idle` lasts one game-hour and a tick at
    // 240 s/day is 0.0016 game-hours, so the first fallback holds for 640 ticks. Two earlier versions of
    // this test ran 400 and concluded the field was dead — the person was simply still idle.
    let mut starts = 0;
    for _ in 0..2400 {
        app.update();
        starts += app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
            .drain()
            .filter(|start| start.person == chooser && start.activity == "join")
            .count();
    }
    println!("times `join` was chosen: {starts}");
    assert!(
        starts > 0,
        "an activity whose only consideration is a populated `Field` was never chosen in 2 400 ticks \
         (3.75 game-hours, against a 1-hour fallback). `Signal::Field` scored 0 or `Gradient` yielded \
         no target, so the definition is dead"
    );
}
