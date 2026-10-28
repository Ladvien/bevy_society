//! **A walk that makes no progress ends** (issue #26).
//!
//! Under `Locomotion::Host` the crate leaves `travel_timeout_hours` unset, so a destination nobody can
//! reach never ends. `Tuning::stall_timeout_hours` ends it by *lack of progress* rather than total time:
//! a walker who closes on the target, however slowly, is never cut.

use crate::common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, Autonomous, Doing, DoingTarget, Home, Locomotion, Place, PlaceTags,
    SocietyClock, SocietyDefs, SocietySettings,
};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9)],
            activities: [
                (
                    id: "eat",
                    target: Place("food"),
                    duration_hours: 0.75,
                    considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("food", 1.2)],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

/// A host-walked person at the origin and a tavern 8 m away. One tick is 1/64 s of a 64 s day, so a game
/// hour is about 171 ticks.
fn app(stall: Option<f32>) -> (App, Entity, Entity) {
    let mut settings = SocietySettings {
        locomotion: Locomotion::Host,
        tick_secs: 1.0 / 64.0,
        seconds_per_day: 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = None;
    settings.tuning.stall_timeout_hours = stall;
    let mut app = headless(defs(), settings);
    let home = app
        .world_mut()
        .spawn((
            Name::new("home"),
            Place { capacity: 4 },
            PlaceTags(vec![]),
            Transform::default(),
        ))
        .id();
    app.world_mut().spawn((
        Name::new("tavern"),
        Place { capacity: 8 },
        PlaceTags(vec!["food".to_owned()]),
        Transform::from_xyz(8.0, 0.0, 0.0),
    ));
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(home), Autonomous));
    (app, person, home)
}

fn now(app: &App) -> f64 {
    app.world().resource::<SocietyClock>().now_days
}

fn endings(app: &mut App) -> Vec<ActivityEnded> {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
        .drain()
        .collect()
}

/// Ticks until `person` has an unarrived `Doing`, returning the game-day it started.
fn until_walking(app: &mut App, person: Entity) -> f64 {
    for _ in 0..200 {
        app.update();
        if let Some(doing) = app.world().get::<Doing>(person)
            && doing.arrived_day.is_none()
            && matches!(doing.target, DoingTarget::Place(_))
        {
            return doing.started_day;
        }
    }
    panic!("the person never started walking");
}

#[test]
fn a_host_that_never_moves_the_person_ends_the_walk_after_the_stall_window() {
    let (mut app, person, _) = app(Some(1.0));
    let started = until_walking(&mut app, person);
    let _ = endings(&mut app);
    let mut ended_at = None;
    for _ in 0..400 {
        app.update();
        if endings(&mut app)
            .iter()
            .any(|e| e.person == person && e.activity == "eat" && !e.completed)
        {
            ended_at = Some(now(&app));
            break;
        }
    }
    let hours = (ended_at.expect("a motionless walk must end interrupted") - started) * 24.0;
    assert!(
        (1.0..1.2).contains(&hours),
        "ended after {hours:.3} game-hours, expected just over the 1.0 h window"
    );
}

#[test]
fn a_slow_walker_who_keeps_closing_is_never_cut() {
    // A 0.1 h window is ~17 ticks; the host moves 0.05 m a tick, so 0.25 m of progress takes 5.
    let (mut app, person, _) = app(Some(0.1));
    until_walking(&mut app, person);
    let _ = endings(&mut app);
    let mut x = app
        .world()
        .get::<Transform>(person)
        .map_or(0.0, |t| t.translation.x);
    for _ in 0..170 {
        x += 0.05;
        app.world_mut()
            .entity_mut(person)
            .insert(Transform::from_xyz(x, 0.0, 0.0));
        app.update();
        let cut: Vec<_> = endings(&mut app)
            .into_iter()
            .filter(|e| e.person == person && !e.completed)
            .collect();
        assert!(
            cut.is_empty(),
            "a walker closing on its target was cut at x = {x:.2}: {cut:?}"
        );
    }
    let doing = app.world().get::<Doing>(person).expect("still doing it");
    assert!(
        doing.arrived_day.is_some(),
        "8 m of walking at 0.05 m a tick has arrived by x = {x:.2}"
    );
}

#[test]
fn no_stall_timeout_leaves_a_motionless_walk_alone() {
    let (mut app, person, _) = app(None);
    until_walking(&mut app, person);
    let _ = endings(&mut app);
    for _ in 0..800 {
        app.update();
        let cut = endings(&mut app)
            .into_iter()
            .any(|e| e.person == person && !e.completed);
        assert!(!cut, "with no stall timeout nothing ends a motionless walk");
    }
}
