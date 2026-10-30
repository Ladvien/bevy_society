//! **Host-reported deeds with named witnesses, and the teller on `RumorLearned`** (issue #8).
//!
//! Sanctuary resolves events the society has no eyes for: a townsperson found afflicted at dawn, a
//! person accused by a crowd at the well. They must enter reputation and rumor **as seen by the people
//! the game names**, and then spread person to person. And the belief system needs to know *who told* a
//! listener, because what a listener makes of a rumor depends on the teller.

mod common;

use bevy::prelude::*;
use bevy_society::{
    Knowledge, Memory, MemoryKind, ReportDeed, RumorLearned, SocietyDefs, Tie, Ties,
};
use common::{headless, spawn_person};

/// A village with one deed (`brawl`, harm -0.5) and one gossip interaction, so a rumor can both be
/// reported and be told.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.2)],
            topics: [(id: "violence")],
            deeds: [(id: "brawl", topic: "violence", harm: -0.5)],
            activities: [(
                id: "chat",
                target: Person,
                duration_hours: 1.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                interaction: Some("gossip"),
            )],
            interactions: [(id: "gossip", accept: (base: 1.0), gossip: true)],
        )"#,
    )
    .expect("the test definitions parse")
}

fn app() -> App {
    let mut settings = common::settings();
    // A game-day long enough that a walk is not the interesting thing.
    settings.seconds_per_day = 64.0;
    // One game-day is 64 s, so `rumor_threshold` at its default (0.4) is crossed by harm 0.5.
    let mut app = headless(defs(), settings);
    app.update();
    app.update();
    app
}

/// A person's trust in another, or the neutral `Tie::new` default when there is no tie yet — so a
/// before/after comparison measures the change rather than the presence of an entry.
fn trust_of(app: &App, from: Entity, to: Entity) -> f32 {
    app.world()
        .get::<Ties>(from)
        .and_then(|ties| ties.get(to))
        .copied()
        .map(|tie: Tie| tie.trust)
        .unwrap_or(0.5)
}

fn witnessed_memories(app: &App, person: Entity) -> usize {
    app.world()
        .get::<bevy_society::Memories>(person)
        .map(|memories| {
            memories
                .entries
                .iter()
                .filter(|memory: &&Memory| matches!(memory.kind, MemoryKind::Witnessed { .. }))
                .count()
        })
        .unwrap_or(0)
}

/// **Criterion 1.** After `ReportDeed`, each named witness has the trust change and a `Witnessed`
/// memory — and **no other person does**.
#[test]
fn the_named_witnesses_react_and_nobody_else_does() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    let witness_a = spawn_person(&mut app, "Cole", Vec2::new(20.0, 0.0));
    let witness_b = spawn_person(&mut app, "Dale", Vec2::new(-20.0, 0.0));
    // Far away and not named: must be untouched.
    let bystander = spawn_person(&mut app, "Eve", Vec2::new(40.0, 40.0));

    let before = trust_of(&app, witness_a, actor);
    let before_bystander = trust_of(&app, bystander, actor);

    app.world_mut().write_message(ReportDeed {
        actor,
        patient: Some(patient),
        deed: "brawl".to_owned(),
        witnesses: vec![witness_a, witness_b],
    });
    // One tick to queue, one to resolve.
    app.update();
    app.update();

    for (who, name) in [(witness_a, "Cole"), (witness_b, "Dale")] {
        let after = trust_of(&app, who, actor);
        assert!(
            after < before,
            "{name} saw a harmful deed and should trust its actor less: {before} -> {after}"
        );
        assert_eq!(
            witnessed_memories(&app, who),
            1,
            "{name} should have exactly one Witnessed memory"
        );
    }
    assert_eq!(
        trust_of(&app, bystander, actor),
        before_bystander,
        "an unnamed person must be untouched"
    );
    assert_eq!(
        witnessed_memories(&app, bystander),
        0,
        "an unnamed person must have no Witnessed memory"
    );
}

