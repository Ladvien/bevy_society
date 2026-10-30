//! **Authored placement** (issue #41): a RON list of persons with homes, workplaces, factions and
//! ties round-trips, spawns, and gives the same day-1 positions for the same seed.

mod common;

use bevy::prelude::*;
use bevy_society::{BoundPlaces, DefsError, Home, Memberships, PlacementFile, SocietyDefs, Ties};
use common::{headless, run, settings};

const DEFS: &str = r#"(
    needs: [(id: "hunger", decay_per_day: 1.0)],
    factions: [(id: "millers")],
    activities: [(
        id: "grind", target: Bound("work"), duration_hours: 2.0,
        considerations: [(input: Need("hunger"), curve: Linear(-1.0, 1.0))],
        satisfies: [("hunger", 0.3)],
    )],
)"#;

const PLACEMENT: &str = r#"(
    places: [
        (id: "mill", at: (10.0, 0.0), capacity: 3, tags: ["work"]),
        (id: "cottage", at: (-4.0, 2.0), capacity: 2),
    ],
    persons: [
        (name: "Ada", home: Some("cottage"), bound: [("work", "mill")], factions: ["millers"],
         needs: [("hunger", 0.4)], personality: (extraversion: 0.9),
         ties: [(with: "Bram", affinity: 0.2)]),
        (name: "Bram", home: Some("cottage"), at: Some((1.0, 1.0)), autonomous: false),
    ],
)"#;

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(DEFS).expect("defs parse")
}

#[test]
fn a_placement_round_trips_through_ron() {
    let file = PlacementFile::from_ron(PLACEMENT).expect("parses");
    let again = PlacementFile::from_ron(&file.to_ron().expect("writes")).expect("reparses");
    assert_eq!(file, again);
    assert_eq!(file.persons[0].personality.openness, 0.5);
    assert!(file.persons[0].autonomous && !file.persons[1].autonomous);
}

#[test]
fn spawning_gives_persons_homes_workplaces_factions_and_ties() {
    let mut app = headless(defs(), settings());
    let placed = PlacementFile::from_ron(PLACEMENT)
        .unwrap()
        .spawn(app.world_mut())
        .expect("spawns");
    let (ada, bram) = (
        placed.person("Ada").unwrap(),
        placed.person("Bram").unwrap(),
    );
    let world = app.world();
    assert_eq!(
        world.get::<Home>(ada).unwrap().0,
        placed.place("cottage").unwrap()
    );
    assert_eq!(
        world.get::<BoundPlaces>(ada).unwrap().get("work"),
        placed.place("mill")
    );
    assert_eq!(world.get::<Memberships>(ada).unwrap().0.len(), 1);
    assert_eq!(
        world.get::<Ties>(ada).unwrap().get(bram).unwrap().affinity,
        0.2
    );
    // Ada starts at her cottage, Bram where he was put.
    assert_eq!(
        world.get::<Transform>(ada).unwrap().translation,
        Vec3::new(-4.0, 0.0, 2.0)
    );
    assert_eq!(
        world.get::<Transform>(bram).unwrap().translation,
        Vec3::new(1.0, 0.0, 1.0)
    );
}

#[test]
fn a_bad_reference_spawns_nothing_and_names_the_line() {
    let mut app = headless(defs(), settings());
    let bad = PLACEMENT.replace("\"mill\")]", "\"quarry\")]");
    let err = PlacementFile::from_ron(&bad)
        .unwrap()
        .spawn(app.world_mut())
        .unwrap_err();
    assert_eq!(
        err,
        DefsError::UnknownId {
            kind: "place",
            id: "quarry".into(),
            owner: "person 'Ada'".into()
        }
    );
    assert_eq!(
        app.world_mut().query::<&Name>().iter(app.world()).count(),
        0
    );
}

fn day_one(seed: u64) -> Vec<Vec3> {
    let mut s = settings();
    s.seed = seed;
    let mut app = headless(defs(), s);
    let placed = PlacementFile::from_ron(PLACEMENT)
        .unwrap()
        .spawn(app.world_mut())
        .unwrap();
    run(&mut app, 200);
    placed
        .persons
        .iter()
        .map(|(_, e)| app.world().get::<Transform>(*e).unwrap().translation)
        .collect()
}

#[test]
fn the_same_seed_gives_the_same_day_one_positions() {
    assert_eq!(day_one(7), day_one(7));
}
