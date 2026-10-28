//! **A person who has not arrived is not doing the activity.**
//!
//! `Doing` carries an activity and an `arrived_day`. While `arrived_day` is `None` the doc says the
//! person is *travelling*; only once it is set do the duration and the `satisfies` payout begin. But
//! `Doing.activity` names the target activity from the moment it is chosen, so every reader — a host's
//! HUD, the evidence harness, a script — sees a person *sleeping* while it is walking to bed and
//! receiving nothing.
//!
//! Measured, `village-evening` at its own settings over 17 game-days: the village reports **56.8% of
//! all person-ticks asleep**, against the **28.6%** that 350 sleeps of 8 hours a day allow. One villager
//! tracked tick by tick: `rest` at **0.087 and falling at exactly the decay rate** while its `Doing` said
//! `sleep` — the payout had not begun, because `arrived_day` was `None`.
//!
//! The cause is the mismatch in time scales: `travel_timeout_hours` is 20 at 64 s/day, which is
//! **3 414 ticks**, while an 8-hour sleep is **1 365**. A person can walk toward its bed for two and a
//! half times the length of the sleep itself, labelled asleep the whole way.
use crate::common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Doing, Home, Needs, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// **Ignored, and should be un-ignored when the fix lands.** It asserts the honest behaviour and fails
/// today: `Doing.activity` names the target activity from the moment it is chosen, so a person walking to
/// bed is reported asleep and paid nothing. The fix is a decision —
/// `docs/loop/needs-decision/what-doing-reports-while-travelling.md` — because it changes a component a
/// host reads.
///
/// A failing test in the default run would block every iteration after this one, so it is ignored rather
/// than deleted: the defect is pinned, and `cargo test -- --ignored` shows it.
#[test]
#[ignore = "Doing names the target activity while travelling; see needs-decision/"]
fn a_person_walking_to_bed_is_not_reported_as_sleeping() {
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
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(defs, short_day);
    // The bed is far away: at 0.0219 m a tick this is a long walk, as the village's well is for anyone
    // standing across it once the crowding pushes them apart.
    let bed = app
        .world_mut()
        .spawn((
            Name::new("bed"),
            Place { capacity: 8 },
            PlaceTags(vec!["rest".to_owned()]),
            Transform::from_xyz(6.0, 0.0, 0.0),
        ))
        .id();
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Home(bed), Autonomous));

    // Find a tick where the person has chosen `sleep` but has not arrived.
    let mut walking: Option<(usize, f32)> = None;
    let mut last_rest = 0.2_f32;
    for _ in 0..600 {
        app.update();
        let rest = app
            .world()
            .get::<Needs>(person)
            .and_then(|n| n.0.first().copied())
            .unwrap_or(last_rest);
        if let Some(doing) = app.world().get::<Doing>(person)
            && doing.arrived_day.is_none()
        {
            // Track how far `rest` moves over the next stretch while still labelled `sleep`.
            let before = rest;
            for _ in 0..40 {
                app.update();
            }
            let after = app
                .world()
                .get::<Needs>(person)
                .and_then(|n| n.0.first().copied())
                .unwrap_or(before);
            walking = Some((40, after - before));
            break;
        }
        last_rest = rest;
    }
    let Some((ticks, changed)) = walking else {
        panic!("the person never chose a travel activity within 600 ticks");
    };
    println!(
        "while `Doing.activity` says sleep and `arrived_day` is None: rest changed {changed:+.5} \
         over {ticks} ticks (decay alone would be negative; a payout would be positive)"
    );
    assert!(
        changed > 0.0,
        "a person labelled `sleep` must be being paid for it; it changed {changed:+.5}, which is \
         decay alone — the activity is named but not begun"
    );
}