/// **Criterion 2.** A reported deed over `rumor_threshold` creates a rumor known by exactly the
/// witnesses.
#[test]
fn a_heavy_reported_deed_creates_a_rumor_the_witnesses_know() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    let witness_a = spawn_person(&mut app, "Cole", Vec2::new(20.0, 0.0));
    let witness_b = spawn_person(&mut app, "Dale", Vec2::new(-20.0, 0.0));
    let outsider = spawn_person(&mut app, "Eve", Vec2::new(40.0, 40.0));

    app.world_mut().write_message(ReportDeed {
        actor,
        patient: Some(patient),
        deed: "brawl".to_owned(),
        witnesses: vec![witness_a, witness_b],
    });
    app.update();
    app.update();

    let book = app.world().resource::<bevy_society::RumorBook>();
    assert_eq!(
        book.rumors.len(),
        1,
        "harm 0.5 is over the threshold, so one rumor"
    );
    let rumor = &book.rumors[0];
    assert!(rumor.alive, "a fresh rumor is alive");

    for (who, name) in [(witness_a, "Cole"), (witness_b, "Dale")] {
        let knows = app
            .world()
            .get::<Knowledge>(who)
            .is_some_and(|known| known.get(bevy_society::RumorId(0)).is_some());
        assert!(knows, "{name} was a witness, so should know the rumor");
    }
    let outsider_knows = app
        .world()
        .get::<Knowledge>(outsider)
        .is_some_and(|known| known.get(bevy_society::RumorId(0)).is_some());
    assert!(!outsider_knows, "an outsider must not know the rumor");
}

/// **Criterion 3.** A rumor learned through gossip carries `from: Some(teller)`; one learned by
/// witnessing carries `from: None`.
#[test]
fn a_gossiped_rumor_names_the_teller_and_a_witnessed_one_does_not() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    let teller = spawn_person(&mut app, "Cole", Vec2::new(20.0, 0.0));
    let listener = spawn_person(&mut app, "Dale", Vec2::new(20.5, 0.0));
    // Both autonomous and standing together, so the crate's own `chat` activity produces the gossip
    // exchange. A host `Interact` resolves in a tick, but a `chat` activity needs the pair to *arrive*
    // first, and that is the same path a real host takes.
    app.world_mut()
        .entity_mut(teller)
        .insert(bevy_society::Autonomous);
    app.world_mut()
        .entity_mut(listener)
        .insert(bevy_society::Autonomous);

    // A reported deed makes `teller` know the rumor with `from: None`.
    app.world_mut().write_message(ReportDeed {
        actor,
        patient: Some(patient),
        deed: "brawl".to_owned(),
        witnesses: vec![teller],
    });
    app.update();
    app.update();

    let witnessed_learnings: Vec<RumorLearned> = app
        .world_mut()
        .resource_mut::<bevy::ecs::message::Messages<RumorLearned>>()
        .drain()
        .collect();
    assert_eq!(
        witnessed_learnings.len(),
        1,
        "the teller learned it by seeing"
    );
    assert_eq!(
        witnessed_learnings[0].from, None,
        "a witnessed deed is nobody's word"
    );

    // Now `teller` tells `listener`, through a host `Interact`: an exchange resolves on the next tick,
    // which is what a host does when it wants two people to talk right now.
    app.world_mut().write_message(bevy_society::Interact {
        actor: teller,
        target: listener,
        interaction: "gossip".to_owned(),
    });
    // **Drain on every tick.** Bevy keeps a message for two buffer swaps, so collecting only at the end
    // loses everything but the last two updates — which is exactly how this test first failed.
    let mut gossiped: Vec<RumorLearned> = Vec::new();
    for _ in 0..6 {
        app.update();
        gossiped.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<RumorLearned>>()
                .drain(),
        );
    }
    let to_listener: Vec<&RumorLearned> = gossiped
        .iter()
        .filter(|learned| learned.person == listener)
        .collect();
    assert!(
        !to_listener.is_empty(),
        "the listener should have been told the rumor, got {gossiped:?}"
    );
    assert_eq!(
        to_listener[0].from,
        Some(teller),
        "a gossip-learned rumor names its teller"
    );
}

