#![cfg(feature = "fields")]
//! Social fields (plan D9): three millers standing together leave a presence field whose gradient
//! points toward them, and at a cell they have reached they own the whole share.

use crate::common;

use bevy::math::{IVec2, Vec2};
use bevy_society::{FieldInput, SocialFields, SocietyClock, SocietyDefs};

const FACTIONS: &str = "(factions: [(id: \"millers\")])";

#[test]
fn millers_gradient_points_toward_their_cluster() {
    let defs = SocietyDefs::from_ron(FACTIONS).expect("the millers definition parses");
    let mut app = common::headless(defs, common::settings());
    for name in ["Bram", "Ada", "Cole"] {
        let memberships = app
            .world()
            .resource::<SocietyDefs>()
            .memberships(["millers"])
            .expect("millers is defined");
        let entity = common::spawn_person(&mut app, name, Vec2::new(8.0, 0.0));
        app.world_mut().entity_mut(entity).insert(memberships);
    }
    let millers = app
        .world()
        .resource::<SocietyDefs>()
        .faction_id("millers")
        .expect("millers is defined");

    let start = app.world().resource::<SocietyClock>().now_days;
    while app.world().resource::<SocietyClock>().now_days < start + 0.5 {
        app.update();
    }

    let fields = app.world().resource::<SocialFields>();
    // The millers stand at (8, 0), which is cell (40, 32). The probe (4.5, 0.5) is cell (36, 32),
    // four cells west of them.
    let probe = Vec2::new(4.5, 0.5);
    let cell = fields.world_to_cell(probe);
    assert_eq!(cell, IVec2::new(36, 32), "probe lands in the expected cell");

    let own = [millers];
    let share = fields.field_value(FieldInput::OwnShare, &own, cell);
    assert!(
        share > 0.5,
        "OwnShare at the probe is {share}; the millers' presence should own it"
    );

    let target = fields
        .gradient_target(FieldInput::OwnFaction, &own, probe, true)
        .expect("the millers' field has a gradient at the probe");
    assert!(
        target.x > probe.x,
        "uphill target {target} should lie east of the probe {probe}"
    );
}
