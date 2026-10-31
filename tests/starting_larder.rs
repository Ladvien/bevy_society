//! **A person is created holding what the pack says a person holds.**
//!
//! `GoodDef::initial` is to a good what `NeedDef::initial` is to a need: the level a new person starts at. A
//! society whose larders all start empty has a transient the running society does not. Measured on the starter
//! pack's village (24 people, seed 99, `docs/loop/runs/2026-10-30-27/research.md`): nobody held bread for the first
//! 22 game-hours, so the second game-day had 38.8 % of its person-ticks with `food` below 0.05 against 1-11 % from the
//! third day on.
mod common;

use bevy::prelude::*;
use bevy_society::{DefsFile, Needs, Person, SocietyDefs, SocietySettings, Stock, Tuning};

fn defs(goods: &str) -> SocietyDefs {
    SocietyDefs::from_ron(&format!("(goods: [{goods}])")).expect("the definitions parse")
}

fn spawn(app: &mut App, index: usize) -> Entity {
    common::spawn_person(app, &format!("P{index}"), Vec2::new(index as f32, 0.0))
}

fn held(app: &App, person: Entity) -> Vec<f32> {
    app.world()
        .get::<Stock>(person)
        .expect("a pack with goods gives every person a Stock")
        .0
        .clone()
}

#[test]
fn a_new_person_holds_each_goods_starting_level_and_nothing_of_one_that_names_none() {
    let mut app = common::headless(
        defs(r#"(id: "bread", initial: 1.5), (id: "wool")"#),
        common::settings(),
    );
    let people: Vec<Entity> = (0..3).map(|i| spawn(&mut app, i)).collect();
    common::run(&mut app, 2);
    for person in people {
        assert_eq!(held(&app, person), vec![1.5, 0.0]);
    }
}

#[test]
fn a_starting_level_above_the_ceiling_is_held_at_the_ceiling() {
    let settings = SocietySettings {
        tuning: Tuning {
            stock_ceiling: 2.0,
            ..Tuning::default()
        },
        ..common::settings()
    };
    let mut app = common::headless(defs(r#"(id: "bread", initial: 5.0)"#), settings);
    let person = spawn(&mut app, 0);
    common::run(&mut app, 2);
    assert_eq!(held(&app, person), vec![2.0]);
}

#[test]
fn a_short_stock_gains_the_missing_good_at_its_start_and_a_full_one_is_the_hosts_own() {
    let mut app = common::headless(
        defs(r#"(id: "bread", initial: 1.5), (id: "wool", initial: 0.75)"#),
        common::settings(),
    );
    let short = spawn(&mut app, 0);
    let full = spawn(&mut app, 1);
    app.world_mut().entity_mut(short).insert(Stock(vec![0.5]));
    app.world_mut()
        .entity_mut(full)
        .insert(Stock(vec![0.0, 0.0]));
    common::run(&mut app, 2);
    assert_eq!(
        held(&app, short),
        vec![0.5, 0.75],
        "the loaf it had is kept"
    );
    assert_eq!(
        held(&app, full),
        vec![0.0, 0.0],
        "a host that spawned an empty larder meant it"
    );
}

#[test]
fn a_starting_level_must_be_a_number_of_zero_or_more() {
    for bad in [-1.0, f32::NAN, f32::INFINITY] {
        let mut file = DefsFile::from_ron(r#"(goods: [(id: "bread")])"#).expect("parses");
        file.goods[0].initial = bad;
        let Err(err) = SocietyDefs::compile(file) else {
            panic!("a starting level of {bad} was accepted");
        };
        let message = err.to_string();
        assert!(
            message.contains("good 'bread'") && message.contains("initial"),
            "the error names the good and the field: {message}"
        );
    }
}

#[test]
fn the_field_is_optional_and_means_nothing_held() {
    let file = DefsFile::from_ron(r#"(goods: [(id: "bread")])"#).expect("parses");
    assert_eq!(file.goods[0].initial, 0.0);
}

/// The share of villager-ticks on game-day 2 with `food` below 0.05, in a village of 24 whose bread starts at
/// `initial` loaves a head.
fn day_two_floor_share(initial: f32) -> f32 {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    file.merge(
        DefsFile::from_ron(include_str!("../examples/assets/village.society.ron"))
            .expect("the village additions parse"),
    )
    .expect("the village adds to the starter");
    file.goods
        .iter_mut()
        .find(|good| good.id == "bread")
        .expect("the starter pack declares bread")
        .initial = initial;
    let defs = SocietyDefs::compile(file).expect("the village compiles");
    let food = defs.need_id("food").expect("the starter pack has food").0 as usize;
    let mut app = common::village::village(defs, 99, 24);
    // 240 s a game-day at 0.25 s an update.
    let per_day = 960;
    common::run(&mut app, per_day);
    let (mut floor, mut total) = (0_u32, 0_u32);
    for _ in 0..per_day {
        app.update();
        let mut q = app.world_mut().query_filtered::<&Needs, With<Person>>();
        for needs in q.iter(app.world()).filter(|needs| !needs.0.is_empty()) {
            total += 1;
            floor += u32::from(needs.level(food) < 0.05);
        }
    }
    floor as f32 / total as f32
}

/// **The mechanism, paired.** Two villages from one seed whose packs differ only in how much bread a person
/// starts with. With none, the whole village reaches the floor together on its second day; with a loaf, the
/// first dawn is fed from the larder and the floor is a fraction of that.
#[test]
fn a_starting_larder_removes_the_second_days_famine() {
    let empty = day_two_floor_share(0.0);
    let stocked = day_two_floor_share(1.0);
    println!(
        "game-day 2, food below 0.05: {empty:.3} with an empty larder, {stocked:.3} with a loaf"
    );
    assert!(
        empty > 0.2,
        "the transient this guards against has to be there to cure: {empty:.3}"
    );
    assert!(
        stocked < empty / 2.0,
        "a loaf a head should at least halve the second day's floor: {stocked:.3} against {empty:.3}"
    );
}
