//! **A `Place` activity pays `satisfies` for its declared duration, and no longer.**
//!
//! `arrive_and_satisfy` pays `satisfies` on every tick while `Doing::arrived_day` is set, and
//! `end_activities` ends the activity once `(now - arrived) · 24 >= duration_hours`. That is monotone,
//! so the payout should be exactly `duration_hours` worth.
//!
//! Measured in the village layout over 17 game-days: **16.5 hours of sleep per sleep event**, against the
//! 8.0 the definition declares — and **54.2% of all person-ticks asleep** where one 8-hour sleep a day
//! allows 33%. The village's own series says 56.8%.
//!
//! So a `Place` activity overruns. This test pins the contract with a need that **cannot saturate** — a
//! `satisfies` rate low enough that the payout is a direct read of elapsed ticks — and a person who is
//! moved out of range mid-activity, which is what steering does to a standing villager.
mod common;

use bevy::prelude::*;
use bevy_society::{ActivityEnded, Autonomous, Home, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// A 2-hour activity paying **0.1 per game-hour** into a need with no decay.
///
/// The rate is deliberately small: a need clamps to `[0, 1]`, so a 1.0-per-hour activity pays 2.0 into a
/// 1.0 ceiling and reads as 1.0 — which is what a first version of this test measured, and it looked
/// like the payout was half. At 0.1/h the total is 0.2 and the clamp never applies, so the reading is
/// the elapsed time.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "rest", decay_per_day: 0.0, initial: 0.0)],
            activities: [(
                id: "doze",
                target: Place("rest"),
                duration_hours: 2.0,
                considerations: [],
                satisfies: [("rest", 0.1)],
            )],
        )"#,
    )
    .expect("the definitions parse")
}

/// Runs the activity to its end and returns `(completed, rest level)`.
fn run(displace: bool) -> (bool, f32) {
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(defs(), short_day);
    let hall = app
        .world_mut()
        .spawn((
            Name::new("hall"),
            Place { capacity: 8 },
            PlaceTags(vec!["rest".to_owned()]),
            Transform::default(),
        ))
        .id();
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(hall), Autonomous));

    let mut displaced = false;
    for _ in 0..20_000 {
        app.update();
        if displace && !displaced && app.world().get::<bevy_society::Doing>(person).is_some() {
            // Five metres away: well outside the 1.5 m interaction radius. The activity is already
            // "arrived"; the person has merely been pushed aside, as a crowd mover can.
            if let Some(mut transform) = app.world_mut().get_mut::<Transform>(person) {
                transform.translation = Vec3::new(5.0, 0.0, 0.0);
            }
            displaced = true;
        }
        let ended: Vec<ActivityEnded> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
            .drain()
            .collect();
        if let Some(end) = ended.iter().find(|end| end.activity == "doze") {
            let level = app
                .world()
                .get::<bevy_society::Needs>(person)
                .and_then(|n| n.0.first().copied())
                .unwrap_or(0.0);
            return (end.completed, level);
        }
    }
    panic!("the activity never ended in 20 000 ticks");
}

#[test]
fn a_two_hour_activity_pays_two_hours() {
    let (completed, paid) = run(false);
    println!("undisplaced: completed {completed}, paid {paid:.4} (0.2 expected)");
    assert!(completed, "it should complete");
    assert!(
        (paid - 0.2).abs() < 0.02,
        "a 2-hour activity at 0.1/h must pay about 0.2, not {paid:.3}"
    );
}

#[test]
fn being_pushed_away_does_not_extend_the_payout() {
    let (_, paid) = run(true);
    println!("displaced: paid {paid:.4} (0.2 expected)");
    assert!(
        paid <= 0.22,
        "a person pushed out of range must not keep being paid: {paid:.3}, against the 0.2 the \
         definition declares"
    );
}
