//! `Doing` says what a person is doing **now**, not what it is walking towards.
//!
//! `Doing::activity` names the activity that was chosen, from the moment it is chosen. The activity's needs and
//! duration start counting only when `arrived_day` is set, so a reader that takes `activity` at face value
//! reports a person as asleep while it walks to bed and is paid nothing. Measured on the shipped village at
//! 64 s/day, that over-reports sleep by more than three points of every person-tick.
//!
//! `Doing::label` owns the convention — `"travelling:<activity>"` before arrival, the bare name after — so a
//! HUD, a log or a script does not have to rediscover it. The evidence harness had invented exactly this
//! privately before the helper existed, which is the argument for putting it on the component.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Doing, DoingTarget, Home, Place, PlaceTags, SocietyDefs, SocietySettings,
};
use common::{headless, spawn_person};

fn setup() -> (App, Entity) {
    let mut settings: SocietySettings = common::settings();
    settings.seconds_per_day = 64.0;
    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [(id: "rest", decay_per_day: 0.7, initial: 0.2)],
            activities: [(
                id: "sleep",
                target: Home,
                duration_hours: 8.0,
                rank: 1,
                considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                satisfies: [("rest", 0.15)],
            )],
        )"#,
    )
    .expect("the definitions parse");
    let mut app = headless(defs, settings);
    // The bed is far away, so the walk is long relative to an activity in this test's clock.
    let bed = app
        .world_mut()
        .spawn((
            Name::new("bed"),
            Place { capacity: 8 },
            PlaceTags(vec!["rest".to_owned()]),
            Transform::from_xyz(6.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "Walker", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(bed), Autonomous));
    (app, person)
}

#[test]
fn a_person_walking_to_bed_is_labelled_travelling_not_sleeping() {
    let (mut app, person) = setup();
    let defs = app.world().resource::<SocietyDefs>().clone();

    // Wait until the person has chosen `sleep` and is on its way.
    let mut saw_travelling = false;
    for _ in 0..600 {
        app.update();
        let Some(doing) = app.world().get::<Doing>(person) else {
            continue;
        };
        if defs.activity_name(doing.activity) == "sleep" {
            let label = doing.label(&defs);
            assert!(doing.is_travelling(), "the walk must still be in progress");
            assert_eq!(
                label, "travelling:sleep",
                "a walking person is not asleep; `Doing::activity` names the target, `label` must not"
            );
            saw_travelling = true;
            break;
        }
    }
    assert!(
        saw_travelling,
        "the person never chose `sleep` within 600 ticks"
    );
}

#[test]
fn the_label_drops_the_prefix_once_the_person_has_arrived() {
    let (mut app, person) = setup();
    let defs = app.world().resource::<SocietyDefs>().clone();

    // Once `arrived_day` is set the activity has begun and the bare name is the truth.
    for _ in 0..4000 {
        app.update();
        let Some(doing) = app.world().get::<Doing>(person) else {
            continue;
        };
        if defs.activity_name(doing.activity) == "sleep" && !doing.is_travelling() {
            assert_eq!(doing.label(&defs), "sleep");
            return;
        }
    }
    panic!("the person never arrived at the bed within 4000 ticks");
}

#[test]
fn the_two_readings_agree_on_a_person_that_never_moves() {
    // A hand-built `Doing` with an arrival: no walking, so `label` must be the bare activity name. This pins
    // that the prefix is a statement about `arrived_day`, not about the activity.
    let (mut app, person) = setup();
    let defs = app.world().resource::<SocietyDefs>().clone();
    let bed = app
        .world()
        .get::<Home>(person)
        .map_or(Entity::PLACEHOLDER, |home| home.0);
    let sleep = defs
        .activity_id("sleep")
        .expect("the definitions declare `sleep`");
    app.world_mut().entity_mut(person).insert(Doing {
        activity: sleep,
        target: DoingTarget::Place(bed),
        started_day: 0.0,
        arrived_day: Some(0.0),
    });
    let doing = app.world().get::<Doing>(person).expect("inserted");
    assert!(!doing.is_travelling());
    assert_eq!(doing.label(&defs), "sleep");
    assert_eq!(
        defs.activity_name(doing.activity),
        "sleep",
        "`activity` still names the target activity, which is what a raw reader sees"
    );
}
