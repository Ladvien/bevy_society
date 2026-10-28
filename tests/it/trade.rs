//! **Trade between two villagers, and the property that makes it worth calling an economy.**
//!
//! Edgeworth barter is the shape (corpus doc `10.1007_11678809_10`): "buyers and sellers interact directly
//! with each other in repeated pairwise transactions, **without the benefit of an auctioneer**", whose
//! emergent behaviour is that "**in repeated trades, [it] balances supply and demand**".
//!
//! Hall & Milgrom give the condition a trade has to meet first, reported in Hall (2005, DOI
//! `10.3386/w11678`): "once ... they have found that they would enjoy a **joint surplus**, the threats to
//! disclaim the match are hollow". So a trade needs a surplus, and the surplus here is two stores valued by
//! `1 / (1 + held)`.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, GoodId, Home, PersonKey, Place, PlaceTags, SocietyDefs, Stock, value_of,
};

/// The `swap` activity, the only thing a villager can do in these tests.
const SWAP: &str = r#"(id: "swap", target: Person, duration_hours: 0.2,
                 considerations: [(input: Surplus("bread"), curve: Linear(1.0, 0.0))],
                 satisfies: [], interaction: Some("barter"), barks: [])"#;

/// A village with `bread`, optionally a second good, and the given activities.
fn defs_with(goods: &str, activities: &str, barter: &str) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        r#"(
            goods: [{goods}],
            needs: [(id: "fun", decay_per_day: 0.0)],
            activities: [{activities}],
            interactions: [{barter}],
        )"#
    ))
    .expect("the definitions parse")
}

/// A village where one person has plenty and one has none, and a `swap` activity that can pair them.
fn defs() -> SocietyDefs {
    defs_with(
        r#"(id: "bread")"#,
        SWAP,
        r#"(id: "barter",
                 on_accept: (
                    actor_gives: [("bread", 1.0)],
                    actor_to_target: (trust: 0.05),
                    target_to_actor: (trust: 0.05),
                 ))"#,
    )
}

/// Villagers at one place, spaced 0.3 apart, holding the given stores of each good.
fn village_of(defs: SocietyDefs, holds: &[Vec<f32>]) -> (App, Vec<Entity>) {
    let mut app = common::headless(defs, common::settings());
    let bench = app
        .world_mut()
        .spawn((
            Name::new("bench"),
            Place { capacity: 8 },
            PlaceTags(vec!["bench".to_owned()]),
            Transform::default(),
        ))
        .id();
    let mut villagers = Vec::new();
    for (i, store) in holds.iter().enumerate() {
        let entity =
            common::spawn_person(&mut app, &format!("V{i}"), Vec2::new(i as f32 * 0.3, 0.0));
        app.world_mut().entity_mut(entity).insert((
            Autonomous,
            Home(bench),
            PersonKey(i as u64),
            Stock(store.clone()),
        ));
        villagers.push(entity);
    }
    common::run(&mut app, 2);
    (app, villagers)
}

/// Two villagers at one place with the given starting bread.
fn pair(holds: [f32; 2]) -> (App, Vec<Entity>) {
    village_of(defs(), &[vec![holds[0]], vec![holds[1]]])
}

fn bread(app: &App, entity: Entity) -> f32 {
    app.world()
        .get::<Stock>(entity)
        .map_or(0.0, |s| s.get(GoodId(0)))
}

/// **The valuation that makes a surplus possible.** Diminishing marginal value, with no curve and no
/// `Tuning` field: a person with nothing values a unit at 1.0 and a person with a full store far less.
#[test]
fn a_unit_is_worth_more_to_whoever_has_less() {
    assert!((value_of(0.0) - 1.0).abs() < 1e-6);
    assert!(value_of(0.0) > value_of(1.0));
    assert!(value_of(1.0) > value_of(6.0));
    assert!(value_of(100.0) > 0.0, "value must never reach zero");
    // And it is monotone all the way down, which is what a surplus needs.
    let mut previous = f32::MAX;
    for held in 0..20 {
        let v = value_of(held as f32);
        assert!(v < previous, "value must fall as the store rises");
        previous = v;
    }
}

/// **A good moves from the fuller store to the emptier one.** The transfer is declared by the interaction
/// and executed in one call, so the pair cannot create or lose goods.
#[test]
fn a_trade_moves_one_unit_to_the_person_who_has_none() {
    let (mut app, villagers) = pair([2.0, 0.0]);
    common::run(&mut app, 800);
    let (rich, poor) = (bread(&app, villagers[0]), bread(&app, villagers[1]));
    assert!(
        (poor - 1.0).abs() < 1e-5 && (rich - 1.0).abs() < 1e-5,
        "two loaves and none must level to one each, the giver keeping the reserve: rich {rich}, poor {poor}"
    );
}

