//! A trigger with an `area` fires only for an actor standing inside it, and an event that binds no
//! actor cannot carry one.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{FireTrigger, ScriptEvent, SocietyDefs};
use common::{headless, settings, spawn_person};

const AREA: &str = r#"(
    triggers: [(
        id: "poke",
        on: Custom("poke"),
        then: "fx.emit(\"met\", #{})",
        area: Some((min: (0.0, 0.0), max: (10.0, 10.0))),
    )],
)"#;

fn poke(app: &mut App, who: Entity) -> usize {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(who),
        target: None,
    });
    let mut met = 0;
    for _ in 0..2 {
        app.update();
        met += app
            .world_mut()
            .resource_mut::<Messages<ScriptEvent>>()
            .drain()
            .filter(|e| e.name == "met")
            .count();
    }
    met
}

#[test]
fn an_area_trigger_fires_only_for_an_actor_inside_it() {
    let mut app = headless(SocietyDefs::from_ron(AREA).expect("parses"), settings());
    let inside = spawn_person(&mut app, "In", Vec2::new(5.0, 5.0));
    let outside = spawn_person(&mut app, "Out", Vec2::new(30.0, 5.0));
    let edge = spawn_person(&mut app, "Edge", Vec2::new(10.0, 0.0));
    for _ in 0..2 {
        app.update();
    }

    assert_eq!(poke(&mut app, inside), 1, "inside fires");
    assert_eq!(poke(&mut app, outside), 0, "outside does not");
    assert_eq!(poke(&mut app, edge), 1, "the edge is inside");

    let mut t = app
        .world_mut()
        .get_mut::<Transform>(inside)
        .expect("has a transform");
    t.translation.x = 50.0;
    assert_eq!(poke(&mut app, inside), 0, "walking out stops it");
}

#[test]
fn an_area_on_an_event_with_no_actor_is_refused() {
    let err = SocietyDefs::from_ron(
        r#"(triggers: [(
            id: "t", on: Tick, then: "",
            area: Some((min: (0.0, 0.0), max: (1.0, 1.0))),
        )])"#,
    )
    .err()
    .expect("Tick binds no actor");
    assert!(err.to_string().contains("trigger 't'"), "{err}");

    let err = SocietyDefs::from_ron(
        r#"(triggers: [(
            id: "t", on: Custom("x"), then: "",
            area: Some((min: (5.0, 0.0), max: (1.0, 1.0))),
        )])"#,
    )
    .err()
    .expect("inverted corners");
    assert!(err.to_string().contains("min <= max"), "{err}");
}

#[test]
fn a_fired_trigger_is_reported_with_its_id_and_actor() {
    use bevy_society::{PersonId, TriggerFired};
    let mut app = headless(SocietyDefs::from_ron(AREA).expect("parses"), settings());
    let inside = spawn_person(&mut app, "In", Vec2::new(5.0, 5.0));
    let outside = spawn_person(&mut app, "Out", Vec2::new(30.0, 5.0));
    for _ in 0..2 {
        app.update();
    }
    let mut fired = Vec::new();
    for who in [outside, inside] {
        app.world_mut().write_message(FireTrigger {
            name: "poke".to_owned(),
            actor: Some(who),
            target: None,
        });
        for _ in 0..2 {
            app.update();
            fired.extend(
                app.world_mut()
                    .resource_mut::<Messages<TriggerFired>>()
                    .drain(),
            );
        }
    }
    assert_eq!(fired.len(), 1, "only the person inside the area fired it");
    assert_eq!(fired[0].trigger, "poke");
    assert_eq!(fired[0].actor, Some(PersonId(0)));
    assert_eq!(fired[0].target, None);
}

const LIVE: &str = r#"(
    needs: [(id: "rest", decay_per_day: 0.1, initial: 0.5)],
    triggers: [(id: "poke", on: Custom("poke"), then: "fx.emit(\"v1\", #{})", once: true)],
)"#;

fn emitted(app: &mut App, who: Entity) -> Vec<String> {
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
    names
}

