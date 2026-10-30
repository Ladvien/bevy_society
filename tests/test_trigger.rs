//! `test_trigger` dry-runs one authored trigger against the society the last tick froze: it says
//! whether the trigger would fire for given people and what it would write, and changes nothing.

mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{PersonId, ScriptEvent, SocietyDefs, TriggerTestError, test_trigger};
use common::{headless, settings, spawn_person};

const DEFS: &str = r#"(
    triggers: [
        (
            id: "gate",
            on: Custom("poke"),
            if: Some("actor == 1"),
            then: "fx.adjust(actor, 0, \"trust\", 0.2); fx.emit(\"met\", #{})",
            once: true,
            area: Some((min: (0.0, 0.0), max: (10.0, 10.0))),
        ),
        (id: "broken", on: Custom("poke"), then: "fx.adjust(actor, 0, 7, 0.2)"),
    ],
)"#;

fn id_of(app: &App, e: Entity) -> PersonId {
    *app.world().get::<PersonId>(e).expect("a person")
}

#[test]
fn a_dry_run_reports_each_check_and_the_writes_and_changes_nothing() {
    let mut app = headless(SocietyDefs::from_ron(DEFS).expect("parses"), settings());
    let a = spawn_person(&mut app, "A", Vec2::new(5.0, 5.0));
    let b = spawn_person(&mut app, "B", Vec2::new(5.0, 5.0));
    let far = spawn_person(&mut app, "Far", Vec2::new(40.0, 5.0));
    let (a, b, far) = (id_of(&app, a), id_of(&app, b), id_of(&app, far));
    assert_eq!((a.0, b.0), (0, 1), "ids follow spawn order");

    let early = test_trigger(app.world_mut(), "gate", Some(b), None);
    assert_eq!(early, Err(TriggerTestError::NoSnapshot));

    for _ in 0..2 {
        app.update();
    }

    let hit = test_trigger(app.world_mut(), "gate", Some(b), Some(a)).expect("answers");
    assert!(hit.selected && hit.in_area && hit.would_fire);
    assert_eq!(hit.condition, Some(true));
    assert_eq!(hit.effects.len(), 2, "{:?}", hit.effects);
    assert!(hit.effects[0].contains("trust"), "{:?}", hit.effects);

    // Asking twice gives the same answer: a `once` trigger's history is not spent by a dry run.
    assert_eq!(
        test_trigger(app.world_mut(), "gate", Some(b), Some(a)).expect("answers"),
        hit
    );

    let wrong_if = test_trigger(app.world_mut(), "gate", Some(a), None).expect("answers");
    assert_eq!(wrong_if.condition, Some(false));
    assert!(!wrong_if.would_fire && wrong_if.effects.is_empty());

    let outside = test_trigger(app.world_mut(), "gate", Some(far), None).expect("answers");
    assert!(!outside.in_area && !outside.would_fire);
    assert_eq!(outside.condition, None, "the if never ran");

    // Nothing was written or queued: a real firing afterwards still happens, once.
    let real = app
        .world_mut()
        .resource_mut::<Messages<ScriptEvent>>()
        .drain()
        .count();
    assert_eq!(real, 0, "the dry run emitted nothing");
}

#[test]
fn unknown_triggers_and_failing_scripts_are_errors_naming_the_cause() {
    let mut app = headless(SocietyDefs::from_ron(DEFS).expect("parses"), settings());
    let a = spawn_person(&mut app, "A", Vec2::new(5.0, 5.0));
    let a = id_of(&app, a);
    for _ in 0..2 {
        app.update();
    }
    let err = test_trigger(app.world_mut(), "nope", Some(a), None).expect_err("unknown id");
    assert_eq!(err, TriggerTestError::UnknownTrigger("nope".to_owned()));

    let err = test_trigger(app.world_mut(), "broken", Some(a), None).expect_err("bad field type");
    assert!(
        matches!(&err, TriggerTestError::Script { owner, .. } if owner.contains("broken")),
        "{err}"
    );
}
