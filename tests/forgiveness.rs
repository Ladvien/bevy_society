//! **A grievance fades faster toward someone its holder still trusts** (`Tuning::forgiveness`): forgiveness
//! follows relationship value and low exploitation risk (McCullough et al. 2010, DOI 10.1037/a0019349; Burnette
//! et al. 2012, DOI 10.1177/0146167211424582), carried by post-transgression trust (Strelan, Karremans & Krieg
//! 2017, DOI 10.1111/bjso.12173). `ctx.grudge` is what the starter pack's `snub` reads, so this is the number a
//! victim's next insult is chosen on.

mod common;

use bevy::prelude::*;
use bevy_society::{
    Memories, Memory, MemoryKind, PersonId, SocietyDefs, SocietySettings, Ties, test_trigger,
};
use common::{headless, settings, spawn_person};

/// One trigger that fires when the actor resents the target past 0.35.
const DEFS: &str = r#"(
    triggers: [
        (
            id: "resents",
            on: Custom("ask"),
            if: Some("ctx.grudge(actor, target) > 0.35"),
            then: "fx.emit(\"resents\", #{})",
        ),
    ],
)"#;

/// The same insult, three game-days old, held by a victim against a trusted and a distrusted offender.
/// Returns whether the victim still resents each past 0.35: `(trusted, distrusted)`.
fn resents(forgiveness: f32) -> (bool, bool) {
    let mut config = settings();
    config.tuning.forgiveness = forgiveness;
    let mut app = headless(SocietyDefs::from_ron(DEFS).expect("parses"), config);
    let victim = spawn_person(&mut app, "Victim", Vec2::ZERO);
    let trusted = spawn_person(&mut app, "Trusted", Vec2::new(2.0, 0.0));
    let distrusted = spawn_person(&mut app, "Distrusted", Vec2::new(-2.0, 0.0));
    app.update();

    // An insult from each, three game-days before the clock's start, at the size the exchange path gives
    // an accepted insult (0.3 + |harm 0.4|).
    let insult = |other: Entity| Memory {
        day: -3.0,
        kind: MemoryKind::Interaction {
            id: bevy_society::InteractionId(0),
            accepted: true,
            initiated: false,
        },
        other: Some(other),
        valence: -0.7,
        importance: 0.7,
    };
    let mut memories = Memories::default();
    memories.entries.push(insult(trusted));
    memories.entries.push(insult(distrusted));
    let mut ties = Ties::default();
    ties.entry(trusted).trust = 0.9;
    ties.entry(distrusted).trust = 0.1;
    app.world_mut()
        .entity_mut(victim)
        .insert((memories, ties));
    app.update();

    let id = |e: Entity| *app.world().get::<PersonId>(e).expect("a person");
    let (v, t, d) = (id(victim), id(trusted), id(distrusted));
    let ask = |app: &mut App, target: PersonId| {
        test_trigger(app.world_mut(), "resents", Some(v), Some(target))
            .expect("answers")
            .condition
            == Some(true)
    };
    (ask(&mut app, t), ask(&mut app, d))
}

#[test]
fn a_trusted_offender_is_forgiven_and_a_distrusted_one_is_not() {
    // 0.7 · 0.995^(~78 h · (1 + 2 · trust)): 0.24 toward trust 0.9, 0.44 toward trust 0.1.
    assert_eq!(resents(2.0), (false, true));
}

#[test]
fn without_forgiveness_trust_does_not_matter() {
    // 0.7 · 0.995^78 = 0.47 toward both: the one recency curve, whoever the offender is.
    assert_eq!(resents(0.0), (true, true));
}

#[test]
fn a_forgiving_village_is_the_same_village_twice() {
    let run = || {
        let mut app = common::village::village(common::village::village_defs(), 7, 24);
        app.world_mut()
            .resource_mut::<SocietySettings>()
            .tuning
            .forgiveness = 2.0;
        for _ in 0..1500 {
            app.update();
        }
        common::hash::hash(&app)
    };
    assert_eq!(run(), run());
}
