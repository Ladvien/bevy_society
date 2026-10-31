//! **A refusal is remembered** (`Tuning::rebuff_memory_hours`).
//!
//! Excluded people do not seek reconnection with the one who excluded them but with new sources of
//! affiliation (Maner, DeWall, Baumeister & Schaller 2007, DOI 10.1037/0022-3514.92.1.42). Without the
//! memory a person turned down by the nearest neighbour asks the nearest neighbour again at the next
//! decision, because nothing in the decision remembers who said no.

mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, InteractionResolved, SocietyDefs};
use common::{headless, spawn_person};

/// One affiliative activity, `chat`, and one harmful one, `snub`, each scored only by how close the other is.
/// Every exchange is refused or accepted by `base`, which the acceptance draw clamps to `[0.02, 0.98]`.
fn defs(base: f32, harmful: bool) -> SocietyDefs {
    let activity = if harmful {
        r#"(
            id: "snub",
            target: Person,
            duration_hours: 0.1,
            considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
            interaction: Some("insult"),
        )"#
    } else {
        r#"(
            id: "chat",
            target: Person,
            duration_hours: 0.1,
            considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
            interaction: Some("chat"),
        )"#
    };
    let ron = format!(
        r#"(
            topics: [(id: "kindness")],
            deeds: [(id: "insult", topic: "kindness", harm: -0.4)],
            activities: [{activity}],
            interactions: [
                (id: "chat", accept: (base: {base})),
                (id: "insult", accept: (base: {base}), deed: Some("insult")),
            ],
        )"#
    );
    SocietyDefs::from_ron(&ron).expect("the definitions parse")
}

/// An `Autonomous` person with neighbours at 1 m, 2 m ... along x, and every exchange the actor starts over
/// `ticks` ticks (64 ticks are one game-day at one second a day).
fn run(
    base: f32,
    harmful: bool,
    memory: Option<f32>,
    neighbours: usize,
    ticks: usize,
) -> (Vec<InteractionResolved>, Entity, Vec<Entity>) {
    let mut settings = common::settings();
    settings.seconds_per_day = 1.0;
    settings.tuning.rebuff_memory_hours = memory;
    settings.tuning.retaliation_refractory_hours = None;
    let mut app = headless(defs(base, harmful), settings);
    let actor = spawn_person(&mut app, "A", Vec2::ZERO);
    let others: Vec<Entity> = (0..neighbours)
        .map(|i| spawn_person(&mut app, &format!("N{i}"), Vec2::new(1.0 + i as f32, 0.0)))
        .collect();
    app.world_mut().entity_mut(actor).insert(Autonomous);
    let mut resolved = Vec::new();
    for _ in 0..ticks {
        app.update();
        resolved.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
                .drain()
                .filter(|r| r.actor == actor),
        );
    }
    (resolved, actor, others)
}

#[test]
fn without_the_memory_the_person_who_refused_is_asked_again_and_again() {
    let (resolved, _, others) = run(-1.0, false, None, 1, 40);
    assert!(
        resolved.len() >= 4,
        "expected a standing request, saw {resolved:?}"
    );
    assert!(resolved[0].target == others[0] && !resolved[0].accepted);
    assert!(resolved.iter().all(|r| r.target == others[0]));
}

#[test]
fn a_refusal_is_one_approach_until_the_memory_runs_out() {
    // 40 ticks are 0.625 game-days (15 h); a 24 h memory outlasts the run.
    let (resolved, _, _) = run(-1.0, false, Some(24.0), 1, 40);
    assert_eq!(resolved.len(), 1, "{resolved:?}");
    assert!(!resolved[0].accepted, "the guard: the first exchange was refused");
}

#[test]
fn the_memory_ends_and_the_person_is_asked_again() {
    // A 12 h memory in a 24 h run: one approach, then the second after the memory.
    let (resolved, _, _) = run(-1.0, false, Some(12.0), 1, 64);
    assert_eq!(resolved.len(), 2, "{resolved:?}");
}

#[test]
fn a_person_turned_down_turns_to_someone_new() {
    // Neighbours at 1 m and 2 m, both of whom refuse. The nearest is asked first; once refused, the next
    // approach is the other one, and then nobody is left to ask for the rest of the memory.
    let (resolved, _, others) = run(-1.0, false, Some(24.0), 2, 40);
    let targets: Vec<Entity> = resolved.iter().map(|r| r.target).collect();
    assert_eq!(targets, vec![others[0], others[1]], "{resolved:?}");
}

#[test]
fn an_accepted_approach_is_not_remembered() {
    let (resolved, _, others) = run(2.0, false, Some(24.0), 1, 40);
    assert!(resolved.len() >= 4, "{resolved:?}");
    assert!(resolved.iter().all(|r| r.target == others[0]));
}

#[test]
fn a_refused_insult_is_not_a_rebuff() {
    // Hostility after a rejection is its own response (Bondu & Richter 2016); the retaliation refractory,
    // switched off here, is what holds it, not this memory.
    let (resolved, _, _) = run(-1.0, true, Some(24.0), 1, 40);
    assert!(resolved.len() >= 4, "{resolved:?}");
    assert!(resolved.iter().all(|r| r.interaction == "insult"));
}

/// **A reload keeps who has refused this person.** The memory steers the next decision, so a save that dropped
/// it would send a restored person back to someone the uninterrupted run was still avoiding.
#[cfg(feature = "serde")]
#[test]
fn a_restore_keeps_the_memory_of_a_refusal() {
    use bevy_society::{PersonKey, SocietySave, capture, restore};

    fn keyed(memory: Option<f32>) -> (App, Entity, Entity) {
        let mut settings = common::settings();
        settings.seconds_per_day = 1.0;
        settings.tuning.rebuff_memory_hours = memory;
        settings.tuning.retaliation_refractory_hours = None;
        let mut app = headless(defs(-1.0, false), settings);
        let actor = spawn_person(&mut app, "A", Vec2::ZERO);
        let other = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
        app.world_mut().entity_mut(actor).insert((Autonomous, PersonKey(0)));
        app.world_mut().entity_mut(other).insert(PersonKey(1));
        (app, actor, other)
    }

    let (mut before, ..) = keyed(Some(24.0));
    common::run(&mut before, 30);
    let save: SocietySave = capture(before.world_mut());
    let kept = &save.persons[0].rebuffed;
    assert_eq!(kept.len(), 1, "the refuser is saved: {kept:?}");
    assert_eq!(kept[0].0, 1, "by its person key");

    let (mut after, ..) = keyed(Some(24.0));
    after.update();
    restore(after.world_mut(), save.clone()).expect("the save applies");
    assert_eq!(
        capture(after.world_mut()).persons[0].rebuffed,
        save.persons[0].rebuffed,
        "the restored person avoids the same person until the same day"
    );
}
