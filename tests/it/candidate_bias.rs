//! **An activity's odds do not depend on how many targets it happens to have.**
//!
//! `targets_for` gives a `Person` activity one candidate per person in perception and a `Place`
//! activity one per matching place. Selection then draws over every candidate, so an activity's
//! selection mass is `weight · score · (number of targets)` — a crowd of eight makes gossip eight
//! times as likely as the same activity with one neighbour. Measured on the starter pack with
//! everyone standing on one plaza (so travel is zero and only choice differs):
//!
//! | people | `food` need at 0 | `fun` need at 0 |
//! |---|---|---|
//! | 1 | 0.0% | 5.9% |
//! | 4 | 4.5% | 5.4% |
//! | 8 | 9.6% | 28.2% |
//! | 16 | 11.4% | **43.4%** |
//!
//! Starvation growing with the number of *neighbours*, with every rate held fixed, is the signature:
//! Dill's weight-based random draws over **options**, and an option is an action, not an
//! action-target pair.
use crate::common;

use bevy::prelude::*;
use bevy_society::{ActivityStarted, Autonomous, Home, Place, PlaceTags, SocietyDefs};
use common::{headless, settings};

/// Two activities that are identical except for their target kind, so the only difference in their
/// odds is how many candidates each generates.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            activities: [
                (
                    id: "visit",
                    target: Person,
                    duration_hours: 0.5,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 0.5)],
                ),
                (
                    id: "rest_here",
                    target: Place("gathering"),
                    duration_hours: 0.5,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 0.5)],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

fn crowded(population: usize) -> (u32, u32) {
    let mut app = headless(defs(), settings());
    let plaza = app
        .world_mut()
        .spawn((
            Name::new("plaza"),
            Place { capacity: 64 },
            PlaceTags(vec!["gathering".to_owned()]),
            Transform::default(),
        ))
        .id();
    for i in 0..population {
        let person = common::spawn_person(&mut app, &format!("P{i}"), Vec2::ZERO);
        app.world_mut()
            .entity_mut(person)
            .insert((Home(plaza), Autonomous));
    }

    let mut visit = 0u32;
    let mut rest = 0u32;
    for _ in 0..900 {
        app.update();
        let started: Vec<ActivityStarted> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
            .drain()
            .collect();
        for message in started {
            match message.activity.as_str() {
                "visit" => visit += 1,
                "rest_here" => rest += 1,
                _ => {}
            }
        }
    }
    (visit, rest)
}

#[test]
fn a_crowd_does_not_multiply_a_person_activity_s_odds() {
    // One of each, at one person and at twelve. The two activities have identical scores by
    // construction, so their counts should stay in the same ballpark as the crowd grows.
    let (visit1, rest1) = crowded(1);
    let (visit12, rest12) = crowded(12);

    let share1 = visit1 as f32 / (visit1 + rest1).max(1) as f32;
    let share12 = visit12 as f32 / (visit12 + rest12).max(1) as f32;
    println!("one person: visit {visit1} rest {rest1} -> visit share {share1:.2}");
    println!("twelve:     visit {visit12} rest {rest12} -> visit share {share12:.2}");

    // With the bias, twelve neighbours make `visit` roughly twelve times as likely as `rest_here`,
    // i.e. a share near 0.92. Without it the share stays near a half.
    assert!(
        share12 < 0.70,
        "twelve neighbours must not tilt a Person activity to {share12:.2} of all starts \
         (visit {visit12}, rest_here {rest12}); an activity's odds must not scale with its \
         number of targets"
    );
}
