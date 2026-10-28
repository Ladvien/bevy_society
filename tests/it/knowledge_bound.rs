//! **A person's rumor memory is bounded, like every other memory in the crate.**
//!
//! `Memories` evicts its lowest-salience entry at `memory_capacity` (default 64), and `Ties` trims at
//! `dunbar_capacity`. `Knowledge` had no cap at all, so it grew with the number of rumors the world
//! ever planted — and `best_telling` scans every entry on every gossip, so gossip cost grew with the
//! run. Measured on 12 people round one well:
//!
//! | tick | rumors in book | max entries a person holds |
//! |---|---|---|
//! | 300 | 24 | 24 |
//! | 1 000 | 88 | 78 |
//! | 2 000 | 173 | 155 |
//! | 4 000 | 386 | **344** |
use crate::common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Knowledge, Place, PlaceTags, SocietyDefs, SocietySettings};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    // A deed on every tick would be ideal, but the simplest reliable source of many rumors is a
    // crowd that snubs: every judged violation plants one. Use the starter pack, which has all four
    // deeds and the activities that produce them.
    SocietyDefs::starter().expect("the starter pack parses")
}

#[test]
fn a_person_does_not_remember_every_rumor_forever() {
    let capacity = 32;
    let settings = SocietySettings {
        seconds_per_day: 4.0,
        tick_secs: 1.0 / 64.0,
        knowledge_capacity: capacity,
        ..common::settings()
    };
    let mut app = headless(defs(), settings);
    let plaza = app
        .world_mut()
        .spawn((
            Name::new("plaza"),
            Place { capacity: 16 },
            PlaceTags(vec![
                "gathering".to_owned(),
                "food".to_owned(),
                "work".to_owned(),
            ]),
            Transform::default(),
        ))
        .id();
    for i in 0..12 {
        let angle = i as f32 * std::f32::consts::TAU / 12.0;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(0.3 * angle.cos(), 0.3 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Home(plaza), Autonomous));
    }
    common::run(&mut app, 3000);

    let mut query = app.world_mut().query::<&Knowledge>();
    let lens: Vec<usize> = query.iter(app.world()).map(|k| k.0.len()).collect();
    let max = lens.iter().copied().max().unwrap_or(0);
    let book = app
        .world()
        .resource::<bevy_society::RumorBook>()
        .rumors
        .len();
    println!("book={book} knowledge max={max} of capacity {capacity}");
    assert!(
        book > capacity,
        "the run must make more rumors than the cap, or it cannot show anything: {book}"
    );
    assert!(
        max <= capacity,
        "no person may hold more than {capacity} rumors, but one holds {max} \
         (the book has {book})"
    );
}
