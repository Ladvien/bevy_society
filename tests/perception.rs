//! **Perception finds exactly the neighbours brute force finds, and costs what it costs.**
//!
//! `perceive` is a brute-force double loop over every person: `O(n²)`, measured at **1.45 ms** a tick
//! at 1 000 people and **35.5 ms** at 5 000 — 12.9% of the whole tick and growing with the square of
//! the population.
//!
//! A uniform grid was implemented against Ihmsen et al., *SPH Fluids in Computer Graphics*, Sec. 2
//! ("grids are built in `O(n)` and particles are accessed in `O(1)`") and **reverted**: measured, it
//! gave **nothing** at 5 000 (297.96 vs 300.8 ms/tick), was **slower** at small populations (0.707 vs
//! 0.573 at 100), and a `BTreeMap` variant was slower still (16.97 vs 15.67 at 1 000). The reason is
//! the neighbour lists: brute force produces them in ascending index order for free, the grid
//! scrambles them and pays to sort back, and that sort costs as much as the distance checks the grid
//! removes.
//!
//! This test keeps the reference brute force alive, so a future index is checked against it rather
//! than trusted.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Perception, Place, PlaceTags, SocietyDefs, SocietySettings};
use common::{headless, settings, spawn_person};

/// A crowd laid out so that people sit near cell boundaries, in clusters, and alone.
fn crowd(population: usize) -> bevy::app::App {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let mut short_day = settings();
    short_day.seconds_per_day = 4.0;
    let mut app = headless(defs, short_day);
    let well = app
        .world_mut()
        .spawn((
            Name::new("well"),
            Place { capacity: 64 },
            PlaceTags(vec![
                "gathering".to_owned(),
                "food".to_owned(),
                "work".to_owned(),
            ]),
            Transform::default(),
        ))
        .id();
    for i in 0..population {
        // Deliberately awkward: a fine grid of positions at the perception radius, so cells are
        // filled to their edges and plenty of people land exactly on a boundary.
        let x = (i % 13) as f32 * 0.98;
        let y = (i / 13) as f32 * 0.98;
        let person = spawn_person(&mut app, &format!("P{i}"), Vec2::new(x, y));
        app.world_mut()
            .entity_mut(person)
            .insert((Home(well), Autonomous));
    }
    app
}

/// A neighbour list: `(index, distance)` pairs, ascending by index.
type Near = Vec<Vec<(u32, f32)>>;
/// A contact list: indices, ascending.
type Contact = Vec<Vec<u32>>;

/// Every neighbour list the brute-force loop would produce, from the reported positions.
fn brute_force(perception: &Perception, settings: &SocietySettings) -> (Near, Contact) {
    let mut near = vec![Vec::new(); perception.pos.len()];
    let mut contact = vec![Vec::new(); perception.pos.len()];
    for (i, pi) in perception.pos.iter().enumerate() {
        for (j, pj) in perception.pos.iter().enumerate() {
            if i == j {
                continue;
            }
            let d = pi.distance(*pj);
            if d <= settings.perception_radius {
                near[i].push((j as u32, d));
            }
            if d <= settings.contact_radius {
                contact[i].push(j as u32);
            }
        }
    }
    (near, contact)
}

#[test]
fn perception_finds_exactly_what_brute_force_finds() {
    let mut app = crowd(180);
    // Let the society run so people have moved off the tidy starting grid.
    for _ in 0..600 {
        app.update();
    }
    let perception = app.world().resource::<Perception>().clone();
    let settings = app.world().resource::<SocietySettings>().clone();
    let (near_brute, contact_brute) = brute_force(&perception, &settings);

    let mut compared = 0usize;
    for (i, (grid_list, brute_list)) in perception.near.iter().zip(&near_brute).enumerate() {
        assert_eq!(
            grid_list.len(),
            brute_list.len(),
            "person {i} has {} neighbours by grid and {} by brute force",
            grid_list.len(),
            brute_list.len()
        );
        for (a, b) in grid_list.iter().zip(brute_list) {
            assert_eq!(
                a.0, b.0,
                "person {i}: neighbour order or membership differs"
            );
            assert!(
                (a.1 - b.1).abs() < 1e-6,
                "person {i}: distance to {} differs, {} vs {}",
                a.0,
                a.1,
                b.1
            );
            compared += 1;
        }
    }
    for (i, (grid_list, brute_list)) in perception.contact.iter().zip(&contact_brute).enumerate() {
        assert_eq!(grid_list, brute_list, "person {i}: contact lists differ");
    }
    println!(
        "compared {compared} neighbour entries across {} people",
        perception.pos.len()
    );
    assert!(
        compared > 100,
        "the layout must produce neighbours to compare: {compared}"
    );
}
