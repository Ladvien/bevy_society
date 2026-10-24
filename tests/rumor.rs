//! Rumors and what a person knows of them (plan D5): planting, learning, and the death of a rumor
//! with its subject.

mod common;

use bevy::prelude::*;
use bevy_society::{Knowledge, PlantRumor, RumorBook, RumorId, SocietyDefs, SocietySettings};
use common::{headless, run, settings, spawn_person};

/// Two topics, one deed `steal` on `honesty` with harm −0.7, and nothing else.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            topics: [(id: "kindness"), (id: "honesty")],
            deeds: [(id: "steal", topic: "honesty", harm: -0.7)],
        )"#,
    )
    .expect("the two-topic definition parses")
}

fn plant(app: &mut App, origin: Entity, subject: Entity, deed: &str, truth: bool) {
    app.world_mut().write_message(PlantRumor {
        origin,
        subject,
        patient: None,
        deed: deed.to_owned(),
        truth,
    });
}

#[test]
fn planted_rumor_is_known_by_its_origin_and_dies_with_its_subject() {
    let mut app = headless(defs(), settings());
    run(&mut app, 1);
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(40.0, 0.0));
    run(&mut app, 1);

    plant(&mut app, a, b, "steal", false);
    run(&mut app, 2);

    let book: Vec<_> = app.world().resource::<RumorBook>().rumors.clone();
    assert_eq!(book.len(), 1, "one rumor planted");
    let rumor = &book[0];
    assert_eq!(rumor.known_by, 1, "only the origin knows it");
    assert!(rumor.alive);
    assert!(!rumor.truth);
    assert!(
        (rumor.valence + 0.7).abs() < 1e-6,
        "valence is the deed's harm"
    );

    let known = app
        .world()
        .get::<Knowledge>(a)
        .expect("A is a person")
        .get(RumorId(0))
        .copied()
        .expect("A knows the rumor it planted");
    assert!(known.spreader, "the origin spreads it");
    assert!((known.belief - 1.0).abs() < 1e-6, "at full belief");

    // The subject leaves the world: the rumor dies and nobody knows it any more.
    app.world_mut().despawn(b);
    run(&mut app, 1);

    assert!(
        !app.world().resource::<RumorBook>().rumors[0].alive,
        "a rumor about a despawned person stops circulating"
    );
    assert!(
        app.world()
            .get::<Knowledge>(a)
            .expect("A is a person")
            .get(RumorId(0))
            .is_none(),
        "A no longer knows a dead rumor"
    );
}

#[test]
fn a_planted_rumor_naming_an_unknown_deed_creates_nothing() {
    let mut app = headless(defs(), settings());
    run(&mut app, 1);
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(2.0, 0.0));
    run(&mut app, 1);

    plant(&mut app, a, b, "nonsense", true);
    run(&mut app, 2);

    assert!(
        app.world().resource::<RumorBook>().rumors.is_empty(),
        "an unknown deed is logged and ignored"
    );
}

#[test]
fn a_lone_planted_rumor_has_no_reach() {
    let mut app = headless(defs(), settings());
    run(&mut app, 1);
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(40.0, 0.0));
    run(&mut app, 1);
    plant(&mut app, a, b, "steal", true);
    run(&mut app, 2);

    let book = app.world().resource::<RumorBook>();
    let reach = book.reach(RumorId(0), 2);
    assert!(
        (reach - 0.5).abs() < 1e-6,
        "one knower in two persons: {reach}"
    );
    assert_eq!(book.reach(RumorId(9), 2), 0.0, "an unknown id has no reach");
    assert_eq!(book.reach(RumorId(0), 0), 0.0, "no persons, no reach");
}

#[test]
fn settings_keep_seeded_randomness_out_of_planting() {
    // Planting is a message, not a draw: the same seed and the same message give the same book.
    let mut first = headless(defs(), SocietySettings::default());
    let mut second = headless(defs(), SocietySettings::default());
    for app in [&mut first, &mut second] {
        run(app, 1);
        let a = spawn_person(app, "A", Vec2::ZERO);
        let b = spawn_person(app, "B", Vec2::new(40.0, 0.0));
        run(app, 1);
        plant(app, a, b, "steal", true);
        run(app, 2);
    }
    assert_eq!(
        first.world().resource::<RumorBook>().rumors,
        second.world().resource::<RumorBook>().rumors,
    );
}

#[test]
fn a_planted_rumor_spreads_through_gossip() {
    // The chain case, end to end: A knows a rumor, and an accepted gossip interaction with B passes
    // it on. Without this, `rumor_reach` in the hero capture stays at 1/n forever.
    use bevy_society::Interact;

    let defs = SocietyDefs::from_ron(
        r#"(
            topics: [(id: "honesty")],
            deeds: [(id: "steal", topic: "honesty", harm: -0.7)],
            interactions: [(id: "tell", accept: (base: 0.99), gossip: true)],
        )"#,
    )
    .expect("the gossip definition parses");
    let mut settings = settings();
    settings.seconds_per_day = 1.0;
    let mut app = headless(defs, settings);
    // Three people: the rumor is about C, so B is a legitimate listener. A rumor about the listener
    // is deliberately not told to them, which is what the gossip rule says.
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    let c = spawn_person(&mut app, "C", Vec2::new(-20.0, 0.0));
    run(&mut app, 2);

    plant(&mut app, a, c, "steal", true);
    run(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "tell".to_owned(),
    });
    run(&mut app, 4);

    let book = app.world().resource::<RumorBook>();
    assert_eq!(book.rumors.len(), 1, "one rumor");
    assert_eq!(
        book.rumors[0].known_by, 2,
        "the listener learned it, so two people know it"
    );
    let learned = app
        .world()
        .get::<Knowledge>(b)
        .and_then(|k| k.get(RumorId(0)).copied());
    let learned = learned.expect("B knows the rumor");
    assert!(learned.spreader, "a fresh listener spreads it on");
    assert!(
        book.reach(RumorId(0), 3) > 0.6,
        "reach is known_by over living persons"
    );
}
