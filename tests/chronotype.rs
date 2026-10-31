//! **Body clocks.** A person has their own clock: `Chronotype` is how many hours it runs behind the wall clock,
//! `Input::BodyHour` reads it, and an activity marked `unavailable` (the starter pack's `sleep`) cannot be walked up to
//! once its person has arrived at it. The research is `docs/research/chronotype.md`; the evidence that a village stops going
//! to bed in one hour is the harness's (`docs/loop/runs/2026-10-31-02/`). These tests pin the mechanisms:
//!
//! 1. the arithmetic and the sampler (a population like Fischer et al. 2017's, deterministic, with no draw of its own);
//! 2. a lark, a person with no offset and an owl each go to bed at the wall hour their own clock says, and the person with
//!    no offset goes at 22:00 exactly as the pack always had it;
//! 3. a sleeper is not a target, a walk toward someone who falls asleep ends, and with the flag off both happen;
//! 4. the offset survives a save and an authored placement, and a file from before it restores as no offset.
mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_emerge_rng::seeded;
use bevy_society::{
    ActivityEnded, ActivityStarted, Autonomous, Chronotype, DefsFile, Doing, DoingTarget, Home,
    Needs, Place, PlacementFile, SocietyClock, SocietyDefs,
};
use common::{headless, run, settings, spawn_person};

/// Game-hours are ten seconds at the default 240 s/day, which is 640 updates at 64 Hz.
const UPDATES_PER_HOUR: usize = 640;

// ---- the arithmetic and the sampler ---------------------------------------------------------------------------------

#[test]
fn the_body_clock_runs_behind_the_wall_clock_and_wraps_both_ways() {
    assert_eq!(Chronotype(0.0).body_hour(13.5), 13.5, "no offset is the wall clock");
    assert_eq!(Chronotype(2.0).body_hour(0.0), 22.0, "an owl is two hours behind: 00:00 is 22:00 to them");
    assert_eq!(Chronotype(-3.0).body_hour(23.0), 2.0, "a lark is ahead and wraps past midnight");
    for offset in [-9.5, -2.0, 0.0, 4.25, 11.0] {
        for tenth in 0..240 {
            let body = Chronotype(offset).body_hour(tenth as f32 / 10.0);
            assert!((0.0..24.0).contains(&body), "{offset} at {tenth}: {body}");
        }
    }
}

#[test]
fn a_sampled_population_is_normal_about_the_villages_clock_and_cut_at_three_deviations() {
    let sd = Chronotype::POPULATION_SD_HOURS;
    let mut rng = seeded(41);
    let draws: Vec<f32> = (0..20_000)
        .map(|_| Chronotype::sample(&mut rng, sd).0)
        .collect();
    let mean = draws.iter().sum::<f32>() / draws.len() as f32;
    let spread = (draws.iter().map(|d| (d - mean).powi(2)).sum::<f32>() / draws.len() as f32).sqrt();
    assert!(mean.abs() < 0.05, "mean {mean}");
    assert!((spread / sd - 1.0).abs() < 0.05, "sd {spread} against {sd}");
    assert!(draws.iter().all(|d| d.abs() <= 3.0 * sd), "the tails are cut at 3 sd");
    // Larks and owls both exist: a quarter of a normal is beyond 0.67 sd on each side.
    let owls = draws.iter().filter(|d| **d > 0.67 * sd).count() as f32 / draws.len() as f32;
    assert!((owls - 0.25).abs() < 0.02, "a quarter are owls, got {owls}");
    // Same seed, same person; zero (and a nonsense spread) is the village's clock; the crate draws nothing of its own.
    assert_eq!(Chronotype::sample(&mut seeded(7), sd), Chronotype::sample(&mut seeded(7), sd));
    assert_eq!(Chronotype::sample(&mut seeded(7), 0.0), Chronotype(0.0));
    assert_eq!(Chronotype::sample(&mut seeded(7), f32::NAN), Chronotype(0.0));
}

// ---- going to bed by the body's clock --------------------------------------------------------------------------------

/// The starter pack with `sleep` and nothing else, so a person's day is the one rule under test.
fn sleep_only() -> SocietyDefs {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    file.activities.retain(|a| a.id == "sleep");
    SocietyDefs::compile(file).expect("the pack compiles")
}

