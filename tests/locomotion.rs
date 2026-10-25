//! **The host owns locomotion** (issue #1).
//!
//! The society decides *where* a person goes; a host with a street grid, terrain height, its own
//! houses and figures the society does not own decides *how* they get there. Under
//! [`Locomotion::Host`] no crate system writes a person's `Transform`, `Destination` stays the request
//! to read, and the host says a walk is impossible with `TravelFailed` rather than watching a timeout
//! fire.
//!
//! `Locomotion::Builtin` is the default and is unchanged.

mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, Autonomous, Destination, Doing, DoingTarget, Home, Locomotion, Place, PlaceTags,
    SocietyDefs, SocietySettings, TravelFailed,
};
use common::{headless, spawn_person};

/// A defs file with one place-targeted activity and one person-targeted one, so both the walk and the
/// chase can be observed.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "food", decay_per_day: 0.9), (id: "social", decay_per_day: 0.5)],
            activities: [
                (
                    id: "eat",
                    target: Place("food"),
                    duration_hours: 0.75,
                    considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("food", 1.2)],
                ),
                (
                    id: "chat",
                    target: Person,
                    duration_hours: 0.5,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 1.0)],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

/// An app with a food place 8 m away and one autonomous person at the origin.
fn app(locomotion: Locomotion, timeout: Option<f32>) -> (App, Entity, Entity) {
    let mut settings = SocietySettings {
        locomotion,
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = timeout;
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
    let tavern = app
        .world_mut()
        .spawn((
            Name::new("tavern"),
            Place { capacity: 8 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::from_xyz(8.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(home), Autonomous));
    (app, person, tavern)
}

fn translation(app: &App, person: Entity) -> Vec3 {
    app.world()
        .get::<Transform>(person)
        .map(|t| t.translation)
        .unwrap_or_default()
}

fn doing(app: &App, person: Entity) -> Option<Doing> {
    app.world().get::<Doing>(person).cloned()
}

fn ended(app: &mut App) -> Vec<ActivityEnded> {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
        .drain()
        .collect()
}

/// **Criterion 1.** Under `Host`, no crate system touches a travelling person's `Transform`.
#[test]
fn no_crate_system_moves_a_person_under_host() {
    let (mut app, person, _) = app(Locomotion::Host, Some(2.0));
    // Run long enough for a decision to be made and a `Destination` to be written.
    for _ in 0..600 {
        app.update();
    }
    let at = translation(&app, person);
    assert_eq!(
        at,
        Vec3::ZERO,
        "a Host-locomotion person must not be moved by the crate, ended at {at:?}"
    );
    // And it really was asked to walk somewhere, or the test proves nothing.
    assert!(
        app.world().get::<Destination>(person).is_some(),
        "the decision should have written a Destination to walk to"
    );
}

/// **Criterion 2.** With `Host`, a host that moves the person onto the place gets the arrival, the
/// `arrived_day`, and the activity's `satisfies` paid.
#[test]
fn a_host_that_moves_the_person_gets_the_arrival() {
    let (mut app, person, tavern) = app(Locomotion::Host, None);
    for _ in 0..40 {
        app.update();
    }
    let Some(activity) = doing(&app, person) else {
        panic!("the person should have chosen an activity by now");
    };
    assert_eq!(
        activity.target,
        DoingTarget::Place(tavern),
        "it should be walking to the tavern"
    );
    assert!(activity.arrived_day.is_none(), "and not have arrived yet");

    // The host teleports the figure onto the place, which is what its own walker would eventually do.
    // `perceive` reads `GlobalTransform`, and Bevy propagates it from `Transform` between updates, so
    // the arrival is seen on the following tick rather than in the same `update()`.
    app.world_mut()
        .entity_mut(person)
        .insert(Transform::from_xyz(7.5, 0.0, 0.0));
    app.update();
    app.update();

    let arrived = doing(&app, person).expect("still doing the activity");
    assert!(
        arrived.arrived_day.is_some(),
        "moving the person onto the place must register the arrival"
    );
    assert!(
        app.world().get::<Destination>(person).is_none(),
        "arrival removes the Destination"
    );
    // The need is paid from the arrival on, so a few ticks later it must have risen.
    let before = app
        .world()
        .get::<bevy_society::Needs>(person)
        .map(|n| n.level(0))
        .unwrap_or(0.0);
    for _ in 0..40 {
        app.update();
    }
    let after = app
        .world()
        .get::<bevy_society::Needs>(person)
        .map(|n| n.level(0))
        .unwrap_or(0.0);
    assert!(
        after > before,
        "an arrived activity pays satisfies: food {before:.3} -> {after:.3}"
    );
}

/// **Criterion 3.** `TravelFailed` ends the running activity in that tick, `completed: false`.
#[test]
fn travel_failed_ends_the_activity_at_once() {
    let (mut app, person, _) = app(Locomotion::Host, None);
    for _ in 0..40 {
        app.update();
    }
    let activity = doing(&app, person).expect("an activity is running");
    // Clear anything the walk itself already ended.
    let _ = ended(&mut app);

    app.world_mut().write_message(TravelFailed { person });
    app.update();

    let endings = ended(&mut app);
    assert_eq!(
        endings.len(),
        1,
        "exactly one activity should end, got {endings:?}"
    );
    assert_eq!(endings[0].person, person);
    let expected = activity_name(&app, &activity);
    assert_eq!(endings[0].activity, expected, "the running activity ends");
    assert!(
        !endings[0].completed,
        "a failed walk is not a completed activity"
    );
    // The person re-decides in the same tick, so whatever is running now was started *this* tick —
    // it may legitimately be the same activity chosen afresh, which is not the old one continuing.
    if let Some(next) = doing(&app, person) {
        let now = app
            .world()
            .resource::<bevy_society::SocietyClock>()
            .now_days;
        assert!(
            (next.started_day - now).abs() < 1.0e-9,
            "a re-decision starts its activity now: started_day {} against clock {now}",
            next.started_day
        );
        assert!(next.arrived_day.is_none(), "and it has not arrived");
    }
    let _ = &activity;
    // A `Destination` may already be back: the person re-decided in this same tick, and a new
    // decision writes a new one. What must be gone is the *old* walk, which is what the ending above
    // proves; the new Destination is the point of re-deciding.
}

fn activity_name(app: &App, doing: &Doing) -> String {
    app.world()
        .resource::<SocietyDefs>()
        .activity_name(doing.activity)
        .to_owned()
}

/// A `TravelFailed` about a person with no activity, or one who does not exist, is ignored rather than
/// panicking: the host reports what it sees, and a despawn mid-walk is ordinary.
#[test]
fn a_travel_failed_for_nobody_is_ignored() {
    let (mut app, person, _) = app(Locomotion::Host, None);
    for _ in 0..40 {
        app.update();
    }
    let _ = ended(&mut app);
    // An entity that was never a person.
    let stranger = app.world_mut().spawn_empty().id();
    app.world_mut()
        .write_message(TravelFailed { person: stranger });
    app.world_mut().write_message(TravelFailed { person });
    app.update();
    // The real failure still landed; the stranger's did not add a second ending.
    let endings = ended(&mut app);
    assert_eq!(
        endings.len(),
        1,
        "only the real person's failure ends an activity: {endings:?}"
    );
}

/// **Criterion 4.** With `travel_timeout_hours: None`, an unreached walk never times out — ten game
/// days of it.
#[test]
fn no_timeout_keeps_the_walk_alive_for_ten_days() {
    // 4 s per game-day at 1/64 s a tick is 256 ticks a day, so ten days is 2 560 ticks.
    let mut settings = SocietySettings {
        locomotion: Locomotion::Host,
        tick_secs: 1.0 / 64.0,
        seconds_per_day: 4.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = None;
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
        Transform::from_xyz(400.0, 0.0, 0.0),
    ));
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(home), Autonomous));

    // Pick a moment after the first decision and hold the person still from there.
    for _ in 0..40 {
        app.update();
    }
    let walking = doing(&app, person).expect("an activity started");
    assert!(walking.arrived_day.is_none());
    let walking_name = activity_name(&app, &walking);
    let walking_activity = walking.activity;

    let mut ended_while_walking = 0;
    for _ in 0..2_560 {
        app.update();
        for ending in ended(&mut app) {
            // A walk that never arrives and never times out may still be replaced by a *different*
            // decision (the duration clock only starts on arrival), so count only endings of this
            // exact activity.
            if ending.person == person && ending.activity == walking_name && !ending.completed {
                ended_while_walking += 1;
            }
        }
    }
    assert!(
        ended_while_walking <= 1,
        "a walk with no timeout must not keep failing on the way: {} endings in ten game-days",
        ended_while_walking
    );
    // The core claim: nothing timed it out, so either it is still the same activity or the society
    // chose something else — never a timeout ending.
    if let Some(still) = doing(&app, person)
        && still.activity == walking_activity
    {
        assert!(
            still.arrived_day.is_none(),
            "it can never have arrived, having never moved"
        );
    }
}

