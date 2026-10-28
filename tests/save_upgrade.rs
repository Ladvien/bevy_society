//! **An older save is upgraded, not refused** (issue #28).
//!
//! `tests/fixtures/save-v1.json` is a version-1 save of a four-person village: the version-2 capture
//! with `version` set to 1 and every person's `stock` key removed, which is exactly what 0.3.0 wrote.
#![cfg(feature = "serde")]

mod common;

use bevy::prelude::*;
use bevy_society::{
    Person, PersonId, PersonKey, Place, PlaceKey, RestoreError, SocietySave, Stock, VERSION,
    capture, restore,
};

const V1: &str = include_str!("fixtures/save-v1.json");

fn keyed_village() -> App {
    let mut app = common::village::village(common::village::village_defs(), 9, 4);
    let mut persons: Vec<(u32, Entity)> = {
        let mut q = app
            .world_mut()
            .query_filtered::<(Entity, &PersonId), With<Person>>();
        q.iter(app.world()).map(|(e, id)| (id.0, e)).collect()
    };
    persons.sort_by_key(|&(i, _)| i);
    for (i, (_, e)) in persons.iter().enumerate() {
        app.world_mut().entity_mut(*e).insert(PersonKey(i as u64));
    }
    let places: Vec<Entity> = {
        let mut q = app.world_mut().query_filtered::<Entity, With<Place>>();
        q.iter(app.world()).collect()
    };
    for (i, e) in places.iter().enumerate() {
        app.world_mut().entity_mut(*e).insert(PlaceKey(i as u64));
    }
    for _ in 0..2 {
        app.update();
    }
    app
}

fn v1() -> SocietySave {
    serde_json::from_str(V1).expect("the fixture is a save")
}

#[test]
fn a_version_one_save_restores_and_says_it_was_upgraded() {
    let mut app = keyed_village();
    let save = v1();
    assert_eq!(save.version, 1);
    let report = restore(app.world_mut(), save.clone()).expect("a v1 save restores");
    assert_eq!(report.upgraded_from, Some(1));
    assert!(
        report.unknown_keys.is_empty() && report.fresh.is_empty(),
        "{report:?}"
    );

    let after = capture(app.world_mut());
    assert_eq!(after.version, VERSION);
    assert_eq!(after.persons.len(), save.persons.len());
    for (before, now) in save.persons.iter().zip(&after.persons) {
        assert_eq!(before.key, now.key);
        assert_eq!(before.ties, now.ties);
        assert_eq!(before.memories, now.memories);
        assert_eq!(before.memberships, now.memberships);
        assert!(now.stock.iter().all(|level| *level == 0.0), "holds nothing");
    }
    assert_eq!(save.rumors, after.rumors);

    let mut q = app.world_mut().query::<&Stock>();
    for stock in q.iter(app.world()) {
        assert_eq!(stock.get(bevy_society::GoodId(0)), 0.0);
    }

    // A current save reports no upgrade.
    let again = restore(app.world_mut(), after).expect("a current save restores");
    assert_eq!(again.upgraded_from, None);
}

#[test]
fn upgrade_returns_a_current_save_that_survives_serde_and_restore() {
    let up = v1().upgrade().expect("v1 upgrades");
    assert_eq!(up.version, VERSION);
    let json = serde_json::to_string(&up).expect("plain data");
    let back: SocietySave = serde_json::from_str(&json).expect("a save");
    assert_eq!(back.version, VERSION);
    let mut app = keyed_village();
    let report = restore(app.world_mut(), back).expect("restores");
    assert_eq!(report.upgraded_from, None);
}

#[test]
fn an_unknown_version_is_refused_and_leaves_the_world_alone() {
    for bad in [0, VERSION + 1] {
        let mut app = keyed_village();
        let before = capture(app.world_mut());
        let mut save = v1();
        save.version = bad;
        match restore(app.world_mut(), save) {
            Err(RestoreError::Version { found, expected }) => {
                assert_eq!((found, expected), (bad, VERSION));
            }
            other => panic!("version {bad} must be refused, got {other:?}"),
        }
        let after = capture(app.world_mut());
        assert_eq!(before.persons, after.persons);
    }
}
