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

#[test]
fn a_place_names_who_lives_or_works_there_and_a_rect_selects_who_stands_in_it() {
    let mut app = headless(defs(), settings());
    let placed = PlacementFile::from_ron(PLACEMENT)
        .unwrap()
        .spawn(app.world_mut())
        .expect("spawns");
    run(&mut app, 1);
    let world = app.world();
    let cottage = bevy_society::place_dependents(world, placed.place("cottage").unwrap());
    let who: Vec<_> = cottage
        .iter()
        .map(|d| (d.name.as_deref().unwrap(), d.role.as_str()))
        .collect();
    assert_eq!(who, [("Ada", "home"), ("Bram", "home")]);
    let mill = bevy_society::place_dependents(world, placed.place("mill").unwrap());
    assert_eq!(mill.len(), 1);
    assert_eq!(
        (mill[0].name.as_deref(), mill[0].role.as_str()),
        (Some("Ada"), "work")
    );

    let (ada, bram) = (
        placed.person("Ada").unwrap(),
        placed.person("Bram").unwrap(),
    );
    let near_origin =
        bevy_society::persons_in_rect(world, Vec2::new(0.0, 0.0), Vec2::new(3.0, 3.0));
    assert_eq!(near_origin, [bram]);
    let west = bevy_society::persons_in_rect(world, Vec2::new(-5.0, 0.0), Vec2::new(-3.0, 3.0));
    assert_eq!(west, [ada]);
}

#[test]
fn capture_reads_a_live_edit_back_and_respawns_the_same_town() {
    let mut app = headless(defs(), settings());
    let placed = PlacementFile::from_ron(PLACEMENT)
        .unwrap()
        .spawn(app.world_mut())
        .unwrap();
    // The user drags Bram across the map and the mill gains a tag.
    let bram = placed.person("Bram").unwrap();
    app.world_mut()
        .get_mut::<Transform>(bram)
        .unwrap()
        .translation = Vec3::new(7.0, 0.0, -3.0);
    let mill = placed.place("mill").unwrap();
    app.world_mut()
        .get_mut::<bevy_society::PlaceTags>(mill)
        .unwrap()
        .0
        .push("loud".into());

    let captured = PlacementFile::capture(app.world_mut()).expect("captures");
    let bram_def = captured.persons.iter().find(|p| p.name == "Bram").unwrap();
    assert_eq!(bram_def.at, Some((7.0, -3.0)));
    let ada = captured.persons.iter().find(|p| p.name == "Ada").unwrap();
    assert_eq!(ada.bound, [("work".to_owned(), "mill".to_owned())]);
    assert_eq!(ada.factions, ["millers"]);
    assert_eq!(ada.ties[0].with, "Bram");
    assert!(!bram_def.autonomous);
    assert_eq!(captured.places[1].tags, ["work", "loud"]);

    // Saved, reloaded into a fresh world, captured again: the same file.
    let reloaded = PlacementFile::from_ron(&captured.to_ron().unwrap()).unwrap();
    let mut fresh = headless(defs(), settings());
    reloaded.spawn(fresh.world_mut()).unwrap();
    assert_eq!(PlacementFile::capture(fresh.world_mut()).unwrap(), captured);
}

#[test]
fn capture_refuses_an_unnamed_person() {
    let mut app = headless(defs(), settings());
    app.world_mut().spawn(bevy_society::Person);
    run(&mut app, 1);
    let err = PlacementFile::capture(app.world_mut()).unwrap_err();
    assert!(err.to_string().contains("needs a Name"), "{err}");
}
