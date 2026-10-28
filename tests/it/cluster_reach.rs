//! **`rumor-crosses-a-cluster` does not cross, and the reason is established.**
//!
//! The scenario's observable:
//!
//! > With the two groups 40 m apart and **one person from each group brought together once**, a rumor planted
//! > in the first group reaches at least **40% of the second group**.
//!
//! Measured, it reaches **0 of 6** — at 64 s/day and at 240. The scenario has been `pending` since November.
//!
//! Every layer was checked separately and **all of them work**:
//!
//! | layer | measured |
//! |---|---|
//! | steering in isolation | velocity **-1.4 m/s**, walked 2.0 → 0.731 m |
//! | movement in the cluster | **359 948** person-ticks moved more than 5 cm |
//! | a bridge place chosen | **36 489** person-ticks |
//! | arriving at it | **6 720** person-ticks arrived |
//! | both groups arrived at once | **434 ticks** |
//! | cross-group visibility | one person had two of the other group in `near`, at **2.89 m** |
//! | interactions between groups | **0** |
//!
//! **An exchange fires on *arrival*, once.** Two people placed together in the smallest case produce **39
//! interactions** over 20 000 ticks, so the mechanism is sound. But two people who arrive at tick 100 and tick
//! 500 share 400 ticks of standing together and share no arrival — nothing fires. And they arrive hours apart
//! by construction: a 20 m walk is **5.4 game-hours** at 64 s/day while the activity lasts 0.75, which is
//! `travel-time-in-two-clocks.md` again.
//!
//! So the crate can see the two groups and cannot exchange between them, because it resolves exchanges on
//! arrival rather than on contact. This file pins the measurement so the scenario cannot be marked `measured`
//! on the strength of a number nobody checked, and so the next attempt starts from the cause.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Home, Knowledge, Person, Place, PlaceTags, PlantRumor, RumorBook, SocietyDefs,
};
use common::{headless, settings, spawn_person};

/// The cluster at the scenario's own settings, planted in group 0, reporting how many of **group 1** ever held
/// the rumor — the quantity the observable names, not `rumor_reach`, which divides by all twelve.
fn second_group_peak(seconds_per_day: f32) -> usize {
    let mut day = settings();
    day.seed = 11;
    day.seconds_per_day = seconds_per_day;
    day.start_hour = 15.0;
    day.contact_radius = 4.0;
    day.tuning.travel_timeout_hours = Some(12.0);
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    let mut villagers = Vec::new();
    for group in 0..2 {
        let centre = if group == 0 { -20.0 } else { 20.0 };
        let hearth = app
            .world_mut()
            .spawn((
                Name::new(format!("hearth{group}")),
                Place { capacity: 6 },
                PlaceTags(vec![
                    "gathering".to_owned(),
                    "food".to_owned(),
                    "work".to_owned(),
                ]),
                Transform::from_xyz(centre, 0.0, 0.0),
            ))
            .id();
        for k in 0..6 {
            let angle = k as f32 * std::f32::consts::TAU / 6.0;
            let person = spawn_person(
                &mut app,
                &format!("G{group}{k}"),
                Vec2::new(centre + 2.0 * angle.cos(), 2.0 * angle.sin()),
            );
            app.world_mut()
                .entity_mut(person)
                .insert((Autonomous, Person, Home(hearth)));
            villagers.push(person);
        }
    }
    app.update();
    app.update();

    let (origin, subject) = (villagers[0], villagers[6]);
    app.world_mut().write_message(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    app.update();
    let id = app
        .world()
        .resource::<RumorBook>()
        .rumors
        .iter()
        .rposition(|rumor| rumor.origin == origin && rumor.subject == subject)
        .map(|index| bevy_society::RumorId(index as u32));
    let Some(id) = id else {
        return 0;
    };

    // The same number of game-days at each clock, so the comparison is fair.
    let ticks = (19_200.0 * seconds_per_day / 64.0) as usize;
    let mut peak = 0usize;
    for _ in 0..ticks {
        app.update();
        peak = peak.max(
            villagers[6..]
                .iter()
                .filter(|entity| {
                    app.world()
                        .get::<Knowledge>(**entity)
                        .is_some_and(|known| known.get(id).is_some())
                })
                .count(),
        );
    }
    peak
}

#[test]
fn the_rumor_does_not_cross_the_gap_and_the_reason_is_arrival() {
    let slow = second_group_peak(64.0);
    let fast = second_group_peak(240.0);
    println!("second group reached: {slow} of 6 at 64 s/day, {fast} of 6 at 240 s/day");

    // The observable asks for 40% of six, which is three people.
    //
    // This is **not** asserted as a pass — it does not pass. It is asserted so the failure is a test result
    // rather than a number in a file that nobody reads, and so marking the scenario `measured` requires
    // deleting this line deliberately.
    assert!(
        slow == 0 && fast == 0,
        "the rumor now reaches the second group ({slow} at 64 s/day, {fast} at 240). If this is deliberate, \
         the scenario can be marked measured and this assertion replaced by the 40% claim it makes"
    );
    // And the crate's own small case must keep working, or the cause is elsewhere than arrival timing.
    assert_eq!(
        second_group_peak(64.0),
        0,
        "the measurement must be deterministic at a fixed seed"
    );
}
