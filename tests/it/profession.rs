//! **A task is not a skill.** Acemoglu & Autor (2010, DOI `10.3386/w16082`) separate "a unit of work
//! activity that produces output" from "a worker's endowment of capabilities for performing various
//! tasks", and the crate modelled neither: `work` was one `Hour(8, 17)` gate at any place tagged `work`, so
//! a miller and a weaver were the same person in a different colour.
//!
//! This file holds the two claims the mechanic makes.
use crate::common;

use bevy::prelude::*;
use bevy_society::{
    ActivityStarted, Autonomous, Home, Personality, Place, PlaceTags, SkillGroup, SocietyDefs,
    skill_level,
};
use std::collections::BTreeMap;

/// Two tasks at one bench: `craft`, which a diligent person does better, and `potter`, which nobody does
/// better. **The rival task is what makes the test bite** — with only one option every villager does it
/// whatever its score, which is how the first version of this measurement "proved" the skill did nothing.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            skills: [(id: "craft", group: Craft)],
            needs: [(id: "fun", decay_per_day: 0.1)],
            activities: [
                (id: "craft", target: Place("bench"), duration_hours: 0.5,
                 profession: Some("craft"),
                 considerations: [(input: Skill("craft"), curve: Linear(0.9, 0.05))],
                 satisfies: [], interaction: None, barks: []),
                (id: "potter", target: Place("bench"), duration_hours: 0.5,
                 considerations: [(input: Need("fun"), curve: Step(0.0))],
                 satisfies: [], interaction: None, barks: []),
            ],
        )"#,
    )
    .expect("the definitions parse")
}

/// A village of one bench and `people`, each with the given conscientiousness.
fn village(traits: &[f32]) -> (App, Vec<Entity>) {
    let mut app = common::headless(defs(), common::settings());
    let bench = app
        .world_mut()
        .spawn((
            Name::new("bench"),
            Place { capacity: 8 },
            PlaceTags(vec!["bench".to_owned()]),
            Transform::default(),
        ))
        .id();
    let mut people = Vec::new();
    for (i, c) in traits.iter().enumerate() {
        let entity =
            common::spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.5, 0.0));
        app.world_mut().entity_mut(entity).insert((
            Autonomous,
            Home(bench),
            Personality {
                conscientiousness: *c,
                ..Default::default()
            },
        ));
        people.push(entity);
    }
    (app, people)
}

/// `(starts of `craft`, all starts)` per person.
fn counts(app: &mut App, people: &[Entity], ticks: usize) -> Vec<(u32, u32)> {
    let mut craft: BTreeMap<Entity, u32> = BTreeMap::new();
    let mut all: BTreeMap<Entity, u32> = BTreeMap::new();
    for _ in 0..ticks {
        app.update();
        for message in app
            .world_mut()
            .resource_mut::<Messages<ActivityStarted>>()
            .drain()
        {
            if message.activity == "craft" {
                *craft.entry(message.person).or_insert(0) += 1;
            }
            *all.entry(message.person).or_insert(0) += 1;
        }
    }
    people
        .iter()
        .map(|e| {
            (
                craft.get(e).copied().unwrap_or(0),
                all.get(e).copied().unwrap_or(0),
            )
        })
        .collect()
}

/// **The same personality is the same skill, and a different one is not.**
#[test]
fn a_skill_is_the_trait_its_group_names() {
    let diligent = Personality {
        conscientiousness: 0.9,
        ..Default::default()
    };
    let idle = Personality {
        conscientiousness: 0.1,
        ..Default::default()
    };
    assert!((skill_level(&diligent, SkillGroup::Craft) - 0.9).abs() < 1e-6);
    assert!((skill_level(&idle, SkillGroup::Craft) - 0.1).abs() < 1e-6);
    // Two people with the same traits are equally good at the same trade: nothing is stored, so nothing
    // can drift apart, which is what keeps this additive and save-free.
    assert_eq!(
        skill_level(&diligent, SkillGroup::Craft),
        skill_level(&diligent.clone(), SkillGroup::Craft)
    );
    // The groups read different traits, so craft and invention are independent.
    let inventive = Personality {
        conscientiousness: 0.1,
        openness: 0.9,
        ..Default::default()
    };
    assert!((skill_level(&inventive, SkillGroup::Craft) - 0.1).abs() < 1e-6);
    assert!((skill_level(&inventive, SkillGroup::Creative) - 0.9).abs() < 1e-6);
}

/// **The skill decides who does the task.** Comparative advantage, measured over a run rather than
/// asserted of `signal_value`: three villagers, one bench, two tasks, and the diligent one does the
/// crafting.
#[test]
fn the_skill_decides_who_does_the_task() {
    let (mut app, people) = village(&[0.95, 0.5, 0.05]);
    let counts = counts(&mut app, &people, 2_400);
    let (diligent, idle) = (counts[0].0, counts[2].0);
    let total: u32 = counts.iter().map(|(c, _)| c).sum();
    assert!(
        total > 0,
        "the fixture must produce crafting at all: {counts:?}"
    );
    assert!(
        diligent > idle,
        "the person with the highest craft skill must craft more: {counts:?}"
    );
    assert_eq!(
        idle, 0,
        "and the least diligent never chooses it, because its only consideration scores them near zero: \
         {counts:?}"
    );
}

/// **Nothing is gated.** A village of people who are all bad at the trade still does it when it is the only
/// option — assignment is not what the mechanic does.
#[test]
fn a_low_skill_person_still_takes_the_task() {
    let only_craft = SocietyDefs::from_ron(
        r#"(
            skills: [(id: "craft", group: Craft)],
            needs: [(id: "fun", decay_per_day: 0.1)],
            activities: [
                (id: "craft", target: Place("bench"), duration_hours: 0.5,
                 profession: Some("craft"),
                 considerations: [(input: Skill("craft"), curve: Linear(0.9, 0.05))],
                 satisfies: [], interaction: None, barks: []),
            ],
        )"#,
    )
    .expect("the definitions parse");
    let mut app = common::headless(only_craft, common::settings());
    let bench = app
        .world_mut()
        .spawn((
            Name::new("bench"),
            Place { capacity: 4 },
            PlaceTags(vec!["bench".to_owned()]),
            Transform::default(),
        ))
        .id();
    let person = common::spawn_person(&mut app, "Feckless", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert((
        Autonomous,
        Home(bench),
        Personality {
            conscientiousness: 0.01,
            ..Default::default()
        },
    ));
    let counts = counts(&mut app, &[person], 800);
    assert!(
        counts[0].0 > 0,
        "a task is a choice, not a licence: a barely-conscientious villager must still be able to take \
         the only work on offer ({counts:?})"
    );
}

/// Same seed, same traits, same choices: the skill adds no draw and no state.
#[test]
fn the_skill_is_deterministic() {
    let run = || {
        let (mut app, people) = village(&[0.9, 0.3]);
        counts(&mut app, &people, 600)
    };
    assert_eq!(run(), run(), "two identical runs must choose identically");
}