/// **No surplus, no trade.** Two people with the same store value a unit identically, so there is nothing to
/// split and the activity should not fire.
#[test]
fn two_equal_stores_do_not_trade() {
    let (mut app, villagers) = pair([2.0, 2.0]);
    common::run(&mut app, 800);
    assert!(
        (bread(&app, villagers[0]) - 2.0).abs() < 1e-5
            && (bread(&app, villagers[1]) - 2.0).abs() < 1e-5,
        "equal stores must not swap: {} and {}",
        bread(&app, villagers[0]),
        bread(&app, villagers[1])
    );
}

/// Same seed, same stores, same trade: no draw is added.
#[test]
fn trade_is_deterministic() {
    let run = || {
        let (mut app, villagers) = pair([3.0, 0.0]);
        common::run(&mut app, 600);
        (bread(&app, villagers[0]), bread(&app, villagers[1]))
    };
    assert_eq!(run(), run(), "two identical runs must trade identically");
}

/// **A giver keeps the reserve.** Half a loaf and exactly one loaf are both at or below
/// `Tuning::trade_reserve` (1.0), so the surplus is zero however empty the other shelf is: a villager does not
/// hand over the last thing it has. The first version gated at "at least the reserve", and the log of who gave
/// what showed villagers giving away their only loaf ("Vesna holds 0.0").
#[test]
fn a_giver_at_or_below_the_reserve_does_not_trade() {
    for held in [0.5, 1.0] {
        let (mut app, villagers) = pair([held, 0.0]);
        common::run(&mut app, 800);
        assert!(
            (bread(&app, villagers[0]) - held).abs() < 1e-6 && bread(&app, villagers[1]) == 0.0,
            "a giver holding {held} must keep it: {} and {}",
            bread(&app, villagers[0]),
            bread(&app, villagers[1])
        );
    }
}

/// **Both sides move in one call, and the pair cannot create goods.** Two goods, each villager holding the
/// one the other lacks: `actor_gives` and `target_gives` are both executed, so each ends with one of each
/// and the village's totals are exactly what they were.
#[test]
fn a_barter_moves_a_good_each_way_and_conserves_both() {
    let defs = defs_with(
        r#"(id: "bread"), (id: "fish")"#,
        SWAP,
        r#"(id: "barter",
                 on_accept: (
                    actor_gives: [("bread", 1.0)],
                    target_gives: [("fish", 1.0)],
                 ))"#,
    );
    let (mut app, villagers) = village_of(defs, &[vec![2.0, 0.0], vec![0.0, 2.0]]);
    common::run(&mut app, 800);
    let held = |e: Entity| {
        let stock = app.world().get::<Stock>(e).expect("stock");
        (stock.get(GoodId(0)), stock.get(GoodId(1)))
    };
    let (a, b) = (held(villagers[0]), held(villagers[1]));
    assert!(
        a.1 > 0.0 && b.0 > 0.0,
        "each side must have received the other's good: {a:?} and {b:?}"
    );
    assert!(
        (a.0 + b.0 - 2.0).abs() < 1e-5 && (a.1 + b.1 - 2.0).abs() < 1e-5,
        "a barter cannot create or lose either good: {a:?} and {b:?}"
    );
}

/// **Trade levels the village, which is what "balances supply and demand" means for a store.** One person
/// holds four loaves and three hold none. Without a `swap` activity the spread stays exactly where it began;
/// with one, the loaves move toward the people who have none and the spread falls. The total is the same.
#[test]
fn trade_narrows_the_spread_of_stores() {
    let spread = |defs: SocietyDefs| {
        let (mut app, villagers) = village_of(defs, &[vec![4.0], vec![0.0], vec![0.0], vec![0.0]]);
        common::run(&mut app, 2_400);
        let held: Vec<f32> = villagers.iter().map(|&e| bread(&app, e)).collect();
        let mean = held.iter().sum::<f32>() / held.len() as f32;
        let sd = (held.iter().map(|h| (h - mean).powi(2)).sum::<f32>() / held.len() as f32).sqrt();
        (held.iter().sum::<f32>(), sd)
    };
    let (total_without, sd_without) = spread(defs_with(
        r#"(id: "bread")"#,
        r#"(id: "linger", target: Itself, duration_hours: 1.0,
              considerations: [(input: Hour, curve: Linear(0.0, 0.5))],
              satisfies: [], barks: [])"#,
        "",
    ));
    let (total_with, sd_with) = spread(defs());
    assert!((total_without - 4.0).abs() < 1e-5 && (total_with - 4.0).abs() < 1e-5);
    assert!(
        sd_without > 1.5,
        "the control village must start uneven and stay so: sd {sd_without}"
    );
    assert!(
        sd_with < sd_without * 0.6,
        "trade must level the stores: sd {sd_with} against {sd_without} without it"
    );
}
