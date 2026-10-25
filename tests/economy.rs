//! **A good economy rests when inflow equals outflow, and moves when it does not.**
//!
//! That is not this crate's invention — it is the acceptance test the corpus's own game-economy model states
//! (*Games and AI*, corpus doc `eth-cgl-ar_games-Zun16a`):
//!
//! > "If the parameters are chosen such that the inflow of resources into the system equals the outflow, then
//! > the economy is **stable and prices fluctuate around a constant value**. However, by upsetting this
//! > balance we can simulate a variety of interesting real market phenomenon, such as **shortages and
//! > oversupply**."
//!
//! So the test is run twice on the same village: once with production matching consumption, once with the
//! producer starved, and the price is expected to sit in the first and climb in the second.
mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, Autonomous, Home, Person, Place, PlaceTags, Prices, SocietyDefs, Stock, Tuning,
};

/// A village with a baker and an eater. `bread_per_bake` and `eat_hours` set whether the two balance.
fn defs(bread_per_bake: f32, eat_hours: f32) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "bake", target: Place("oven"), duration_hours: 0.5,
                 produces: [("bread", {bread_per_bake})],
                 considerations: [(input: Need("food"), curve: Step(0.0))],
                 satisfies: [], interaction: None, barks: []),
                (id: "eat", target: Place("oven"), duration_hours: {eat_hours},
                 consumes: [("bread", 1.0)],
                 considerations: [(input: Need("food"), curve: Step(0.0))],
                 satisfies: [], interaction: None, barks: []),
            ],
        )"#
    ))
    .expect("the definitions parse")
}

/// `people` villagers at one oven, half of them bakers.
fn village(bread_per_bake: f32, eat_hours: f32, people: usize) -> (App, Vec<Entity>) {
    let mut app = common::headless(defs(bread_per_bake, eat_hours), common::settings());
    let oven = app
        .world_mut()
        .spawn((
            Name::new("oven"),
            Place { capacity: 8 },
            PlaceTags(vec!["oven".to_owned()]),
            Transform::default(),
        ))
        .id();
    let mut villagers = Vec::new();
    for i in 0..people {
        let entity =
            common::spawn_person(&mut app, &format!("V{i}"), Vec2::new(i as f32 * 0.4, 0.0));
        app.world_mut()
            .entity_mut(entity)
            .insert((Autonomous, Home(oven)));
        villagers.push(entity);
    }
    common::run(&mut app, 2);
    (app, villagers)
}

/// Total bread held by everyone.
fn total_stock(app: &App, villagers: &[Entity]) -> f32 {
    villagers
        .iter()
        .filter_map(|e| app.world().get::<Stock>(*e))
        .map(|s| s.get(bevy_society::GoodId(0)))
        .sum()
}

fn price(app: &App) -> f32 {
    app.world()
        .resource::<Prices>()
        .get(bevy_society::GoodId(0))
}

/// **The store moves.** A village that bakes and eats has bread that rises and falls; a constant would mean
/// the goods system is not connected to anything.
#[test]
fn production_and_consumption_move_the_store() {
    let (mut app, villagers) = village(2.0, 0.5, 6);
    let mut samples = Vec::new();
    for _ in 0..1_200 {
        app.update();
        samples.push(total_stock(&app, &villagers));
    }
    let distinct = {
        let mut sorted: Vec<u32> = samples.iter().map(|s| (s * 100.0) as u32).collect();
        sorted.sort_unstable();
        sorted.dedup();
        sorted.len()
    };
    assert!(
        distinct > 3,
        "the bread store must move, not sit at one value: {distinct} distinct levels"
    );
    assert!(
        samples.iter().any(|s| *s > 0.0),
        "somebody must hold bread at some point"
    );
}

/// **Balanced inflow leaves the price alone; a shortage raises it.** The literature's own test, run as two
/// configurations of one village.
#[test]
fn a_balanced_economy_rests_and_a_shortage_does_not() {
    // Balanced: the baker makes more per bake than the eater uses per meal, and the ceiling stops either
    // running away, so the two flows sit level.
    let (mut balanced, _) = village(2.0, 0.5, 4);
    common::run(&mut balanced, 2_400);
    let settled = price(&balanced);
    common::run(&mut balanced, 600);
    let after = price(&balanced);
    assert!(
        (after - settled).abs() < 0.2,
        "a balanced village's price must stay put: {settled:.3} -> {after:.3}"
    );

    // Shortage: the baker makes a tenth of a loaf per bake and the eater still eats one, so consumption
    // exceeds production every tick the two both run.
    let (mut starved, _) = village(0.1, 0.5, 4);
    let before = price(&starved);
    common::run(&mut starved, 2_400);
    let later = price(&starved);
    assert!(
        later > before,
        "an undersupplied good must get dearer: {before:.3} -> {later:.3}"
    );
}

