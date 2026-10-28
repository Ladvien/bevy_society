//! **A full place is still a place.**
//!
//! Target expansion used to filter on `used < capacity`, so a place with no free seat produced **no candidate**
//! and its activity became impossible. That is not a small loss: measured on the project's own village, a 30-seat
//! village given 48 people took `work` from **10.3% to 3.6%** of a life and rival density from **7.8% to 20.1%** of
//! directed pairs — because the hostile events that always happen landed on a contact graph made dense by
//! unemployment.
//!
//! The failure has a name in the crowd-simulation literature. Pentheny, *Game AI Pro 2* ch. 17 (corpus
//! `gameaipro2-ch17-advanced-techniques-robust-efficient-crowds`):
//!
//! > "As crowd density increases and the agents' goals converge, this approach falls apart. In situations like
//! > this, agents fight the rest of the crowd to follow their ideal path as closely as possible, **seemingly
//! > ignoring all other possible routes**."
//!
//! and the repair is a **cost**, not a deletion: "agents ... appear to be more aware of their surroundings,
//! **reconsidering their current path choice if a less-congested alternative exists**". The crate already has the
//! ingredients for a host to price a crowd itself — `capacity` is a property of the place and `Place(tag)` picks
//! the nearest — so the defect was the deletion, and this file pins its removal.
//!
//! `capacity: 0` stays a prohibition: a place that seats nobody is closed, not full.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    ActivityStarted, Autonomous, BoundPlaces, Home, Person, Place, PlaceTags, SocietyDefs,
};
use common::{headless, settings, spawn_person};

/// One activity that goes to a place tagged `work`, plus an idle option so a person always has something to do.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 4.0), (id: "rest", decay_per_day: 4.0)],
            activities: [
                (
                    id: "tend",
                    target: Place("work"),
                    // **Long enough to occupy the place.** At this fixture's 1-second game-day a 0.5-hour
                    // activity lasts about one tick, so two villagers never overlap there and the old capacity
                    // veto never fired — the first version of these tests could not tell the two rules apart.
                    duration_hours: 6.0,
                    considerations: [(input: Need("social"), curve: Linear(0.5, 0.0))],
                    satisfies: [("social", 0.5)],
                ),
                (
                    id: "wait",
                    target: Itself,
                    duration_hours: 1.0,
                    considerations: [(input: Need("rest"), curve: Linear(0.5, 0.0))],
                    satisfies: [("rest", 0.5)],
                ),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

fn one_second_day() -> bevy_society::SocietySettings {
    let mut settings = settings();
    settings.seconds_per_day = 1.0;
    settings
}

fn place_at(app: &mut App, name: &str, tag: &str, capacity: u32) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            Place { capacity },
            PlaceTags(vec![tag.to_owned()]),
            Transform::default(),
        ))
        .id()
}

/// Villagers spread around the place so they are inside each other's perception.
fn villagers(app: &mut App, place: Entity, count: usize) {
    for i in 0..count {
        let angle = i as f32 * std::f32::consts::TAU / count.max(1) as f32;
        let person = spawn_person(
            app,
            &format!("P{i}"),
            Vec2::new(0.3 * angle.cos(), 0.3 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person, Home(place)));
    }
}

/// How many times `tend` was started, over `ticks`, per villager.
fn tend_per_person(app: &mut App, ticks: usize, people: usize) -> f32 {
    let mut starts = 0;
    for _ in 0..ticks {
        app.update();
        for event in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
            .drain()
        {
            if event.activity == "tend" {
                starts += 1;
            }
        }
    }
    starts as f32 / people.max(1) as f32
}

/// **A place that seats nobody is closed.** The one prohibition that survives: no arrangement of people makes
/// room, so a `capacity: 0` place must never be a candidate.
#[test]
fn a_place_that_seats_nobody_is_never_chosen() {
    let mut app = headless(defs(), one_second_day());
    let shut = place_at(&mut app, "shut", "work", 0);
    villagers(&mut app, shut, 2);
    assert_eq!(
        tend_per_person(&mut app, 300, 2),
        0.0,
        "a capacity-0 place can never take anyone"
    );
}

/// **A full place is worked anyway.** One seat, four villagers: under the old rule the activity had no target at
/// all and nobody worked; now they take turns. This is the whole change.
#[test]
fn a_full_place_is_still_worked() {
    let mut app = headless(defs(), one_second_day());
    let tight = place_at(&mut app, "tight", "work", 1);
    villagers(&mut app, tight, 4);
    let rate = tend_per_person(&mut app, 400, 4);
    assert!(
        rate > 0.0,
        "four villagers and one seat must still work it: {rate} starts a villager"
    );
}

/// **A bound place at capacity is still taken.** `Bound` had the same veto and gets the same treatment.
#[test]
fn a_bound_place_at_capacity_is_still_taken() {
    let mut app = headless(defs(), one_second_day());
    let tight = place_at(&mut app, "tight", "work", 1);
    villagers(&mut app, tight, 2);
    for i in 0..2 {
        let person = app
            .world_mut()
            .query::<(Entity, &Name)>()
            .iter(app.world())
            .find(|(_, name)| name.as_str() == format!("P{i}"))
            .map(|(entity, _)| entity);
        if let Some(person) = person {
            app.world_mut()
                .entity_mut(person)
                .insert(BoundPlaces(vec![("work".to_owned(), tight)]));
        }
    }
    assert!(
        tend_per_person(&mut app, 300, 2) > 0.0,
        "a bound place with no free seat must still be worked"
    );
}

/// **A bound place that seats nobody is still closed**, which is the invariant `tests/bound_places.rs` pins for a
/// capacity-0 binding.
#[test]
fn a_bound_place_that_seats_nobody_is_closed() {
    let mut app = headless(defs(), one_second_day());
    let shut = place_at(&mut app, "shut", "work", 0);
    villagers(&mut app, shut, 2);
    for i in 0..2 {
        let person = app
            .world_mut()
            .query::<(Entity, &Name)>()
            .iter(app.world())
            .find(|(_, name)| name.as_str() == format!("P{i}"))
            .map(|(entity, _)| entity);
        if let Some(person) = person {
            app.world_mut()
                .entity_mut(person)
                .insert(BoundPlaces(vec![("work".to_owned(), shut)]));
        }
    }
    assert_eq!(
        tend_per_person(&mut app, 300, 2),
        0.0,
        "a bound place that seats nobody can never take anyone"
    );
}
