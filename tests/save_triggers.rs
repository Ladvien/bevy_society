//! A save's trigger bookkeeping follows the trigger, not its place in the defs list, and a person
//! who had already fired a `NeedBelow` line is still disarmed after a reload.
//!
//! `reload_defs` already matches `once` and cooldown state by the authored `id`; `restore` has to as
//! well, because an editor saves an edited defs file and the game loads the old save into it.
#![cfg(feature = "serde")]

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{
    FireTrigger, Needs, PersonKey, ScriptEvent, SocietyDefs, TriggerFired, capture, restore,
};
use common::{headless, settings, spawn_person};

const V1: &str = r#"(
    triggers: [
        (id: "welcome", on: Custom("poke"), then: "fx.emit(\"welcome\", #{})", once: true),
        (id: "gone", on: Custom("poke"), then: "fx.emit(\"gone\", #{})", once: true),
    ],
)"#;

/// V1 with a trigger inserted ahead of `welcome` and `gone` removed.
const V2: &str = r#"(
    triggers: [
        (id: "dawn", on: Custom("poke"), then: "fx.emit(\"dawn\", #{})", once: true),
        (id: "welcome", on: Custom("poke"), then: "fx.emit(\"welcome\", #{})", once: true),
    ],
)"#;

/// A society of one keyed person named A, two updates in.
fn app(defs: &str) -> (App, Entity) {
    let mut app = headless(SocietyDefs::from_ron(defs).expect("parses"), settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    app.world_mut().entity_mut(a).insert(PersonKey(1));
    for _ in 0..2 {
        app.update();
    }
    (app, a)
}

fn poke(app: &mut App, who: Entity) -> Vec<String> {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(who),
        target: None,
    });
    let mut names = Vec::new();
    for _ in 0..2 {
        app.update();
        names.extend(
            app.world_mut()
                .resource_mut::<Messages<ScriptEvent>>()
                .drain()
                .map(|e| e.name),
        );
    }
    names.sort();
    names
}

#[test]
fn a_restore_keeps_once_state_with_its_trigger_after_the_defs_change() {
    let (mut old, a) = app(V1);
    assert_eq!(poke(&mut old, a), ["gone", "welcome"]);
    let save = capture(old.world_mut());

    // `dawn` is new and `welcome` moved from index 0 to 1: its spent state moves with it, and the
    // state of `gone`, which the defs no longer have, is dropped.
    let (mut new, a) = app(V2);
    restore(new.world_mut(), save).expect("names unchanged");
    assert_eq!(poke(&mut new, a), ["dawn"]);
}

#[test]
fn a_save_without_trigger_ids_applies_its_indices_as_before() {
    let (mut old, a) = app(V1);
    assert_eq!(poke(&mut old, a), ["gone", "welcome"]);
    let mut save = capture(old.world_mut());
    assert_eq!(
        save.triggers.ids,
        ["welcome", "gone"],
        "the save names its triggers"
    );

    // What a version-4 file holds: indices only.
    save.version = 4;
    save.triggers.ids.clear();
    let (mut same, a) = app(V1);
    let report = restore(same.world_mut(), save).expect("upgrades");
    assert_eq!(report.upgraded_from, Some(4));
    assert!(poke(&mut same, a).is_empty(), "both stay spent, by index");
}

const NEED: &str = r#"(
    needs: [(id: "social", decay_per_day: 0.0)],
    triggers: [(
        id: "lonely", on: NeedBelow(need: "social", below: 0.3),
        then: "fx.emit(\"lonely\", #{})",
    )],
)"#;

fn lonely_town() -> (App, Entity) {
    let (mut app, a) = app(NEED);
    app.world_mut().get_mut::<Needs>(a).expect("has needs").0[0] = 0.2;
    (app, a)
}

fn firings(app: &mut App, ticks: usize) -> usize {
    let mut n = 0;
    for _ in 0..ticks {
        app.update();
        n += app
            .world_mut()
            .resource_mut::<Messages<TriggerFired>>()
            .drain()
            .count();
    }
    n
}

#[test]
fn a_person_who_fired_a_need_line_is_still_disarmed_after_a_restore() {
    let (mut old, _) = lonely_town();
    assert_eq!(firings(&mut old, 5), 1, "falls below once");
    let save = capture(old.world_mut());
    assert_eq!(save.triggers.need_disarmed.len(), 1);

    // A fresh world is armed for everybody; the save puts the person back where they were.
    let (mut new, _) = lonely_town();
    restore(new.world_mut(), save).expect("restores");
    assert_eq!(firings(&mut new, 5), 0, "already fired, not recovered");
}
