//! Groups and status (plan D6): conferred standing and per-faction statistics.

use crate::common;

use bevy::prelude::*;
use bevy_society::{FactionStats, SocietyClock, SocietyDefs, SocietySettings, Status, Ties};
use common::{headless, run, settings, spawn_person};

/// Settings where one game-day is 64 ticks.
fn day_settings() -> SocietySettings {
    let mut s = settings();
    s.seconds_per_day = 1.0;
    s
}

/// Sets the tie `owner → target`. `last_contact_day` is the current clock, so familiarity equals the peak.
fn set_tie(app: &mut App, owner: Entity, target: Entity, affinity: f32, respect: f32, peak: f32) {
    let now = app.world().resource::<SocietyClock>().now_days as f32;
    let mut ties = app
        .world_mut()
        .get_mut::<Ties>(owner)
        .expect("owner has ties");
    let tie = ties.entry(target);
    tie.affinity = affinity;
    tie.respect = respect;
    tie.familiarity_peak = peak;
    tie.last_contact_day = now;
}

#[test]
fn standing_is_the_mean_respect_of_familiar_persons_only() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    // Far apart (out of contact range), so only the explicit ties matter.
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let b = spawn_person(&mut app, "b", Vec2::new(40.0, 0.0));
    let c = spawn_person(&mut app, "c", Vec2::new(80.0, 0.0));
    let d = spawn_person(&mut app, "d", Vec2::new(120.0, 0.0));

    // Respect from b and c toward a is counted: familiar, respect 0.6 each.
    set_tie(&mut app, b, a, 0.0, 0.6, 1.0);
    set_tie(&mut app, c, a, 0.0, 0.6, 1.0);
    // d holds respect 1.0 toward a but barely knows them: excluded (familiarity < 0.2).
    set_tie(&mut app, d, a, 0.0, 1.0, 0.1);

    run(&mut app, 2);

    let standing = app
        .world()
        .get::<Status>(a)
        .expect("a has a status")
        .standing;
    assert!(
        (standing - 0.6).abs() < 1e-2,
        "standing {standing}, expected 0.6 from b and c only"
    );
}

#[test]
fn a_person_nobody_knows_has_zero_standing() {
    let mut app = headless(SocietyDefs::default(), day_settings());
    let a = spawn_person(&mut app, "a", Vec2::new(0.0, 0.0));
    let _b = spawn_person(&mut app, "b", Vec2::new(40.0, 0.0));

    run(&mut app, 2);

    let standing = app
        .world()
        .get::<Status>(a)
        .expect("a has a status")
        .standing;
    assert_eq!(standing, 0.0);
}

#[test]
fn faction_stats_report_members_cohesion_and_regard_between_factions() {
    let defs = SocietyDefs::from_ron(
        r#"(
            factions: [
                (id: "millers"),
                (id: "weavers"),
            ],
        )"#,
    )
    .expect("factions RON parses");
    let millers = defs.faction_id("millers").expect("millers defined");
    let weavers = defs.faction_id("weavers").expect("weavers defined");
    let miller_set = defs.memberships(["millers"]).expect("memberships");
    let weaver_set = defs.memberships(["weavers"]).expect("memberships");

    let mut app = headless(defs, day_settings());
    let a1 = spawn_person(&mut app, "a1", Vec2::new(0.0, 0.0));
    let a2 = spawn_person(&mut app, "a2", Vec2::new(40.0, 0.0));
    let b1 = spawn_person(&mut app, "b1", Vec2::new(80.0, 0.0));
    app.world_mut().entity_mut(a1).insert(miller_set.clone());
    app.world_mut().entity_mut(a2).insert(miller_set);
    app.world_mut().entity_mut(b1).insert(weaver_set);

    // **Every value here is inside `Tuning::affinity_ceiling` (0.35).** `relax_affinity` runs on each
    // tick and now clamps through `Tie::add_affinity`, so an input above the ceiling would be silently
    // pulled down before the stat is read and the expected means would be wrong. The test is about what
    // `FactionStats` averages, not about the clamp.
    // Within millers: a1→a2 0.3, a2→a1 0.2 (mean 0.25).
    set_tie(&mut app, a1, a2, 0.3, 0.0, 0.0);
    set_tie(&mut app, a2, a1, 0.2, 0.0, 0.0);
    // Millers toward weavers: a1→b1 0.35, a2→b1 0.2 (mean 0.275).
    set_tie(&mut app, a1, b1, 0.35, 0.0, 0.0);
    set_tie(&mut app, a2, b1, 0.2, 0.0, 0.0);
    // Weavers toward millers: b1→a1 0.0, b1→a2 -0.2 (mean -0.1).
    set_tie(&mut app, b1, a1, 0.0, 0.0, 0.0);
    set_tie(&mut app, b1, a2, -0.2, 0.0, 0.0);

    run(&mut app, 2);

    let stats = app.world().resource::<FactionStats>();
    let m = stats.get(millers).expect("millers stats");
    let w = stats.get(weavers).expect("weavers stats");

    assert_eq!(m.members, 2);
    assert_eq!(w.members, 1);
    assert!(
        (m.cohesion - 0.25).abs() < 2e-2,
        "millers cohesion {}",
        m.cohesion
    );
    assert!(
        (m.regard[weavers.0 as usize] - 0.275).abs() < 2e-2,
        "millers→weavers regard {}",
        m.regard[weavers.0 as usize]
    );
    assert!(
        (w.regard[millers.0 as usize] + 0.1).abs() < 1e-2,
        "weavers→millers regard {}",
        w.regard[millers.0 as usize]
    );
}