/// Three people at one bed, a lark (-3 h), nobody in particular and an owl (+3 h), all tired from 18:00. Returns the
/// game-hour (counted from midnight of day 0) at which each first started to sleep.
fn bedtimes(seed: u64) -> Vec<f64> {
    let rest = sleep_only().need_id("rest").expect("the starter pack has rest").0 as usize;
    let mut app = headless(
        sleep_only(),
        bevy_society::SocietySettings {
            start_hour: 18.0,
            seed,
            ..settings()
        },
    );
    let bed = app
        .world_mut()
        .spawn((Name::new("bed"), Place { capacity: 8 }, Transform::default()))
        .id();
    let people: Vec<Entity> = [-3.0, 0.0, 3.0]
        .into_iter()
        .enumerate()
        .map(|(i, offset)| {
            let person = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.3, 0.0));
            app.world_mut()
                .entity_mut(person)
                .insert((Autonomous, Home(bed), Chronotype(offset)));
            person
        })
        .collect();
    // The first updates size every person's `Needs`; a rested person scores no sleep, so tire them.
    run(&mut app, 2);
    for &person in &people {
        app.world_mut().get_mut::<Needs>(person).unwrap().0[rest] = 0.2;
    }
    let mut first: Vec<Option<f64>> = vec![None; people.len()];
    // 18:00 to 03:30 the next morning.
    for _ in 0..(9 * UPDATES_PER_HOUR + UPDATES_PER_HOUR / 2) {
        app.update();
        let now = app.world().resource::<SocietyClock>().now_days;
        let started: Vec<ActivityStarted> = app
            .world_mut()
            .resource_mut::<Messages<ActivityStarted>>()
            .drain()
            .collect();
        for message in started.into_iter().filter(|m| m.activity == "sleep") {
            if let Some(i) = people.iter().position(|p| *p == message.person) {
                first[i].get_or_insert(now * 24.0);
            }
        }
    }
    first
        .into_iter()
        .map(|hour| hour.expect("every tired person goes to bed within the night"))
        .collect()
}

/// **Each person goes to bed when their own clock reaches 22:00.** The pack's window is `Hours(22, 6)` on `BodyHour`: a lark
/// at -3 h finds it open at 19:00, a person with no offset at 22:00 (the hour it always was), an owl at 01:00. A person
/// re-decides when its idle hour ends, so a bedtime lands within an hour after the window opens, never before it.
#[test]
fn a_lark_a_plain_person_and_an_owl_go_to_bed_by_their_own_clocks() {
    let slept = bedtimes(5);
    let (lark, plain, owl) = (slept[0], slept[1], slept[2]);
    for (name, got, opens) in [("lark", lark, 19.0), ("plain", plain, 22.0), ("owl", owl, 25.0)] {
        assert!(
            (opens..opens + 1.1).contains(&got),
            "the {name} began sleeping at hour {got:.2}, the window opens at {opens}"
        );
    }
    assert!(lark + 2.0 < plain && plain + 2.0 < owl, "{slept:?}");
}

#[test]
fn bedtimes_are_the_same_for_the_same_seed() {
    assert_eq!(bedtimes(5), bedtimes(5));
}

// ---- a sleeper cannot be approached --------------------------------------------------------------------------------

/// `nap` is the pack's `sleep` shrunk to what the test needs, `greet` a `Person` activity open to anyone with `social` left.
/// `unavailable` is the one thing that differs between the two arms.
fn naps_and_greetings(unavailable: bool) -> SocietyDefs {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    file.activities.clear();
    let extra = DefsFile::from_ron(&format!(
        r#"(activities: [
            (id: "nap", target: Home, duration_hours: 8.0, unavailable: {unavailable},
             considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
             satisfies: [("rest", 0.1)], interaction: None, barks: []),
            (id: "greet", target: Person, duration_hours: 0.2,
             considerations: [(input: Need("social"), curve: Linear(1.0, 0.0))],
             satisfies: [], interaction: Some("compliment"), barks: []),
        ])"#
    ))
    .expect("the two activities parse");
    file.activities = extra.activities;
    SocietyDefs::compile(file).expect("the pack compiles")
}

/// A sleeper (`rest` 0, no wish to greet) at the table, then a visitor with `social` to spend `gap` metres away. Returns the app,
/// the sleeper, the visitor and the `nap` activity.
fn sleeper_and_visitor(
    unavailable: bool,
    gap: f32,
) -> (App, Entity, Entity, bevy_society::ActivityId) {
    let defs = naps_and_greetings(unavailable);
    let (rest, social) = (
        defs.need_id("rest").unwrap().0 as usize,
        defs.need_id("social").unwrap().0 as usize,
    );
    let nap = defs.activity_id("nap").unwrap();
    let mut app = headless(defs, settings());
    let table = app
        .world_mut()
        .spawn((Name::new("table"), Place { capacity: 8 }, Transform::default()))
        .id();
    let sleeper = spawn_person(&mut app, "Sleeper", Vec2::new(0.2, 0.0));
    app.world_mut()
        .entity_mut(sleeper)
        .insert((Autonomous, Home(table)));
    run(&mut app, 2);
    {
        let mut needs = app.world_mut().get_mut::<Needs>(sleeper).unwrap();
        needs.0[rest] = 0.0;
        needs.0[social] = 0.0;
    }
    run(&mut app, 8);
    let doing = app.world().get::<Doing>(sleeper).expect("the sleeper chose");
    assert_eq!(doing.activity, nap);
    assert!(doing.arrived_day.is_some(), "the sleeper is in bed");
    let visitor = spawn_person(&mut app, "Visitor", Vec2::new(gap, 0.0));
    app.world_mut()
        .entity_mut(visitor)
        .insert((Autonomous, Home(table)));
    run(&mut app, 2);
    {
        let mut needs = app.world_mut().get_mut::<Needs>(visitor).unwrap();
        needs.0[rest] = 1.0;
        needs.0[social] = 1.0;
    }
    (app, sleeper, visitor, nap)
}

