//! **Production that answers to the shelf.** The starter pack's `bake` is a one-hour dawn errand: it runs on the clock,
//! whatever the larder holds. `examples/assets/provisioning.society.ron` replaces `bake` and `eat` with versions that
//! read `Input::Stock`, so a hungry household with a bare shelf makes bread whenever it is light, and one with bread
//! in hand does not. That is a balancing loop on a stock (Meadows, *Thinking in Systems*; Zunic's production-consumption
//! model, corpus doc `eth-cgl-ar_games-Zun16a`); the clock is an open loop, stable only if its output happens to equal what
//! the village eats, which a town of sixty does not (`docs/loop/runs/2026-10-30-28/`).
//!
//! The town-scale result is an evidence run (`eval.md`): it depends on the clock and the layout, and a unit test that
//! depends on both would be pinning an emergent number. These tests pin the two halves of the mechanism instead, each
//! against the starter pack so a failure of the overlay cannot hide behind one of the pack.
mod common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, DefsFile, Home, Needs, Place, PlaceTags, SocietyDefs, SocietySettings, Stock,
};

const PEOPLE: usize = 12;
/// Game-hours a test runs: the default clock is 240 s a game-day, so ten real seconds is one game-hour.
const HOURS: usize = 4;
const UPDATES_PER_HOUR: usize = 64 * 10;

fn compile(overlay: bool) -> SocietyDefs {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    // **Only the two activities under test**, so a person's choice is the pack's rule and not a draw against `chat`
    // and `compliment`: with nothing else to score, whoever has a positive `bake` score bakes.
    file.activities.retain(|a| a.id == "bake" || a.id == "eat");
    if overlay {
        let overlay =
            DefsFile::from_ron(include_str!("../examples/assets/provisioning.society.ron"))
                .expect("the provisioning overlay parses");
        for activity in overlay.activities {
            let slot = file
                .activities
                .iter_mut()
                .find(|a| a.id == activity.id)
                .expect("the starter pack has the activity the overlay replaces");
            *slot = activity;
        }
    }
    SocietyDefs::compile(file).expect("the pack compiles")
}

/// Twelve people at one food place, each holding `loaves` and fed to `food`, starting at `start_hour`. Returns the app
/// and the total bread in the town after each game-hour of the run.
fn hourly_bread(overlay: bool, start_hour: f32, loaves: f32, food: f32) -> Vec<f32> {
    let defs = compile(overlay);
    let food_need = defs.need_id("food").expect("the starter pack has food").0 as usize;
    let settings = SocietySettings {
        start_hour,
        ..common::settings()
    };
    let mut app = common::headless(defs, settings);
    let table = app
        .world_mut()
        .spawn((
            Name::new("table"),
            Place { capacity: 64 },
            PlaceTags(vec!["food".to_owned()]),
            Transform::default(),
        ))
        .id();
    let people: Vec<Entity> = (0..PEOPLE)
        .map(|i| {
            let person =
                common::spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.2, 0.0));
            app.world_mut().entity_mut(person).insert((
                Autonomous,
                Home(table),
                Stock(vec![loaves]),
            ));
            person
        })
        .collect();
    // The first updates size every person's `Needs`; set the level once they exist.
    common::run(&mut app, 2);
    for &person in &people {
        if let Some(mut needs) = app.world_mut().get_mut::<Needs>(person) {
            needs.0[food_need] = food;
        }
    }
    (0..HOURS)
        .map(|_| {
            common::run(&mut app, UPDATES_PER_HOUR);
            people
                .iter()
                .map(|&p| app.world().get::<Stock>(p).map_or(0.0, |s| s.0[0]))
                .sum()
        })
        .collect()
}

/// **A hungry household with a bare shelf makes bread at noon, when the starter pack's dawn window is shut.** Twelve
/// people at 12:00, hungry (`food` 0.2), holding nothing: on the starter pack nobody can make bread for the next four
/// hours, so the town's bread is exactly zero; on the overlay they bake and the town holds bread within them.
#[test]
fn a_hungry_household_with_nothing_makes_bread_at_noon() {
    let clock = hourly_bread(false, 12.0, 0.0, 0.2);
    let shelf = hourly_bread(true, 12.0, 0.0, 0.2);
    println!("bread by game-hour, clock {clock:?}, shelf {shelf:?}");
    assert!(
        clock.iter().all(|&total| total == 0.0),
        "the dawn window is shut at noon, so the clock-driven pack makes none: {clock:?}"
    );
    assert!(
        shelf.last().copied().unwrap_or(0.0) > 2.0,
        "hungry households with a bare shelf must bake within four hours: {shelf:?}"
    );
}

/// **A household that already has bread does not make more.** Twelve people at 06:00 holding three loaves each and not
/// hungry: the starter pack's dawn bake runs whatever the shelf holds, and the town's bread rises above what it held;
/// on the overlay a shelf of loaves has nothing to make, so the total never rises.
#[test]
fn a_household_with_bread_in_hand_does_not_bake() {
    let start = PEOPLE as f32 * 3.0;
    let clock = hourly_bread(false, 6.0, 3.0, 1.0);
    let shelf = hourly_bread(true, 6.0, 3.0, 1.0);
    println!("bread by game-hour, clock {clock:?}, shelf {shelf:?}, started with {start}");
    assert!(
        clock.iter().any(|&total| total > start + 0.5),
        "the clock-driven dawn bake adds to a full-enough shelf: {clock:?}"
    );
    assert!(
        shelf.iter().all(|&total| total <= start + 1e-4),
        "a shelf with bread on it must make no more: {shelf:?}"
    );
}