/// **Criterion 5 (the existing behaviour).** `Builtin` walks the person, as it always did.
#[test]
fn builtin_still_walks_and_arrives() {
    let (mut app, person, _) = app(Locomotion::Builtin, Some(2.0));
    let mut moved = 0.0_f32;
    for _ in 0..700 {
        app.update();
        moved = moved.max(translation(&app, person).x);
    }
    assert!(
        moved > 0.5,
        "Builtin must walk the person toward the tavern: furthest x was {moved:.3} m"
    );
}

/// A `Person` target keeps moving, so `Destination` must be rewritten every tick — otherwise a host
/// is asked to walk to where the target was when the activity started.
#[test]
fn a_person_targets_destination_follows_it() {
    let mut settings = SocietySettings {
        locomotion: Locomotion::Host,
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = None;
    let mut app = headless(defs(), settings);

    let walker = spawn_person(&mut app, "Bram", Vec2::new(0.0, 0.0));
    let target = spawn_person(&mut app, "Ada", Vec2::new(6.0, 0.0));
    // **Not `Autonomous` on the walker**, so the forced `chat` below is not replaced by a decision and
    // the chase is the only thing being observed.
    app.world_mut().entity_mut(target).insert(Autonomous);

    // Two updates first, so the observer that assigns `PersonId` and fills `PersonIndex` has run:
    // `arrive_and_satisfy` iterates `PersonIndex.sorted`, so a person not yet in it is invisible.
    app.update();
    app.update();

    // Pin Bram onto a `chat`, so the chase is the only thing being observed.
    let chat = activity_id(&app, "chat");
    let now = app
        .world()
        .resource::<bevy_society::SocietyClock>()
        .now_days;
    app.world_mut().entity_mut(walker).insert(Doing {
        activity: chat,
        target: DoingTarget::Person(target),
        started_day: now,
        arrived_day: None,
    });
    app.update();
    let first = app
        .world()
        .get::<Destination>(walker)
        .copied()
        .expect("a Person chase writes a Destination");

    // Move the target a long way, and the Destination must follow on the next tick.
    app.world_mut()
        .entity_mut(target)
        .insert(Transform::from_xyz(-9.0, 0.0, 0.0));
    // Two updates: `GlobalTransform` is propagated from `Transform` between them.
    app.update();
    app.update();
    let followed = app
        .world()
        .get::<Destination>(walker)
        .copied()
        .expect("the chase is still running");
    assert_ne!(
        (first.0.x, first.0.y),
        (followed.0.x, followed.0.y),
        "a Person target moved and Destination stayed at the old point: {first:?} -> {followed:?}"
    );
    assert!(
        followed.0.x < 0.0,
        "the Destination should be near the target's new side, got {followed:?}"
    );
}

fn activity_id(app: &App, name: &str) -> bevy_society::ActivityId {
    app.world()
        .resource::<SocietyDefs>()
        .activity_id(name)
        .expect("the activity is defined")
}