#[test]
fn an_edited_trigger_takes_effect_in_the_running_society() {
    use bevy_society::{DefsFile, reload_defs};
    let mut app = headless(SocietyDefs::from_ron(LIVE).expect("parses"), settings());
    let ada = spawn_person(&mut app, "Ada", Vec2::ZERO);
    let bram = spawn_person(&mut app, "Bram", Vec2::ZERO);
    for _ in 0..2 {
        app.update();
    }
    assert_eq!(emitted(&mut app, ada), ["v1"]);
    assert!(emitted(&mut app, ada).is_empty(), "once, so it is spent");

    // Edit the script and add a second trigger; `poke` keeps its id, so it stays spent.
    let edited = LIVE.replace("v1", "v2").replace(
        "once: true)],",
        "once: true), (id: \"extra\", on: Custom(\"poke\"), then: \"fx.emit(\\\"x\\\", #{})\")],",
    );
    reload_defs(
        app.world_mut(),
        DefsFile::from_ron(&edited).expect("parses"),
    )
    .expect("reloads");
    assert_eq!(
        emitted(&mut app, ada),
        ["x"],
        "poke stays spent, extra is new"
    );
    assert_eq!(
        emitted(&mut app, bram),
        ["v2", "x"],
        "a new actor runs the edited script"
    );

    // A broken edit and an edit that moves an id table are both refused, changing nothing.
    let broken = LIVE.replace("fx.emit(", "fx.emit(((");
    let err = reload_defs(
        app.world_mut(),
        DefsFile::from_ron(&broken).expect("parses"),
    )
    .expect_err("does not compile");
    assert!(err.to_string().contains("poke"), "{err}");
    let moved = LIVE.replace("\"rest\"", "\"sleep\"");
    let err = reload_defs(app.world_mut(), DefsFile::from_ron(&moved).expect("parses"))
        .expect_err("a need was renamed");
    assert!(err.to_string().contains("needs"), "{err}");
    assert_eq!(emitted(&mut app, bram), ["x"], "still the last good defs");
}

const BOUND: &str = r#"(
    factions: [(id: "millers"), (id: "smiths")],
    triggers: [
        (id: "by_faction", on: Custom("poke"), then: "fx.emit(\"faction\", #{})", factions: ["millers"]),
        (id: "by_person", on: Custom("poke"), then: "fx.emit(\"person\", #{})", persons: [1]),
    ],
)"#;

fn poke_names(app: &mut App, who: Entity) -> Vec<String> {
    app.world_mut().write_message(FireTrigger {
        name: "poke".to_owned(),
        actor: Some(who),
        target: None,
    });
    let mut seen = Vec::new();
    for _ in 0..2 {
        app.update();
        seen.extend(
            app.world_mut()
                .resource_mut::<Messages<ScriptEvent>>()
                .drain()
                .map(|e| e.name),
        );
    }
    seen
}

#[test]
fn a_trigger_bound_to_factions_or_persons_fires_only_for_them() {
    use bevy_society::Memberships;
    let defs = SocietyDefs::from_ron(BOUND).expect("parses");
    let miller = defs.faction_id("millers").expect("defined");
    let smith = defs.faction_id("smiths").expect("defined");
    let mut app = headless(defs, settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::ZERO);
    let c = spawn_person(&mut app, "C", Vec2::ZERO);
    app.world_mut()
        .entity_mut(a)
        .insert(Memberships(vec![miller]));
    app.world_mut()
        .entity_mut(b)
        .insert(Memberships(vec![smith]));
    for _ in 0..2 {
        app.update();
    }

    assert_eq!(
        poke_names(&mut app, a),
        ["faction"],
        "A is a miller; PersonId 0 is not listed"
    );
    assert_eq!(
        poke_names(&mut app, b),
        ["person"],
        "B is a smith, listed by id 1"
    );
    assert!(poke_names(&mut app, c).is_empty(), "C is neither");
}

#[test]
fn factions_must_exist_and_need_an_actor() {
    let err = SocietyDefs::from_ron(
        r#"(triggers: [(id: "t", on: Custom("x"), then: "", factions: ["ghosts"])])"#,
    )
    .err()
    .expect("unknown faction");
    assert!(err.to_string().contains("ghosts"), "{err}");

    let err = SocietyDefs::from_ron(
        r#"(factions: [(id: "f")], triggers: [(id: "t", on: Tick, then: "", factions: ["f"])])"#,
    )
    .err()
    .expect("Tick binds no actor");
    assert!(err.to_string().contains("trigger 't'"), "{err}");
}
