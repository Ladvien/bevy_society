//! **Save and restore a whole society** (issue #7).
//!
//! A host autosaves and continues. For the reload to be invisible, four things have to hold, and
//! this file is one test per thing:
//!
//! 1. **Round trip.** `capture` → JSON → `restore` → `capture` is the same save. A field read but
//!    not written, or written but not read, shows up here as a difference.
//! 2. **Bit-identical continuation.** The 100 ticks after a restore are the ticks the uninterrupted
//!    run would have taken. Everything the models keep has to line up — the RNG's word position, the
//!    clock, the tick accumulator, every trigger cooldown, the tie sort order — not merely look
//!    right.
//! 3. **An unknown key is reported, not fatal**, and a person the host never keyed is not saved.
//! 4. **A save whose definitions have moved is refused**, leaving the world untouched, rather than
//!    writing today's needs into yesterday's opinion slots.
//!
//! The village is `common::village`'s — the same society `tests/determinism.rs` and
//! `tests/friendship_bound.rs` measure — and the hash is `common::hash`'s, the instrument
//! `determinism.rs` uses, so the two claims are measured with one ruler.
#![cfg(feature = "serde")]

mod common;

use bevy::prelude::*;
use bevy_society::{
    DoingTargetSave, Person, PersonId, PersonKey, Place, PlaceKey, RestoreError, SocietyClock,
    SocietySave, capture, restore,
};
use common::capture::{LogCapture, log_plugin};
use common::hash::hash;

/// The villagers and places of one village, in spawn order.
struct Keyed {
    /// Villager entities, ascending by key.
    persons: Vec<Entity>,
    /// Place entities, ascending by key.
    places: Vec<Entity>,
}

/// Keys every person and place a village spawned, in spawn order.
///
/// `common::village` is shared with two tests whose measured bars this feature must not disturb, so
/// the keys are attached here instead: `PersonKey` and `PlaceKey` are inert components no model
/// reads, and the crate never writes one.
fn key_the_village(app: &mut App) -> Keyed {
    let mut persons: Vec<(u32, Entity)> = {
        let mut query = app
            .world_mut()
            .query_filtered::<(Entity, &PersonId), With<Person>>();
        query.iter(app.world()).map(|(e, id)| (id.0, e)).collect()
    };
    persons.sort_by_key(|&(id, _)| id);
    let mut keyed_persons = Vec::with_capacity(persons.len());
    for (index, (_, entity)) in persons.iter().enumerate() {
        app.world_mut()
            .entity_mut(*entity)
            .insert(PersonKey(index as u64));
        keyed_persons.push(*entity);
    }

    let mut places: Vec<Entity> = {
        let mut query = app.world_mut().query_filtered::<Entity, With<Place>>();
        query.iter(app.world()).collect()
    };
    // Spawn order, which is `PLACES` order and therefore stable.
    for (index, entity) in places.iter().enumerate() {
        app.world_mut()
            .entity_mut(*entity)
            .insert(PlaceKey(index as u64));
    }
    places.sort_by_key(|entity| {
        app.world()
            .get::<PlaceKey>(*entity)
            .map_or(u64::MAX, |key| key.0)
    });

    Keyed {
        persons: keyed_persons,
        places,
    }
}

/// A keyed village that has taken its first society tick, so `Startup` has seeded the clock and the
/// RNG and the social fields exist.
///
/// A restore has to come **after** that: `setup_society` writes the clock's start hour and a fresh
/// ChaCha8 stream, so a restore applied before it would be overwritten, and `SocialFieldPlugin`
/// inserts the grids through `Commands` on its first fixed step, so a capture before that records no
/// fields at all. A host loads a save on a frame; so does this.
fn seeded_village(seed: u64) -> (App, Keyed) {
    let mut app = common::village::village(common::village::village_defs(), seed, 24);
    let keyed = key_the_village(&mut app);
    run_ticks(&mut app, 1);
    (app, keyed)
}

/// A keyed village run for `ticks` society ticks, with the save it produced.
fn saved_after(seed: u64, ticks: u64) -> (App, Keyed, SocietySave) {
    let (mut app, keyed) = seeded_village(seed);
    run_ticks(&mut app, ticks);
    let save = capture(app.world_mut());
    (app, keyed, save)
}

/// Advances `app` until the society clock has taken exactly `ticks` more ticks.
///
/// Counting ticks rather than updates is what makes the two apps of a continuation comparable. The
/// village's settings are `ManualDuration(0.25)` against a 1/64 fixed step, so one update is sixteen
/// fixed steps, and sixteen fixed steps are exactly `tick_secs` of 0.25 — the accumulator oversteps
/// by zero and every update is exactly one tick, in both apps.
fn run_ticks(app: &mut App, ticks: u64) {
    let target = app.world().resource::<SocietyClock>().tick + ticks;
    let mut updates = 0u32;
    while app.world().resource::<SocietyClock>().tick < target {
        app.update();
        updates += 1;
        assert!(
            updates < 200_000,
            "the society clock never reached tick {target}; it stopped at {}",
            app.world().resource::<SocietyClock>().tick
        );
    }
}

