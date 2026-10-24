//! **The `need-cycle` scenario, measured per person — and why it is `pending`.**
//!
//! Its observable, verbatim from `docs/loop/scenarios.yaml`:
//!
//! > Every person's `rest` and `food` needs fall and rise again over a game-day: each crosses below
//! > 0.35 at least once and recovers above 0.7 at least once.
//!
//! **It does not hold, and two iterations said it did.** At the scenario's own settings — 64 s/day,
//! 80 000 ticks, 8 people, seed 7 — measured per person:
//!
//! | need | lowest any person reached | crosses 0.35? |
//! |---|---|---|
//! | social | **0.39** | no |
//! | rest | **0.50** | no |
//! | food | 0.00 | **yes, once a day** |
//! | fun | 0.12 | mostly, but 2 of 8 people never |
//!
//! Only `food` cycles for everyone. The claim that all four did came from the evidence series recording
//! a row at **tick 0**, before `size_person_state` fills `Needs`: every need read 0.000 once, which made
//! "crosses below 0.35" true of the *mean* while no individual came close. A mean over eight people, all
//! out of phase, cannot fake a 0.000 that none of them reached — the artefact was the row, not the
//! averaging.
//!
//! Three needs staying above 0.5 is **correct**: their rates are generous enough that they never run low.
//! The observable assumes starvation that does not happen, which is a definitions question rather than a
//! code defect, and the scenario stays `pending` until someone decides which it should be.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Needs, Place, PlaceTags, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Per-person minimum and maximum on one need, and whether it crossed the observable's thresholds.
#[derive(Debug, Clone, Copy)]
struct Cycle {
    low: f32,
    high: f32,
}

impl Cycle {
    fn fell_and_rose(&self) -> bool {
        // The observable's shape: it must have been low at some point and high at another.
        self.low < 0.35 && self.high > 0.7
    }
}

/// Runs the scenario and returns one `Cycle` per person per need.
fn run() -> Vec<[Cycle; 4]> {
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    short_day.start_hour = 7.0;
    let mut app = headless(defs, short_day);
    let well = app
        .world_mut()
        .spawn((
            Name::new("well"),
            Place { capacity: 8 },
            PlaceTags(vec![
                "gathering".to_owned(),
                "food".to_owned(),
                "work".to_owned(),
            ]),
            Transform::default(),
        ))
        .id();
    let mut people = Vec::new();
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(2.0 * angle.cos(), 2.0 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Home(well), Autonomous));
        people.push(person);
    }
    let mut cycles = vec![
        [Cycle {
            low: 1.0,
            high: 0.0
        }; 4];
        people.len()
    ];
    // The first update is what sizes `Needs`; sampling before it records a state the society never has.
    for _ in 0..80_000 {
        app.update();
        for (k, &person) in people.iter().enumerate() {
            if let Some(needs) = app.world().get::<Needs>(person)
                && needs.0.len() == 4
            {
                for (cycle, level) in cycles[k].iter_mut().zip(&needs.0) {
                    cycle.low = cycle.low.min(*level);
                    cycle.high = cycle.high.max(*level);
                }
            }
        }
    }
    cycles
}

#[test]
fn only_food_cycles_for_every_person_and_the_scenario_says_so() {
    let cycles = run();
    let names = ["social", "rest", "food", "fun"];
    for (i, name) in names.iter().enumerate() {
        let lows: Vec<f32> = cycles.iter().map(|c| c[i].low).collect();
        let all_cycle = cycles.iter().all(|c| c[i].fell_and_rose());
        println!(
            "{name:6} lowest any person reached {:.2}, cycles for all 8: {all_cycle}",
            lows.iter().cloned().fold(f32::INFINITY, f32::min)
        );
    }
    // `food` is the one need that genuinely cycles for everyone, and this is what makes the observable's
    // first sentence false for the rest: it is a statement about the definitions' rates, not about code.
    assert!(
        cycles.iter().all(|c| c[2].fell_and_rose()),
        "food must cycle once a day for every person"
    );
    assert!(
        cycles.iter().all(|c| !c[1].fell_and_rose()),
        "rest stays above 0.5 at these rates; if this ever fails the observable became true and the \
         scenario can be marked measured"
    );
}

#[test]
fn a_series_row_taken_before_the_first_update_is_not_a_sample() {
    // The concrete artefact this iteration removed: at tick 0 `Needs` is empty, so a mean over the
    // villagers reads 0.000 for every need. Recording it made "crosses below 0.35" true of the mean
    // while no individual ever came close.
    let defs = SocietyDefs::starter().expect("the starter pack compiles");
    let mut short_day = settings();
    short_day.seconds_per_day = 64.0;
    let mut app = headless(defs, short_day);
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    let before = app
        .world()
        .get::<Needs>(person)
        .map(|n| n.0.len())
        .unwrap_or(0);
    // Two updates: the first accumulates fixed-step time and the second is the society's first tick,
    // because `size_person_state` runs in `FixedUpdate` and the clock needs one step to be due.
    app.update();
    app.update();
    let after = app
        .world()
        .get::<Needs>(person)
        .map(|n| n.0.len())
        .unwrap_or(0);
    println!("needs before any update: {before}; after two: {after}");
    assert_eq!(
        before, 0,
        "the pre-update state has no needs, so a row taken there is not a sample of the society"
    );
    assert!(after > 0, "the first society tick sizes them");
}