/// **Nobody is blocked.** A villager with an empty store still eats: a good changes what a person prefers,
/// never what they may do. Gating a dominant activity was measured to wreck the village.
#[test]
fn an_empty_store_does_not_stop_anyone() {
    let (mut app, villagers) = village(0.0, 0.5, 3);
    let mut eats = 0u32;
    for _ in 0..600 {
        app.update();
        for message in app
            .world_mut()
            .resource_mut::<Messages<ActivityEnded>>()
            .drain()
        {
            if message.activity == "eat" {
                eats += 1;
            }
        }
    }
    assert!(
        eats > 0,
        "an eater with no bread must still be able to eat ({eats} meals)"
    );
    // And the store never goes negative.
    for entity in &villagers {
        if let Some(stock) = app.world().get::<Stock>(*entity) {
            assert!(
                stock.0.iter().all(|level| *level >= 0.0),
                "stock must never go negative: {:?}",
                stock.0
            );
        }
    }
}

/// Same seed, same choices: the economy adds no draw and iterates by index.
#[test]
fn the_economy_is_deterministic() {
    let run = || {
        let (mut app, villagers) = village(2.0, 0.5, 4);
        common::run(&mut app, 800);
        (total_stock(&app, &villagers), price(&app))
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "two identical runs must agree on stock and price");
}

/// A price never leaves its band, however lopsided the village.
#[test]
fn a_price_stays_inside_its_band() {
    let (mut app, _) = village(0.0, 2.0, 4);
    common::run(&mut app, 4_000);
    let tuning = Tuning::default();
    let p = price(&app);
    assert!(
        (tuning.price_floor..=tuning.price_ceiling).contains(&p),
        "price {p} escaped [{}, {}]",
        tuning.price_floor,
        tuning.price_ceiling
    );
    let _ = Person;
}

/// **A store survives a reload, and an old save is refused rather than misread.**
///
/// `Stock` is the first per-person state added since the save format was written, so
/// `SocietySave::VERSION` moved 1 → 2. A version-1 file must come back as `RestoreError::Version` — that
/// is what `VERSION` exists for, and `restore` already refuses what it does not understand.
#[test]
#[cfg(feature = "serde")]
fn a_store_survives_a_round_trip_and_an_old_save_is_refused() {
    use bevy_society::{PersonKey, SocietySave, VERSION, capture, restore};

    let (mut app, villagers) = village(2.0, 0.5, 3);
    // **A save is keyed by the host's ids.** Without a `PersonKey` a person is left out of the capture
    // entirely (with one `warn!`), which is the crate's documented behaviour and why this fixture has to
    // key its villagers before it can expect a store to come back.
    for (i, entity) in villagers.iter().enumerate() {
        app.world_mut()
            .entity_mut(*entity)
            .insert(PersonKey(i as u64));
    }
    common::run(&mut app, 600);
    let saved: f32 = total_stock(&app, &villagers);
    assert!(saved > 0.0, "the fixture must have baked something to save");

    let save = capture(app.world_mut());
    assert_eq!(save.version, 2, "the format version moved with `Stock`");
    assert_eq!(VERSION, 2);

    let json = serde_json::to_string(&save).expect("a save is plain data");
    let reread: SocietySave = serde_json::from_str(&json).expect("the file is a save");
    // **The store, not the whole struct.** `now_days` is an `f64` that `serde_json` writes with the
    // shortest representation that round-trips *as a decimal*, so a whole-save comparison already fails on
    // the clock and always has; `DoingSave` stores its days as bits for exactly that reason. What this
    // test is for is that `Stock` — the new field — survives.
    assert_eq!(
        reread
            .persons
            .iter()
            .map(|p| p.stock.clone())
            .collect::<Vec<_>>(),
        save.persons
            .iter()
            .map(|p| p.stock.clone())
            .collect::<Vec<_>>(),
        "the goods a person holds must survive JSON exactly"
    );

    // Back into a fresh village of the same shape: the same keys are not present, so every saved person is
    // reported unknown and every live one fresh — but the round trip must not corrupt anything.
    let (mut fresh, fresh_villagers) = village(2.0, 0.5, 3);
    for (i, entity) in fresh_villagers.iter().enumerate() {
        fresh
            .world_mut()
            .entity_mut(*entity)
            .insert(PersonKey(i as u64));
    }
    common::run(&mut fresh, 2);
    let report = restore(fresh.world_mut(), reread).expect("the save applies");
    assert!(
        report.unknown_keys.is_empty() && report.fresh.is_empty(),
        "every key is in both worlds: {report:?}"
    );
    // The store came back, which is the whole point of the version bump.
    let restored: f32 = total_stock(&fresh, &fresh_villagers);
    assert!(
        (restored - saved).abs() < 1e-6,
        "bread must come back exactly: {restored} against {saved}"
    );

    // And a version-1 save is refused with the version error, not read as if it were current.
    let mut old = capture(fresh.world_mut());
    old.version = 1;
    match restore(fresh.world_mut(), old) {
        Err(bevy_society::RestoreError::Version { found, expected }) => {
            assert_eq!((found, expected), (1, VERSION));
        }
        other => panic!("a version-1 save must be refused, got {other:?}"),
    }
}