/// Every person's `Transform` and `GlobalTransform`, in key order.
///
/// A save carries no position: on the host's ground the host owns them, and `Locomotion::Host` is
/// the contract that says so. A test that compares two runs therefore has to stand the reloaded
/// people where the saved run left them, exactly as a host does.
type Pose = (Transform, GlobalTransform);

/// The poses of every person, in key order.
fn poses(app: &App, keyed: &Keyed) -> Vec<Pose> {
    keyed
        .persons
        .iter()
        .map(|&entity| {
            (
                app.world()
                    .get::<Transform>(entity)
                    .copied()
                    .unwrap_or_default(),
                app.world()
                    .get::<GlobalTransform>(entity)
                    .copied()
                    .unwrap_or_default(),
            )
        })
        .collect()
}

/// Writes saved poses back, so a reloaded village stands where the saved one stood.
fn set_poses(app: &mut App, keyed: &Keyed, saved: &[Pose]) {
    for (&entity, &(transform, global)) in keyed.persons.iter().zip(saved) {
        if let Ok(mut em) = app.world_mut().get_entity_mut(entity) {
            em.insert((transform, global));
        }
    }
}

#[test]
fn a_round_trip_through_json_gives_the_same_save() {
    let (source, source_keyed, save) = saved_after(7, 200);
    let json = serde_json::to_string(&save).expect("a save is plain data");
    let reread: SocietySave = serde_json::from_str(&json).expect("the JSON is a save");
    assert_eq!(reread, save, "the save survives its own file format");

    // One activity directed at a keyed place, so `PlaceKey` is exercised rather than merely stored:
    // a `Doing` whose place could not be keyed would have been saved as no activity at all.
    let place_keys: Vec<u64> = source_keyed
        .places
        .iter()
        .enumerate()
        .map(|(index, _)| index as u64)
        .collect();
    assert!(
        save.persons.iter().any(|person| matches!(
            person.doing.map(|doing| doing.target),
            Some(DoingTargetSave::Place(key)) if place_keys.contains(&key)
        )),
        "some villager's running activity is at a keyed place"
    );

    // The world the host reloads into: the same village, standing where the saved one stood.
    let (mut app, keyed) = seeded_village(7);
    set_poses(&mut app, &keyed, &poses(&source, &source_keyed));

    let report = restore(app.world_mut(), reread).expect("the save applies");
    assert!(
        report.unknown_keys.is_empty(),
        "every saved key was in the world: {:?}",
        report.unknown_keys
    );
    assert!(
        report.fresh.is_empty(),
        "every person in the world was in the save: {:?}",
        report.fresh
    );
    assert_eq!(
        report.dropped_activities, 0,
        "every saved activity resolved to its place or person"
    );

    let recaptured = capture(app.world_mut());
    assert_eq!(
        recaptured, save,
        "capture → restore → capture gives the same save, so every saved field is also read back"
    );
}

#[test]
fn the_hundred_ticks_after_a_restore_are_the_uninterrupted_ones() {
    // App A: 2 200 ticks, a save, then 100 more.
    let (mut uninterrupted, keyed, save) = saved_after(7, 2_200);
    let clock_at_save = *uninterrupted.world().resource::<SocietyClock>();
    let poses_at_save = poses(&uninterrupted, &keyed);
    let expected = {
        run_ticks(&mut uninterrupted, 100);
        hash(&uninterrupted)
    };
    assert_ne!(expected, 0, "the hash covers a non-empty society");

    // App B: the same village, A's positions, then the save applied — and 100 ticks.
    let (mut reloaded, reloaded_keyed) = seeded_village(7);
    set_poses(&mut reloaded, &reloaded_keyed, &poses_at_save);
    restore(reloaded.world_mut(), save).expect("the save applies");
    assert_eq!(
        *reloaded.world().resource::<SocietyClock>(),
        clock_at_save,
        "the clock comes back at the instant the save was taken"
    );
    run_ticks(&mut reloaded, 100);

    assert_eq!(
        hash(&reloaded),
        expected,
        "100 ticks after a restore are bit-identical to the 100 that would have followed without one"
    );
}

/// A small headless society with log capture, for the warning assertions.
///
/// The `tracing` subscriber is process-global and settable once, so this binary may build only one
/// capturing app; both warnings are asserted from this one run.
fn capturing_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        log_plugin(),
    ))
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(0.25),
    ))
    .insert_resource(bevy_society::SocietySettings {
        seed: 7,
        ..bevy_society::SocietySettings::default()
    })
    .insert_resource(common::village::village_defs());
    let plugins = bevy_society::SocietyPlugins.build();
    #[cfg(feature = "dialogue")]
    let plugins = plugins.disable::<bevy_society::DialoguePlugin>();
    app.add_plugins(plugins);
    app.finish();
    app.cleanup();
    app
}

