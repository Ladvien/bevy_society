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
//! So the test is run on two configurations of the same village: once with production matching consumption,
//! once with the producer starved. The price is **read off the store**, so the fed village's price sits where
//! its store does and the starved one's is dearer — a comparison between villages, not a climb within one.
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

/// **A balanced economy rests and a short one is dearer.** The literature's own test, run as two
/// configurations of one village.
///
/// This used to assert that a starved village's price *rose* from its starting value, which only made sense
/// for a price that integrated a flow from a nominal `1.0`. A price read off the stores is already at the
/// empty-store extreme at tick 0 — the village starts holding nothing — so the comparison has to be between
/// villages, not before and after one. `balanced` is the paper's "inflow equals outflow": its store holds
/// still, so its price does. `starved` bakes a tenth of a loaf per bake, so its store never fills and its
/// price sits near double.
#[test]
fn a_balanced_economy_rests_and_a_shortage_is_dearer() {
    let (mut balanced, _) = village(2.0, 0.5, 4);
    common::run(&mut balanced, 2_400);
    let settled = price(&balanced);
    common::run(&mut balanced, 600);
    let after = price(&balanced);
    assert!(
        (after - settled).abs() < 0.2,
        "a balanced village's price must stay put: {settled:.3} -> {after:.3}"
    );

    let (mut starved, _) = village(0.1, 0.5, 4);
    common::run(&mut starved, 2_400);
    let short = price(&starved);
    assert!(
        short > settled,
        "an undersupplied good must be dearer than a fed one: {short:.3} against {settled:.3}"
    );
    // The shape's own bound: an empty store is worth double the nominal unit, and the band does not clip
    // that (floor 0.25, ceiling 4.0).
    assert!(
        short > 1.5,
        "a village that never fills its store must price near double, not {short:.3}"
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

/// **A store survives a reload, and an old save is upgraded rather than misread.**
///
/// `Stock` is the first per-person state added since the save format was written, so
/// `SocietySave::VERSION` moved 1 → 2; `restore` upgrades a version-1 file (`tests/save_upgrade.rs`).
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
    assert_eq!(save.version, 4, "the format version moved with `Stock`");
    assert_eq!(VERSION, 4);

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

    // A version-1 save is upgraded, not refused: it has no stores, so everyone comes back holding nothing.
    let mut old = capture(fresh.world_mut());
    old.version = 1;
    let report = restore(fresh.world_mut(), old).expect("a version-1 save is upgraded");
    assert_eq!(report.upgraded_from, Some(1));
}

/// **The price book comes back with the stores.** `ctx.price` reads the tick-start snapshot, which is frozen
/// before the tick's own price update, so a restore that left the fresh world's book in place made the first
/// tick after it read a price the uninterrupted run never saw.
#[test]
#[cfg(feature = "serde")]
fn a_restore_puts_the_price_book_back() {
    use bevy_society::{PersonKey, capture, restore};

    let key = |app: &mut App, villagers: &[Entity]| {
        for (i, entity) in villagers.iter().enumerate() {
            app.world_mut()
                .entity_mut(*entity)
                .insert(PersonKey(i as u64));
        }
    };
    let (mut app, villagers) = village(2.0, 0.5, 3);
    key(&mut app, &villagers);
    common::run(&mut app, 600);
    let saved = app.world().resource::<Prices>().0.clone();
    let save = capture(app.world_mut());

    let (mut fresh, fresh_villagers) = village(2.0, 0.5, 3);
    key(&mut fresh, &fresh_villagers);
    assert_ne!(
        fresh.world().resource::<Prices>().0,
        saved,
        "the fixture must differ before the restore, or the test proves nothing"
    );
    restore(fresh.world_mut(), save).expect("the save applies");
    assert_eq!(fresh.world().resource::<Prices>().0, saved);
}

// -------------------------------------------------------------------------------------------------
// A meal is what was on the shelf
// -------------------------------------------------------------------------------------------------

/// A village whose `eat` actually pays a need, so the payout is what is being measured. `bake` produces
/// `bread_per_bake`; the eater asks for a whole loaf.
///
/// `food` does not decay here, so any change in it is the meal's own payout and nothing else.
fn fed_defs(bread_per_bake: f32) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "bake", target: Place("oven"), duration_hours: 0.5,
                 produces: [("bread", {bread_per_bake})],
                 considerations: [(input: Need("food"), curve: Step(0.0))],
                 satisfies: [], interaction: None, barks: []),
                (id: "eat", target: Place("oven"), duration_hours: 0.5,
                 consumes: [("bread", 1.0)],
                 considerations: [(input: Need("food"), curve: Step(0.0))],
                 satisfies: [("food", 1.0)], interaction: None, barks: []),
            ],
        )"#
    ))
    .expect("the definitions parse")
}