fn greetings_of(app: &mut App, who: Entity, of: Entity, updates: usize) -> usize {
    let mut count = 0;
    for _ in 0..updates {
        app.update();
        count += app
            .world_mut()
            .resource_mut::<Messages<ActivityStarted>>()
            .drain()
            .filter(|m| m.person == who && m.activity == "greet" && m.target == Some(of))
            .count();
    }
    count
}

/// **Nobody greets a sleeper.** With the flag, the visitor standing beside a sleeping person finds nobody to greet in two
/// game-hours; without it the visitor greets the sleeper (the failure this guards against: a village that
/// compliments its sleepers once people keep their own hours).
#[test]
fn a_person_asleep_is_nobody_s_target_only_when_their_activity_is_unavailable() {
    let (mut with_flag, sleeper, visitor, _) = sleeper_and_visitor(true, 0.3);
    assert_eq!(greetings_of(&mut with_flag, visitor, sleeper, 2 * UPDATES_PER_HOUR), 0);
    let (mut without, sleeper, visitor, _) = sleeper_and_visitor(false, 0.3);
    assert!(
        greetings_of(&mut without, visitor, sleeper, 2 * UPDATES_PER_HOUR) > 0,
        "an available sleeper is a target like anyone"
    );
}

/// **A walk toward someone who falls asleep ends, not completed.** The visitor sets off toward an awake person eight metres
/// away; when that person is put to sleep mid-walk the visitor's `greet` ends interrupted. Without the flag it walks on.
#[test]
fn a_walk_toward_someone_who_falls_asleep_ends_interrupted() {
    for (unavailable, ends) in [(true, true), (false, false)] {
        let defs = naps_and_greetings(unavailable);
        let (rest, social) = (
            defs.need_id("rest").unwrap().0 as usize,
            defs.need_id("social").unwrap().0 as usize,
        );
        let nap = defs.activity_id("nap").unwrap();
        let mut app = headless(defs, settings());
        let table = app
            .world_mut()
            .spawn((Name::new("table"), Place { capacity: 8 }, Transform::default()))
            .id();
        let host = spawn_person(&mut app, "Host", Vec2::new(8.0, 0.0));
        let visitor = spawn_person(&mut app, "Visitor", Vec2::ZERO);
        for person in [host, visitor] {
            app.world_mut()
                .entity_mut(person)
                .insert((Autonomous, Home(table)));
        }
        run(&mut app, 2);
        // The host has no wish to do anything (rested, and nothing to spend); the visitor wants company.
        for (person, social_level) in [(host, 0.0), (visitor, 1.0)] {
            let mut needs = app.world_mut().get_mut::<Needs>(person).unwrap();
            needs.0[rest] = 1.0;
            needs.0[social] = social_level;
        }
        run(&mut app, 40);
        let walking = app.world().get::<Doing>(visitor).expect("the visitor set off");
        assert_eq!(walking.target, DoingTarget::Person(host));
        assert!(walking.arrived_day.is_none(), "still on the way");
        // The host falls asleep: its `Doing` is `nap`, arrived.
        let now = app.world().resource::<SocietyClock>().now_days;
        app.world_mut().entity_mut(host).insert(Doing {
            activity: nap,
            target: DoingTarget::Place(table),
            started_day: now,
            arrived_day: Some(now),
        });
        let mut interrupted = false;
        for _ in 0..40 {
            app.update();
            interrupted |= app
                .world_mut()
                .resource_mut::<Messages<ActivityEnded>>()
                .drain()
                .any(|m| m.person == visitor && m.activity == "greet" && !m.completed);
        }
        assert_eq!(interrupted, ends, "unavailable = {unavailable}");
    }
}

// ---- authoring ------------------------------------------------------------------------------------------------------

