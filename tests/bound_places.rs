//! **Per-person bound places** (issue #5).
//!
//! Sanctuary assigns each townsperson their own work site each morning — the woodlot, the quarry, a
//! stretch of wall — and daily life must send them to *their* site, not the nearest site of that kind.
//! `ActivityTarget::Place(tag)` is the same place for everyone; `Home` is one place per person; a
//! `BoundPlaces` binding generalises `Home` to any number of named places.
//!
//! Issue's own wording asked for a warning on a *missing* binding too. That is deliberately not done:
//! a host binds some people and not others (a night watch has no work place), so a missing binding is
//! the ordinary case and warning about it would be one warning per person for nothing. A binding to
//! something that is not a `Place` **is** warned about, once per person and name.

mod common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, BoundPlaces, Doing, DoingTarget, Place, PlaceTags, SocietyDefs, SocietySettings,
};
use common::capture::{LogCapture, log_plugin};
use common::{headless, spawn_person};

/// Two activities, both targeting `Bound("work")`, so a bound person walks to their own place.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "duty", decay_per_day: 1.0)],
            activities: [(
                id: "tend",
                target: Bound("work"),
                duration_hours: 180.0,
                considerations: [(input: Need("duty"), curve: Linear(-1.0, 1.0))],
                satisfies: [("duty", 0.5)],
            )],
        )"#,
    )
    .expect("the definitions parse")
}

fn settings() -> SocietySettings {
    let mut settings = SocietySettings {
        tick_secs: 1.0 / 64.0,
        // One game-day per 64 ticks, so an activity is over quickly and the next decision comes.
        seconds_per_day: 1.0,
        ..SocietySettings::default()
    };
    // These tests are about which place a decision picks, not about walking, and a one-second game-day
    // makes the crate's own walk warnings fire on every app. A walk that never times out is also what a
    // host under `Locomotion::Host` uses, and the activity below is long enough that the "walk outlasts
    // the activity" warning has nothing to report.
    settings.tuning.travel_timeout_hours = None;
    settings
}

/// Spawns a place at `at` and returns it.
fn place(app: &mut App, name: &str, at: Vec3, tags: &[&str]) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            Place { capacity: 4 },
            PlaceTags(tags.iter().map(|t| (*t).to_owned()).collect()),
            Transform::from_translation(at),
        ))
        .id()
}

/// **Criterion 1.** Two persons with different `"work"` bindings reach different places.
#[test]
fn two_bindings_send_two_people_to_two_places() {
    let mut app = headless(defs(), settings());
    let woodlot = place(&mut app, "woodlot", Vec3::new(6.0, 0.0, 0.0), &["work"]);
    let quarry = place(&mut app, "quarry", Vec3::new(-6.0, 0.0, 0.0), &["work"]);

    let a = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let b = spawn_person(&mut app, "Ada", Vec2::ZERO);
    app.world_mut()
        .entity_mut(a)
        .insert((Autonomous, BoundPlaces(vec![("work".to_owned(), woodlot)])));
    app.world_mut()
        .entity_mut(b)
        .insert((Autonomous, BoundPlaces(vec![("work".to_owned(), quarry)])));

    // Run until both have chosen, and collect the place each one's `Doing` targets.
    let mut a_target = None;
    let mut b_target = None;
    for _ in 0..120 {
        app.update();
        if a_target.is_none()
            && let Some(doing) = app.world().get::<Doing>(a)
        {
            a_target = Some(doing.target);
        }
        if b_target.is_none()
            && let Some(doing) = app.world().get::<Doing>(b)
        {
            b_target = Some(doing.target);
        }
        if a_target.is_some() && b_target.is_some() {
            break;
        }
    }

    assert_eq!(
        a_target,
        Some(DoingTarget::Place(woodlot)),
        "Bram is bound to the woodlot"
    );
    assert_eq!(
        b_target,
        Some(DoingTarget::Place(quarry)),
        "Ada is bound to the quarry"
    );
}

/// **Criterion 2.** A person with no `"work"` binding never starts the activity.
#[test]
fn an_unbound_person_never_starts_the_activity() {
    let mut app = headless(defs(), settings());
    // The place exists and has room; only the binding is missing.
    let woodlot = place(&mut app, "woodlot", Vec3::new(3.0, 0.0, 0.0), &["work"]);

    let bound = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let unbound = spawn_person(&mut app, "Ada", Vec2::ZERO);
    app.world_mut()
        .entity_mut(bound)
        .insert((Autonomous, BoundPlaces(vec![("work".to_owned(), woodlot)])));
    app.world_mut().entity_mut(unbound).insert(Autonomous);

    let mut bound_started = 0;
    let mut unbound_started = 0;
    for _ in 0..400 {
        app.update();
        for started in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<bevy_society::ActivityStarted>>()
            .drain()
        {
            if started.person == bound && started.activity == "tend" {
                bound_started += 1;
            }
            if started.person == unbound && started.activity == "tend" {
                unbound_started += 1;
            }
        }
    }
    assert!(bound_started > 0, "the bound person should work");
    assert_eq!(
        unbound_started, 0,
        "a person with no \"work\" binding must never start a Bound(\"work\") activity"
    );
}

