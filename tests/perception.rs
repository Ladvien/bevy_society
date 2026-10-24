//! **The grid must find exactly what the all-pairs scan found.**
//!
//! `perceive` runs a uniform grid: one cell per `perception_radius`, neighbours read from the nine cells
//! around a person's own (Ericson 2005, *Real-Time Collision Detection*, §7.1.1 — cells "large enough
//! (but not much larger) to fit the largest object"; for points that is the query radius). It replaced an
//! all-pairs scan, and it is only safe because the answer is **identical**: the same neighbours, in the
//! same ascending index order, with the same distances.
//!
//! That identity is what this file pins. It is not a tolerance test: a neighbour set that differs by one
//! person changes the society, so the comparison is exact on contents *and* order.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Perception, Person, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Builds a society of `n` people on a spiral and returns the perception the grid produced, next to what
/// brute force says it should have been.
fn compare(n: usize, radius: f32) -> (usize, usize) {
    let mut day = settings();
    day.seconds_per_day = 240.0;
    day.perception_radius = radius;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    for i in 0..n {
        // A golden-angle spiral, like the benchmark: no grid-aligned artefacts, so a cell boundary can
        // fall anywhere relative to the population.
        let fraction = i as f32 / n.max(1) as f32;
        let r = 40.0 * fraction.sqrt();
        let angle = i as f32 * 2.399_963;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(r * angle.cos(), r * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person));
    }
    // Two updates: `size_person_state` on the first, perception on the second.
    app.update();
    app.update();

    let perception = app.world().resource::<Perception>();
    let positions = perception.pos.clone();
    let mut mismatched = 0;
    for (i, list) in perception.near.iter().enumerate() {
        let mut expected: Vec<u32> = (0..positions.len() as u32)
            .filter(|j| *j as usize != i)
            .filter(|j| positions[i].distance(positions[*j as usize]) <= radius)
            .collect();
        expected.sort_unstable();
        let found: Vec<u32> = list.iter().map(|(j, _)| *j).collect();
        if found != expected {
            mismatched += 1;
        }
    }
    (mismatched, perception.near.len())
}

#[test]
fn the_grid_finds_exactly_the_all_pairs_neighbours() {
    for (n, radius) in [(1, 8.0), (2, 8.0), (40, 8.0), (200, 8.0), (500, 4.0)] {
        let (mismatched, lists) = compare(n, radius);
        assert_eq!(
            mismatched, 0,
            "{n} people at a {radius} m radius: {mismatched} of {lists} neighbour lists differ from the \
             all-pairs answer. The grid must reproduce it exactly — same neighbours, same ascending index \
             order — or the society moves"
        );
        assert_eq!(lists, n, "every person must be in the grid");
    }
}
