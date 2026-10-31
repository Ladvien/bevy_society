//! **A town of sixty feeds itself when its households read their own shelves.**
//!
//! The starter pack's `bake` is a one-hour dawn errand drawn against nine other options, so the bread a town makes
//! is whatever the clock and the lottery give it, and it does not scale with the population. At sixty people it makes
//! almost none, the stores run down, and by about game-day 8 the town is hungry for good (`docs/loop/runs/2026-10-30-28/`).
//! `examples/assets/provisioning.society.ron` replaces `bake` and `eat` with versions that read `Input::Stock`: a hungry
//! household with a bare shelf makes bread, and nobody walks to a table with nothing on it. Production then answers to
//! the shelves, which is what a balancing loop is (Meadows, *Thinking in Systems*) and what an open loop on a clock
//! cannot be.
//!
//! Each town is a separate test so the two run on separate threads.
mod common;

use bevy::prelude::*;
use bevy_society::{Home, Needs, Person, Place, SocietyDefs, Stock};

const PEOPLE: usize = 60;
const SEED: u64 = 99;
/// Updates a game-day on the crate's default clock: 240 s at 0.25 s an update.
const PER_DAY: usize = 960;
/// Game-days run before the measurement. The clock-driven town's bread is 0.25 a head by day 8 and 0.05 by day 12; the
/// measurement window is the last day of the run.
const DAYS: usize = 8;

/// The town of `examples/evidence.rs`: each column of eight lives at `PLACES[column % 4]`, so the tavern and the market,
/// which are also the only two places with food, are home to sixteen people each. `village` homes everyone at the well.
/// A table that sixteen people sleep at is full from the evening to the morning, and a full place prices every
/// activity aimed at it at `1 - Tuning::crowding_penalty` — `bake` and `eat` among them — which is the condition the
/// clock-driven `bake` cannot survive.
fn rehome_as_the_evidence_town_does(app: &mut App) {
    let mut places = app.world_mut().query::<(Entity, &Name, &Place)>();
    let homes: Vec<Entity> = common::village::PLACES
        .iter()
        .map(|&(wanted, ..)| {
            places
                .iter(app.world())
                .find(|(_, name, _)| name.as_str() == wanted)
                .map(|(entity, ..)| entity)
                .expect("the village has its four places")
        })
        .collect();
    let mut people = app
        .world_mut()
        .query_filtered::<(Entity, &Name), With<Person>>();
    let mut villagers: Vec<(String, Entity)> = people
        .iter(app.world())
        .map(|(entity, name)| (name.to_string(), entity))
        .collect();
    villagers.sort();
    for (i, (_, entity)) in villagers.into_iter().enumerate() {
        let column = i / 8;
        app.world_mut()
            .entity_mut(entity)
            .insert(Home(homes[column % homes.len()]));
    }
}

/// `(bread a head, share of person-ticks with food below 0.05)` over the last game-day of a town of sixty.
fn last_day(defs: SocietyDefs) -> (f32, f32) {
    let food = defs.need_id("food").expect("the starter pack has food").0 as usize;
    let mut app = common::village::village(defs, SEED, PEOPLE);
    rehome_as_the_evidence_town_does(&mut app);
    common::run(&mut app, (DAYS - 1) * PER_DAY);
    let (mut bread, mut floor, mut ticks) = (0.0_f64, 0_u32, 0_u32);
    for _ in 0..PER_DAY {
        app.update();
        let mut people = app
            .world_mut()
            .query_filtered::<(&Needs, &Stock), With<Person>>();
        for (needs, stock) in people
            .iter(app.world())
            .filter(|(needs, _)| !needs.0.is_empty())
        {
            ticks += 1;
            bread += f64::from(stock.0.first().copied().unwrap_or(0.0));
            floor += u32::from(needs.level(food) < 0.05);
        }
    }
    (
        (bread / f64::from(ticks)) as f32,
        floor as f32 / ticks as f32,
    )
}

/// **The defect, reproduced.** The clock-driven pack leaves the town with a fraction of the bread it began with and a
/// third of its people at the floor on `food`.
#[test]
fn a_town_on_the_clock_driven_bake_runs_out_of_bread() {
    let (bread, floor) = last_day(common::village::village_defs());
    println!("clock-driven town, game-day {DAYS}: {bread:.3} loaves a head, {floor:.3} at the floor");
    assert!(
        bread < 0.5,
        "the town the overlay repairs must be short of bread: {bread:.3} a head"
    );
}

/// **The repair.** The same town, seed and spawn order, with `bake` and `eat` reading the shelf: it holds a
/// loaf-and-more a head, which is what each household wants in hand, and the floor share is a fraction of the
/// clock-driven town's.
#[test]
fn a_town_whose_households_read_their_shelves_holds_its_bread() {
    let (bread, floor) = last_day(common::village::provisioned_village_defs());
    println!("provisioning town, game-day {DAYS}: {bread:.3} loaves a head, {floor:.3} at the floor");
    assert!(
        bread > 0.8,
        "households that bake when hungry and bare must keep the town in bread: {bread:.3} a head"
    );
    assert!(
        floor < 0.3,
        "and fed: no more than 30% of person-ticks at the floor on food: {floor:.3}"
    );
}