/// An unknown deed warns and creates nothing.
#[test]
fn an_unknown_deed_creates_nothing() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let witness = spawn_person(&mut app, "Cole", Vec2::new(2.0, 0.0));
    let before = trust_of(&app, witness, actor);

    app.world_mut().write_message(ReportDeed {
        actor,
        patient: None,
        deed: "not-a-deed".to_owned(),
        witnesses: vec![witness],
    });
    app.update();
    app.update();

    assert_eq!(
        trust_of(&app, witness, actor),
        before,
        "nothing should change"
    );
    assert!(
        app.world()
            .resource::<bevy_society::RumorBook>()
            .rumors
            .is_empty(),
        "an unknown deed makes no rumor"
    );
}

/// An empty witness list creates nothing: a deed nobody saw is not a deed.
#[test]
fn an_empty_witness_list_creates_nothing() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.update();

    app.world_mut().write_message(ReportDeed {
        actor,
        patient: None,
        deed: "brawl".to_owned(),
        witnesses: Vec::new(),
    });
    app.update();
    app.update();

    assert!(
        app.world()
            .resource::<bevy_society::RumorBook>()
            .rumors
            .is_empty(),
        "no witnesses means no rumor"
    );
}

/// The actor and the patient are never witnesses of their own deed, even when the host names them.
#[test]
fn the_parties_are_not_witnesses_of_their_own_deed() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    app.update();

    app.world_mut().write_message(ReportDeed {
        actor,
        patient: Some(patient),
        deed: "brawl".to_owned(),
        // Only the two parties, so the filtered list is empty and nothing happens.
        witnesses: vec![actor, patient],
    });
    app.update();
    app.update();

    assert!(
        app.world()
            .resource::<bevy_society::RumorBook>()
            .rumors
            .is_empty(),
        "naming only the parties leaves no witness, so no deed"
    );
}

/// Two witnesses named in either order give the same society: the list is deduped and ordered by
/// `PersonId` before the deed is reported, so a host's `Vec` order cannot change the result.
#[test]
fn witness_order_does_not_change_the_result() {
    let run = |reversed: bool| {
        let mut app = app();
        let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
        let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
        let a = spawn_person(&mut app, "Cole", Vec2::new(20.0, 0.0));
        let b = spawn_person(&mut app, "Dale", Vec2::new(-20.0, 0.0));
        let witnesses = if reversed { vec![b, a] } else { vec![a, b] };
        app.world_mut().write_message(ReportDeed {
            actor,
            patient: Some(patient),
            deed: "brawl".to_owned(),
            witnesses,
        });
        app.update();
        app.update();
        (
            trust_of(&app, a, actor).to_bits(),
            trust_of(&app, b, actor).to_bits(),
            app.world()
                .resource::<bevy_society::RumorBook>()
                .rumors
                .len(),
        )
    };
    assert_eq!(
        run(false),
        run(true),
        "the order the host lists witnesses in must not change the society"
    );
}

/// A reported deed leaves its victim a `Suffered` memory (not a `Witnessed` one), so the victim's grudge
/// toward the actor is above zero; the actor and an uninvolved witness hold no grudge.
#[test]
fn a_reported_victim_remembers_and_resents_the_actor() {
    let mut app = app();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    let witness = spawn_person(&mut app, "Cole", Vec2::new(20.0, 0.0));
    app.world_mut().write_message(ReportDeed {
        actor,
        patient: Some(patient),
        deed: "brawl".to_owned(),
        witnesses: vec![witness],
    });
    app.update();
    app.update();
    let now = app
        .world()
        .resource::<bevy_society::SocietyClock>()
        .now_days;
    let victim = app.world().get::<bevy_society::Memories>(patient).unwrap();
    assert!(
        victim
            .entries
            .iter()
            .any(|m| matches!(m.kind, MemoryKind::Suffered { actor: a, .. } if a == actor))
    );
    assert_eq!(witnessed_memories(&app, patient), 0);
    assert!(
        victim.grudge(actor, now) > 0.5,
        "0.3 + |-0.5| = 0.8 at the moment of the deed"
    );
    let by = app.world().get::<bevy_society::Memories>(witness).unwrap();
    assert_eq!(by.grudge(actor, now), 0.0, "a bystander is not wronged");
}