/// One eater at an oven, holding `held` loaves and nothing else in the village. No baker, so the store can
/// only fall: whatever the eater pays came from what it walked in with.
fn lone_eater(held: f32) -> (App, Entity) {
    let mut app = common::headless(fed_defs(0.0), common::settings());
    let oven = app
        .world_mut()
        .spawn((
            Name::new("oven"),
            Place { capacity: 8 },
            PlaceTags(vec!["oven".to_owned()]),
            Transform::default(),
        ))
        .id();
    let eater = common::spawn_person(&mut app, "Eater", Vec2::ZERO);
    app.world_mut()
        .entity_mut(eater)
        .insert((Autonomous, Home(oven)));
    // The first update sizes `Needs` and `Stock` from the definitions, so seed after it. A need starts at
    // its `initial` (1.0 by default), so it is zeroed too — otherwise the payout is measured against a
    // ceiling it is already at.
    common::run(&mut app, 2);
    if let Some(mut stock) = app.world_mut().get_mut::<Stock>(eater) {
        stock.0[0] = held;
    }
    if let Some(mut needs) = app.world_mut().get_mut::<bevy_society::Needs>(eater) {
        needs.0[0] = 0.0;
    }
    (app, eater)
}

/// `food` for one person, or `None` before `Needs` exists.
fn food(app: &App, person: Entity) -> Option<f32> {
    app.world()
        .get::<bevy_society::Needs>(person)
        .and_then(|needs| needs.0.first().copied())
}

/// **A meal from an empty shelf feeds nobody.**
///
/// This is the defect the price pass measured and could not fix: the village baked 4.80 loaves a game-day
/// against 21.06 completed meals, so ~77% of its meals were conjugated out of nothing, and it held zero
/// bread for 24.0% of ticks. `satisfies` paid in full regardless of what `Stock::take` found.
///
/// The villager still *eats* — the activity is chosen, the walk happens, the meal takes its ticks. It simply
/// is not fed, which is the difference between this and gating, which was measured to wreck the village.
#[test]
fn a_meal_from_an_empty_shelf_feeds_nobody() {
    let (mut app, eater) = lone_eater(0.0);
    common::run(&mut app, 600);
    let level = food(&app, eater).expect("Needs is sized after the first update");
    assert!(
        level < 1e-3,
        "a meal with no bread must not feed anybody, but food reached {level}"
    );
    // And nobody was blocked: the activity was still chosen, so the store was still debited to nothing.
    let held = app
        .world()
        .get::<Stock>(eater)
        .map(|s| s.get(bevy_society::GoodId(0)))
        .unwrap_or(-1.0);
    assert!(
        held.abs() < 1e-6,
        "the eater must still hold nothing after eating from an empty store, not {held}"
    );
}

/// **A meal with the bread pays exactly what it always did.**
///
/// The guard on the change: a fed village is bit-identical. One whole loaf is an `eat` at `1.0` a game-hour
/// for half an hour, so the payout is `0.5` and the factor must be exactly `1.0`.
#[test]
fn a_fed_meal_pays_in_full() {
    let (mut app, eater) = lone_eater(1.0);
    common::run(&mut app, 1_200);
    let level = food(&app, eater).expect("Needs is sized after the first update");
    assert!(
        level > 0.0,
        "a meal with a loaf must feed its eater, but food stayed at {level}"
    );
    // The store was debited, which is the other half of the same event.
    let held = app
        .world()
        .get::<Stock>(eater)
        .map(|s| s.get(bevy_society::GoodId(0)))
        .unwrap_or(-1.0);
    assert!(
        held < 1e-6,
        "the loaf must have been eaten, but {held} is left"
    );
}

/// **Half a loaf feeds half as much.** Linear in what was there, with no threshold: the crate has no source
/// for a "a meal needs a whole loaf or it does not count" rule, and inventing one would make a partial store
/// worth exactly nothing.
#[test]
fn half_a_loaf_feeds_half() {
    let full = {
        let (mut app, eater) = lone_eater(1.0);
        common::run(&mut app, 1_200);
        food(&app, eater).expect("sized")
    };
    let half = {
        let (mut app, eater) = lone_eater(0.5);
        common::run(&mut app, 1_200);
        food(&app, eater).expect("sized")
    };
    assert!(
        full > 0.1 && half > 1e-3,
        "both meals must feed something for this to measure anything: {full} and {half}"
    );
    let ratio = half / full;
    assert!(
        (ratio - 0.5).abs() < 0.06,
        "half a loaf must feed about half as much, not {ratio:.3} ({half} against {full})"
    );
}

/// **An activity that consumes nothing is untouched.** `satisfies` scales by the store only where the
/// activity declared a `consumes`; every other activity pays exactly what it paid before, so this change
/// stays inside one activity's arithmetic.
#[test]
fn an_activity_that_consumes_nothing_is_unaffected() {
    let defs = SocietyDefs::from_ron(
        r#"(
            goods: [(id: "bread")],
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "rest", target: Place("oven"), duration_hours: 0.5,
                 considerations: [(input: Need("food"), curve: Step(0.0))],
                 satisfies: [("food", 1.0)], interaction: None, barks: []),
            ],
        )"#,
    )
    .expect("the definitions parse");
    let mut app = common::headless(defs, common::settings());
    let oven = app
        .world_mut()
        .spawn((
            Name::new("oven"),
            Place { capacity: 8 },
            PlaceTags(vec!["oven".to_owned()]),
            Transform::default(),
        ))
        .id();
    let person = common::spawn_person(&mut app, "Resting", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Autonomous, Home(oven)));
    common::run(&mut app, 2);
    if let Some(mut needs) = app.world_mut().get_mut::<bevy_society::Needs>(person) {
        needs.0[0] = 0.0;
    }
    common::run(&mut app, 1_200);
    let level = food(&app, person).expect("sized");
    assert!(
        level > 0.1,
        "an activity with no `consumes` must pay in full from an empty store, but food is {level}"
    );
}
