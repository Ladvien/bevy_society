//! **A crowded place admits a walker that a plain radius would turn away.**
//!
//! Everyone goes to bed at the same hour, so every resident of a home aims at the same coordinate inside
//! `interaction_radius`. ORCA holds two people's centres one body diameter apart, so the disc packs solid and
//! the queue behind it expires its whole `travel_timeout_hours`. Measured on `village-evening`: at every
//! timed-out `sleep` the goal already held a median of **6** bodies, and a 1.5 m disc seats about
//! `(1.5 / 0.7)^2 = 4.6`.
//!
//! `Tuning::crowded_arrival_growth` grows a `Place`'s accepted arrival boundary by one body diameter per
//! `sqrt(occupants)`. Setting it to `0.0` restores plain arrival exactly.
//!
//! These assert the **gate itself** rather than a whole village: the village's crowding needs four homes and a
//! shared gathering place interacting, and four attempts to reproduce it end to end produced identical numbers
//! with and without the feature. A gate test that cannot pass with the gate disabled is worth more than an
//! end-to-end test that passes either way.
mod common;

use bevy::prelude::*;
use bevy_society::{Doing, DoingTarget, Home, Place, PlaceTags, SocietyDefs, SocietySettings};

/// A place at the origin, `occupants` people already standing in it, and one walker `out` metres away with a
/// destination at the place.
fn scene(growth: f32, occupants: usize, out: f32) -> (App, Entity) {
    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [(id: "rest", decay_per_day: 0.6, initial: 0.1)],
            activities: [(
                id: "sleep",
                target: Home,
                duration_hours: 48.0,
                rank: 1,
                considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                satisfies: [("rest", 0.5)],
            )],
        )"#,
    )
    .expect("the definitions parse");
    let mut settings: SocietySettings = common::settings();
    settings.seconds_per_day = 64.0;
    settings.interaction_radius = 1.5;
    settings.tuning.crowded_arrival_growth = growth;
    settings.tuning.travel_timeout_hours = Some(10.0);
    let mut app = common::headless(defs, settings);
    let home = app
        .world_mut()
        .spawn((
            Name::new("dormitory"),
            Place { capacity: 32 },
            PlaceTags(vec!["rest".to_owned()]),
            Transform::default(),
        ))
        .id();
    // Bodies already inside the arrival disc, packed on a small ring so they are genuinely *in* it.
    for i in 0..occupants {
        let a = std::f32::consts::TAU * (i as f32) / (occupants.max(1) as f32);
        let person = common::spawn_person(
            &mut app,
            &format!("S{i}"),
            Vec2::new(0.4 * a.cos(), 0.4 * a.sin()),
        );
        let sleep = app
            .world()
            .resource::<SocietyDefs>()
            .activity_id("sleep")
            .expect("the definitions declare `sleep`");
        app.world_mut().entity_mut(person).insert((
            Home(home),
            Doing {
                activity: sleep,
                target: DoingTarget::Place(home),
                started_day: 0.0,
                arrived_day: Some(0.0),
            },
        ));
    }
    // The walker: at `out`, already walking home. `Doing` is set directly rather than left to the decider, so
    // the test measures the arrival gate and nothing else.
    let walker = common::spawn_person(&mut app, "Walker", Vec2::new(out, 0.0));
    let sleep = app
        .world()
        .resource::<SocietyDefs>()
        .activity_id("sleep")
        .expect("the definitions declare `sleep`");
    // **No `Destination`.** The walker stands still at `out`, which is what a person jammed behind a packed
    // disc looks like: nothing moves them, so only the arrival gate can admit them. Giving them a destination
    // would let `walk_toward_destination` walk them in and the test would pass with the gate disabled.
    app.world_mut().entity_mut(walker).insert((
        Home(home),
        Doing {
            activity: sleep,
            target: DoingTarget::Place(home),
            started_day: 0.0,
            arrived_day: None,
        },
    ));
    (app, walker)
}

/// Whether the walker's `Doing` ever records an arrival within `ticks`.
fn arrives(growth: f32, occupants: usize, out: f32) -> bool {
    let (mut app, walker) = scene(growth, occupants, out);
    for _ in 0..2_000 {
        app.update();
        if let Some(doing) = app.world().get::<Doing>(walker)
            && doing.arrived_day.is_some()
        {
            return true;
        }
    }
    false
}

#[test]
fn a_crowded_disc_admits_a_walker_a_plain_radius_would_refuse() {
    // Eight bodies inside a 1.5 m disc, which seats about 4.6. The walker stands at 2.2 m — outside the plain
    // radius, inside the widened one (`1.5 + 0.7 * sqrt(8) = 3.48`).
    assert!(
        !arrives(0.0, 8, 2.2),
        "with growth 0.0 the plain radius must refuse a walker at 2.2 m — if this arrives, the test is not \
         measuring the gate"
    );
    assert!(
        arrives(0.7, 8, 2.2),
        "a disc holding 8 bodies must admit a walker at 2.2 m: the queue is the crowd, not a missed path"
    );
}

#[test]
fn an_empty_disc_is_unaffected() {
    // No crowd, so the accepted boundary is the plain radius however large the growth is. A walker at 2.2 m
    // must be refused, which is what stops this feature from quietly widening every arrival in the crate.
    assert!(
        !arrives(0.7, 0, 2.2),
        "an empty place must not accept a walker at 2.2 m: growth must scale with the crowd, not replace the radius"
    );
}

/// Issue #25: bodies that stand on rings *beyond* `interaction_radius` — where the allowance itself admitted
/// them — still count, so the allowance keeps growing and the next ring is admitted.
#[test]
fn the_allowance_grows_with_arrivals_standing_beyond_the_radius() {
    // Radius 1.5, growth 0.7. Eight arrived bodies stand on a ring at 2.0 m, outside the 1.5 m disc the old
    // count used, which saw none of them. A walker at 3.0 m is admitted only when they count:
    // 1.5 + 0.7 * sqrt(8) = 3.48.
    let (mut app, walker) = scene(0.7, 0, 3.0);
    let home = match app.world().get::<Doing>(walker).map(|d| d.target) {
        Some(DoingTarget::Place(home)) => home,
        _ => panic!("the walker targets the home"),
    };
    let sleep = app
        .world()
        .resource::<SocietyDefs>()
        .activity_id("sleep")
        .expect("the definitions declare `sleep`");
    for i in 0..8 {
        let a = std::f32::consts::TAU * (i as f32) / 8.0;
        let person = common::spawn_person(
            &mut app,
            &format!("R{i}"),
            Vec2::new(2.0 * a.cos(), 2.0 * a.sin()),
        );
        app.world_mut().entity_mut(person).insert((
            Home(home),
            Doing {
                activity: sleep,
                target: DoingTarget::Place(home),
                started_day: 0.0,
                arrived_day: Some(0.0),
            },
        ));
    }
    let mut arrived = false;
    for _ in 0..2_000 {
        app.update();
        if let Some(doing) = app.world().get::<Doing>(walker)
            && doing.arrived_day.is_some()
        {
            arrived = true;
            break;
        }
    }
    assert!(
        arrived,
        "eight arrived bodies on a 2.0 m ring must widen the allowance to reach 3.0 m"
    );
}