/// **Criterion 3**, and the no-warning case.
///
/// One app for both: the `tracing` subscriber is **process-global and settable once**, so a test binary
/// may build only one capturing app. Two people, one with a binding to something that is not a place and
/// one with no binding at all.
#[test]
fn a_non_place_binding_warns_once_and_an_unbound_person_is_silent() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        log_plugin(),
    ))
    .insert_resource(settings())
    .insert_resource(defs());
    let plugins = bevy_society::SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();

    let bound = spawn_person(&mut app, "Bram", Vec2::ZERO);
    // An entity that is NOT a `Place`.
    let stranger = app.world_mut().spawn(Name::new("not-a-place")).id();
    app.world_mut()
        .entity_mut(bound)
        .insert((Autonomous, BoundPlaces(vec![("work".to_owned(), stranger)])));

    let unbound = spawn_person(&mut app, "Ada", Vec2::ZERO);
    // Autonomous with no `BoundPlaces` at all: the ordinary case for a host that binds only some.
    app.world_mut().entity_mut(unbound).insert(Autonomous);

    // **Enough updates for several society ticks.** A tick is many fixed steps apart (the crate counts
    // one per `tick_secs`), and the warning is written by `decide_activities`, which only runs on a
    // tick. 100 updates at this app's settings is one tick, which is why the count has to be driven by
    // the clock rather than guessed.
    let start = app.world().resource::<bevy_society::SocietyClock>().tick;
    let mut updates = 0;
    while app.world().resource::<bevy_society::SocietyClock>().tick < start + 3 && updates < 4_000 {
        app.update();
        updates += 1;
    }
    assert!(
        app.world().resource::<bevy_society::SocietyClock>().tick >= start + 3,
        "the society must actually tick for the decision to run"
    );

    let captured = app.world().resource::<LogCapture>().drain();
    let misplaced: Vec<String> = captured
        .iter()
        .filter(|event| event.level == "WARN" && event.message.contains("which is not a place"))
        .map(|event| event.message.clone())
        .collect();
    assert_eq!(
        misplaced.len(),
        1,
        "a binding to a non-place should warn once, warned {} time(s) over {updates} updates: {misplaced:?}",
        misplaced.len()
    );
    assert!(
        misplaced[0].contains("\"work\""),
        "the warning names the binding: {}",
        misplaced[0]
    );
    // And the unbound person produced nothing: a missing binding is not a mistake.
    // Any warning that names the *unbound* person's id would mean a missing binding was reported.
    let unbound_id = app
        .world()
        .get::<bevy_society::PersonId>(unbound)
        .map(|id| id.0)
        .expect("a person has an id");
    let about_unbound: Vec<&String> = captured
        .iter()
        .filter(|event| {
            event.level == "WARN"
                && event.message.starts_with("bevy_society: person ")
                && event
                    .message
                    .split_whitespace()
                    .nth(1)
                    .is_some_and(|word| word == unbound_id.to_string())
        })
        .map(|event| &event.message)
        .collect();
    assert!(
        about_unbound.is_empty(),
        "an unbound person is not a mistake, warned: {about_unbound:?}"
    );
}

/// An empty binding name is refused at load time, because it can never match anything a host writes.
#[test]
fn an_empty_binding_name_is_refused() {
    let error = match SocietyDefs::from_ron(
        r#"(
            needs: [(id: "duty", decay_per_day: 1.0)],
            activities: [(
                id: "tend",
                target: Bound(""),
                duration_hours: 1.0,
                considerations: [(input: Need("duty"), curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    ) {
        Ok(_) => panic!("an empty Bound name must be refused"),
        Err(error) => error,
    };
    let text = format!("{error}");
    assert!(
        text.contains("Bound target needs a name"),
        "the error should name the problem, got: {text}"
    );
}

/// `Distance` is a valid consideration for a `Bound` target, as it is for `Place` and `Home`.
#[test]
fn distance_is_valid_for_a_bound_target() {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "duty", decay_per_day: 1.0)],
            activities: [(
                id: "tend",
                target: Bound("work"),
                duration_hours: 1.0,
                considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    )
    .expect("Distance is valid for a Bound target");
}

/// `bind` replaces an existing name and `get` reads it back.
#[test]
fn bind_replaces_and_get_reads() {
    let mut app = headless(defs(), settings());
    let first = place(&mut app, "first", Vec3::ZERO, &[]);
    let second = place(&mut app, "second", Vec3::ZERO, &[]);

    let mut bound = BoundPlaces::default();
    assert_eq!(bound.get("work"), None);
    bound.bind("work", first);
    assert_eq!(bound.get("work"), Some(first));
    bound.bind("work", second);
    assert_eq!(
        bound.get("work"),
        Some(second),
        "bind replaces rather than appends"
    );
    assert_eq!(bound.0.len(), 1, "and leaves one entry");
}

/// A bound place gets no candidate when it is at capacity, exactly as `Place(tag)` does.
///
/// Asserted on the decision rather than on positions: two people may be *sent* to a capacity-1 place in
/// the same tick, because `Place(tag)` behaves the same way — a place's `used` count is recomputed each
/// tick from who is already there, and it cannot see a choice made in the same tick.
#[test]
fn a_full_bound_place_gives_no_candidate() {
    let mut app = headless(defs(), settings());
    let tight = app
        .world_mut()
        .spawn((
            Name::new("tight"),
            Place { capacity: 0 },
            PlaceTags(vec!["work".to_owned()]),
            Transform::from_xyz(2.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Autonomous, BoundPlaces(vec![("work".to_owned(), tight)])));

    let mut started = 0;
    for _ in 0..200 {
        app.update();
        for event in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<bevy_society::ActivityStarted>>()
            .drain()
        {
            if event.person == person && event.activity == "tend" {
                started += 1;
            }
        }
    }
    assert_eq!(
        started, 0,
        "a capacity-0 place can never take anyone, and `Bound` must respect that like `Place(tag)`"
    );
}