#[test]
fn an_unkeyed_person_is_not_saved_and_an_unknown_key_is_reported() {
    let mut app = capturing_app();
    app.update();
    let home = app
        .world_mut()
        .spawn((
            Name::new("well"),
            Transform::default(),
            Place { capacity: 8 },
            PlaceKey(1),
        ))
        .id();
    let mut keyed = Vec::new();
    for i in 0..3u64 {
        let entity = app
            .world_mut()
            .spawn((
                Person,
                Name::new(format!("P{i}")),
                bevy_society::Home(home),
                PersonKey(100 + i),
                Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            ))
            .id();
        keyed.push(entity);
    }
    // One person the host forgot to key.
    let forgotten = app
        .world_mut()
        .spawn((
            Person,
            Name::new("Forgotten"),
            Transform::from_xyz(-3.0, 0.0, 0.0),
        ))
        .id();

    run_ticks(&mut app, 20);
    let save = capture(app.world_mut());
    assert_eq!(
        save.persons.len(),
        3,
        "only the three keyed persons are saved"
    );
    assert!(
        save.persons.iter().all(|person| person.key >= 100),
        "the saved persons are the keyed ones"
    );
    let warnings = app
        .world()
        .resource::<LogCapture>()
        .containing("were not saved");
    assert_eq!(
        warnings.len(),
        1,
        "one warning per capture names how many persons were left out: {warnings:?}"
    );
    assert!(
        warnings[0].contains('1'),
        "the warning names the count: {}",
        warnings[0]
    );

    // A save that names a person the world does not have.
    let mut with_ghost = save.clone();
    let mut ghost = with_ghost
        .persons
        .first()
        .cloned()
        .expect("the save has persons");
    ghost.key = 9_999;
    with_ghost.persons.push(ghost);
    let report = restore(app.world_mut(), with_ghost).expect("an unknown key is not an error");
    assert_eq!(
        report.unknown_keys,
        vec![9_999],
        "the unknown key is reported"
    );
    let key_warnings = app
        .world()
        .resource::<LogCapture>()
        .containing("no person in the world");
    assert!(
        key_warnings.iter().any(|line| line.contains("9999")),
        "the unknown key is also warned about: {key_warnings:?}"
    );
    assert_eq!(
        report.fresh,
        vec![forgotten],
        "the person the save does not know is left fresh and reported"
    );
    let fresh_id = app
        .world()
        .get::<PersonId>(forgotten)
        .map(|id| id.0)
        .expect("a fresh person still has an id");
    assert!(
        save.persons
            .iter()
            .all(|person| person.person_id < fresh_id),
        "a fresh person's id comes after every restored one"
    );
    let restored_ids: Vec<u32> = keyed
        .iter()
        .filter_map(|&entity| app.world().get::<PersonId>(entity).map(|id| id.0))
        .collect();
    assert_eq!(
        restored_ids,
        save.persons
            .iter()
            .map(|person| person.person_id)
            .collect::<Vec<_>>(),
        "each restored person has the id the save gave it"
    );

    // The index is what every simulation loop walks, so a person left out of it would silently stop
    // taking part in the society. The unkeyed person the save never knew is in it, at the end.
    let index = app.world().resource::<bevy_society::PersonIndex>();
    let indexed: Vec<Entity> = index.sorted.iter().map(|&(_, entity)| entity).collect();
    assert_eq!(
        indexed.len(),
        4,
        "all four persons are indexed, including the one the save never knew"
    );
    assert!(
        indexed.contains(&forgotten),
        "the person the save does not know still takes part in the society"
    );
    assert_eq!(
        indexed.last().copied(),
        Some(forgotten),
        "its id is after every restored one, so it sorts last"
    );
    assert_eq!(
        app.world().resource::<bevy_society::NextPersonId>().0,
        fresh_id + 1,
        "the next id is past every id in use"
    );
}

#[test]
fn a_renamed_need_is_refused_and_leaves_the_world_alone() {
    let (source, source_keyed, save) = saved_after(7, 200);
    let mut renamed = save;
    renamed.names.needs[0] = "sustenance".to_owned();

    let (mut app, keyed) = seeded_village(7);
    set_poses(&mut app, &keyed, &poses(&source, &source_keyed));
    let before = capture(app.world_mut());
    let error = restore(app.world_mut(), renamed).expect_err("a moved need list is refused");
    assert_eq!(
        error,
        RestoreError::DefsMismatch { kind: "needs" },
        "the error names the list that moved"
    );
    assert_eq!(
        capture(app.world_mut()),
        before,
        "a refused save writes nothing at all"
    );
}
