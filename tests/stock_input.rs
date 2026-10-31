//! **A consideration can read the actor's own shelf.** `Input::Stock(good)` is the appetitive half of a need that
//! a good satisfies: the activity that consumes the good scores by whether there is any to consume, and the one that
//! produces it by whether the shelf is bare. Without it nothing in a pack can close the loop from "my larder is
//! empty" to "make bread", and production runs on the clock alone (`docs/loop/runs/2026-10-30-28/`).
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Doing, Home, Place, PlaceTags, SocietyDefs, Stock};

/// Two activities that need nothing but a shelf: `eat` wants a meal in hand, `make` wants the shelf bare.
fn defs(make_curve: &str) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        r#"(
            goods: [(id: "bread")],
            activities: [
                (id: "eat", target: Itself, duration_hours: 0.5,
                 considerations: [(input: Stock("bread"), curve: Linear(4.0, 0.0))],
                 consumes: [("bread", 0.25)], satisfies: [], interaction: None, barks: []),
                (id: "make", target: Itself, duration_hours: 0.5,
                 considerations: [(input: Stock("bread"), curve: {make_curve})],
                 produces: [("bread", 4.0)], satisfies: [], interaction: None, barks: []),
            ],
        )"#
    ))
    .expect("the definitions parse")
}

/// One person per entry of `held`, each holding that much bread, and the name of what each chose first.
fn choices(make_curve: &str, held: &[f32]) -> Vec<Option<String>> {
    let defs = defs(make_curve);
    let mut app = common::headless(defs.clone(), common::settings());
    let bench = app
        .world_mut()
        .spawn((
            Name::new("bench"),
            Place { capacity: 8 },
            PlaceTags(vec!["bench".to_owned()]),
            Transform::default(),
        ))
        .id();
    let people: Vec<Entity> = held
        .iter()
        .enumerate()
        .map(|(i, &loaves)| {
            let person = common::spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32, 0.0));
            app.world_mut()
                .entity_mut(person)
                .insert((Autonomous, Home(bench), Stock(vec![loaves])));
            person
        })
        .collect();
    common::run(&mut app, 4);
    people
        .iter()
        .map(|&person| {
            app.world()
                .get::<Doing>(person)
                .map(|doing| defs.activity_name(doing.activity).to_owned())
        })
        .collect()
}

/// **The shelf decides which of two activities is possible.** Only one of them scores above zero for each person, so
/// the choice is not a draw: a person with a meal in hand eats and the one with nothing makes bread.
#[test]
fn a_person_with_bread_eats_and_one_without_makes_some() {
    let chose = choices("Linear(-1.0, 1.0)", &[1.0, 0.0]);
    assert_eq!(chose[0].as_deref(), Some("eat"), "holding a loaf: {chose:?}");
    assert_eq!(
        chose[1].as_deref(),
        Some("make"),
        "holding nothing: {chose:?}"
    );
}

/// **The input is clamped to `[0, 1]` like every other.** A person holding six loaves reads the same as one holding
/// one, so a pack's curve describes the first loaf and "a day's supply", and `Linear(-1, 1)` is zero from one loaf up:
/// the person with a shelf of loaves has nothing to make.
#[test]
fn the_shelf_reads_as_one_from_one_loaf_up() {
    let chose = choices("Linear(-1.0, 1.0)", &[1.0, 6.0]);
    assert_eq!(chose[0].as_deref(), Some("eat"));
    assert_eq!(
        chose[1].as_deref(),
        Some("eat"),
        "six loaves must read as one loaf: {chose:?}"
    );
}

/// **A curve rescales the unit to what the pack counts.** `Linear(4, 0)` reads a quarter loaf, a meal, as 1.0 and an
/// empty shelf as 0.0, which is exact zero: with `make` unreachable, a person with nothing has no option at all and
/// idles, where a person with a crumb (0.2 of a loaf, 0.8 of a meal) still eats.
#[test]
fn a_curve_rescales_the_unit_to_what_the_pack_counts() {
    // `Step(2.0)` is never reached by an input clamped to 1.0, so `make` scores zero for everyone.
    let chose = choices("Step(2.0)", &[0.25, 0.2, 0.0]);
    assert_eq!(chose[0].as_deref(), Some("eat"), "{chose:?}");
    assert_eq!(chose[1].as_deref(), Some("eat"), "{chose:?}");
    assert_eq!(chose[2].as_deref(), Some("idle"), "{chose:?}");
}

/// **A good the pack never declares is refused at load,** with the good named, like every other id.
#[test]
fn a_stock_of_an_undeclared_good_is_an_error() {
    let Err(err) = SocietyDefs::from_ron(
        r#"(
            goods: [(id: "bread")],
            activities: [
                (id: "weave", target: Itself, duration_hours: 1.0,
                 considerations: [(input: Stock("wool"), curve: Linear(1.0, 0.0))],
                 satisfies: [], interaction: None, barks: []),
            ],
        )"#,
    ) else {
        panic!("a stock of an undeclared good was accepted");
    };
    let message = err.to_string();
    assert!(
        message.contains("wool") && message.contains("weave"),
        "the error names the good and the activity: {message}"
    );
}