#[test]
fn an_hours_curve_takes_the_social_or_the_body_clock_and_nothing_else() {
    let with = |input: &str| {
        let mut file = DefsFile::starter().unwrap();
        file.activities.clear();
        file.activities.extend(
            DefsFile::from_ron(&format!(
                r#"(activities: [(id: "x", target: Itself, duration_hours: 1.0,
                    considerations: [(input: {input}, curve: Hours(22.0, 6.0))],
                    interaction: None)])"#
            ))
            .unwrap()
            .activities,
        );
        SocietyDefs::compile(file)
    };
    assert!(with("Hour").is_ok());
    assert!(with("BodyHour").is_ok());
    let err = with(r#"Need("rest")"#).unwrap_err().to_string();
    assert!(
        err.contains("Hour or BodyHour"),
        "the refusal names both inputs: {err}"
    );
}

#[test]
fn an_authored_placement_carries_the_offset_in_and_out() {
    let ron = r#"(
        places: [(id: "cottage", at: (0.0, 0.0), capacity: 2)],
        persons: [(name: "Ada", home: Some("cottage"), chronotype_hours: -2.5), (name: "Bram", home: Some("cottage"))],
    )"#;
    let mut app = headless(DefsFile::starter().and_then(SocietyDefs::compile).unwrap(), settings());
    let file = PlacementFile::from_ron(ron).expect("parses");
    assert_eq!(file.persons[0].chronotype_hours, -2.5);
    assert_eq!(file.persons[1].chronotype_hours, 0.0, "left out is the village's clock");
    let placed = file.spawn(app.world_mut()).expect("spawns");
    let (ada, bram) = (placed.person("Ada").unwrap(), placed.person("Bram").unwrap());
    assert_eq!(app.world().get::<Chronotype>(ada), Some(&Chronotype(-2.5)));
    assert_eq!(app.world().get::<Chronotype>(bram), Some(&Chronotype(0.0)));
    let captured = PlacementFile::capture(app.world_mut()).expect("captures");
    assert_eq!(captured.persons.iter().find(|p| p.name == "Ada").unwrap().chronotype_hours, -2.5);
    assert_eq!(PlacementFile::from_ron(&captured.to_ron().unwrap()).unwrap(), captured);
    // A day is not a clock: an offset beyond half of one is refused, naming the person.
    let bad = ron.replace("-2.5", "13.0");
    let err = PlacementFile::from_ron(&bad).unwrap().spawn(app.world_mut()).unwrap_err();
    assert!(err.to_string().contains("Ada") && err.to_string().contains("chronotype_hours"), "{err}");
}

// ---- a save keeps it ------------------------------------------------------------------------------------------------

#[cfg(feature = "serde")]
mod saved {
    use super::*;
    use bevy_society::{PersonKey, capture, restore};

    fn keyed_village() -> (App, Vec<Entity>) {
        let mut app = common::village::village(common::village::village_defs(), 3, 6);
        let mut people: Vec<(u32, Entity)> = {
            let mut query = app
                .world_mut()
                .query_filtered::<(Entity, &bevy_society::PersonId), With<bevy_society::Person>>();
            query.iter(app.world()).map(|(e, id)| (id.0, e)).collect()
        };
        people.sort_by_key(|&(id, _)| id);
        let people: Vec<Entity> = people.into_iter().map(|(_, e)| e).collect();
        for (key, &person) in people.iter().enumerate() {
            app.world_mut().entity_mut(person).insert(PersonKey(key as u64));
        }
        run(&mut app, 3);
        (app, people)
    }

    #[test]
    fn a_save_keeps_each_persons_clock_and_a_file_from_before_it_restores_as_none() {
        let (mut app, people) = keyed_village();
        app.world_mut().entity_mut(people[0]).insert(Chronotype(-2.5));
        app.world_mut().entity_mut(people[3]).insert(Chronotype(1.75));
        let save = capture(app.world_mut());
        let offsets: Vec<f32> = save.persons.iter().map(|p| p.chronotype.0).collect();
        assert_eq!(offsets, vec![-2.5, 0.0, 0.0, 1.75, 0.0, 0.0]);

        // Into a fresh village whose people all keep the village's clock.
        let (mut fresh, fresh_people) = keyed_village();
        let json = serde_json::to_string(&save).expect("writes");
        let loaded: bevy_society::SocietySave = serde_json::from_str(&json).expect("reads");
        restore(fresh.world_mut(), loaded).expect("restores");
        let back: Vec<f32> = fresh_people
            .iter()
            .map(|&p| fresh.world().get::<Chronotype>(p).map_or(f32::NAN, |c| c.0))
            .collect();
        assert_eq!(back, offsets);

        // A file written before the field existed has no `chronotype` key and restores as no offset.
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for person in value["persons"].as_array_mut().unwrap() {
            person.as_object_mut().unwrap().remove("chronotype");
        }
        let old: bevy_society::SocietySave = serde_json::from_value(value).expect("an old file reads");
        assert!(old.persons.iter().all(|p| p.chronotype == Chronotype(0.0)));
    }
}
