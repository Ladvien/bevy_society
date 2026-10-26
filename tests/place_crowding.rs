//! **A full place is a price, not a prohibition.**
//!
//! `Place::capacity` used to *filter* candidates before scoring (`used < capacity`), so a full workshop
//! had no candidate at all and the activity could not be chosen — not "chosen less often", but *not
//! available*. Measured on `village-evening`, whose four places seat 30: `work` fell **10.3% → 3.6%** of
//! person-ticks as the population went 24 → 48, because the workshop (6 seats) was permanently full.
//!
//! Talamali et al. 2019 (DOI `10.1007/s11721-019-00176-9`, Eq. 12) model congestion as a cost that is
//! negligible below the critical number of users and grows exponentially above it, and measure that a
//! crowded source is **still used**. `src/model/crowding.rs` owns the formula; these tests are the
//! end-to-end claim: the activity stays choosable, and the decision sends the next arrival to the emptier
//! place when there is a choice.
mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityStarted, Autonomous, Doing, DoingTarget, Place, PlaceTags, SocietyDefs, SocietySettings,
};
use common::{headless, spawn_person};

/// One activity that goes to a `work` place. Nothing else competes, so the only question a decision can
/// answer is *whether* the activity has a target at all — which is exactly what the old filter destroyed.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.0)],
            activities: [
                (
                    id: "work",
                    target: Place("work"),
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [],
                ),
            ],
        )"#,
    )
    .expect("the crowding definitions parse")
}

/// One game-day per 64 ticks, so the activity does not end inside a test's run.
fn settings() -> SocietySettings {
    let mut settings = common::settings();
    settings.seconds_per_day = 1.0;
    settings
}

fn place(app: &mut App, name: &str, pos: Vec2, capacity: u32) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            Place { capacity },
            PlaceTags(vec!["work".to_owned()]),
            Transform::from_xyz(pos.x, 0.0, pos.y),
        ))
        .id()
}

/// Puts `person` at `place` as if it had already chosen to work there, which is what the occupancy count
/// reads. Inserting `Doing` directly is the honest way to say "somebody is already at the bench" without
/// running enough ticks for a decision to happen.
fn occupy(app: &mut App, person: Entity, place: Entity) {
    app.world_mut().entity_mut(person).insert((
        Autonomous,
        Doing {
            activity: bevy_society::ActivityId(1),
            target: DoingTarget::Place(place),
            started_day: 0.0,
            arrived_day: Some(0.0),
        },
    ));
}

/// The direct regression: **a place at capacity still yields a candidate**.
///
/// Before this change `targets_for` dropped it, the activity had no target, and the person fell to the
/// idle floor. Now the option is priced at `exp(-1) = 0.368` and is still chosen, because it is the only
/// option. A test that asserted "the score fell" would pass with the old filter too (`0` is a fall); what
/// matters is that the activity **starts**.
#[test]
fn a_full_place_is_still_a_candidate() {
    let mut app = headless(defs(), settings());
    let bench = place(&mut app, "bench", Vec2::ZERO, 1);
    let sitting = spawn_person(&mut app, "Alys", Vec2::new(0.4, 0.0));
    occupy(&mut app, sitting, bench);

    let asker = spawn_person(&mut app, "Bram", Vec2::new(0.0, 0.4));
    app.world_mut().entity_mut(asker).insert(Autonomous);

    let mut started = 0;
    for _ in 0..12 {
        app.update();
        for message in app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
            .drain()
        {
            if message.person == asker && message.activity == "work" {
                started += 1;
            }
        }
    }
    assert!(
        started > 0,
        "a place at capacity is a price, not a prohibition: 'work' must still be choosable, and with no \
         other option it must actually be chosen"
    );
    let doing = app
        .world()
        .get::<Doing>(asker)
        .expect("the person chose something");
    assert_eq!(
        doing.target,
        DoingTarget::Place(bench),
        "and it goes to the only place that carries the tag"
    );
}

/// The paper's interior optimum, as an emergent allocation: two workshops of the same size at the same
/// distance, one full and one empty. The decision must send the next worker to the **empty** one.
///
/// This is the claim an all-or-nothing gate cannot make. Under the old filter the full workshop had no
/// candidate and the empty one won by *absence*; here it wins by *score*, and the crowded one remains
/// reachable the moment the empty one fills up.
#[test]
fn the_decision_prefers_the_emptier_of_two_workshops() {
    let mut app = headless(defs(), settings());
    // Equal distance either side, so nothing but occupancy can separate them.
    let full = place(&mut app, "full", Vec2::new(2.0, 0.0), 2);
    let empty = place(&mut app, "empty", Vec2::new(-2.0, 0.0), 2);
    for (i, name) in ["Alys", "Corin"].iter().enumerate() {
        let worker = spawn_person(&mut app, name, Vec2::new(2.0, 0.4 * i as f32));
        occupy(&mut app, worker, full);
    }

    let asker = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(asker).insert(Autonomous);

    for _ in 0..12 {
        app.update();
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
            .drain();
    }
    let doing = app
        .world()
        .get::<Doing>(asker)
        .expect("the person chose an activity");
    assert_eq!(
        doing.target,
        DoingTarget::Place(empty),
        "the full workshop (2/2) must lose to the empty one at the same distance, or occupancy is not a \
         price. The paper's optimum is an interior split, which needs this gradient."
    );
}
