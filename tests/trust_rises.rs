//! **A successful exchange must be able to raise trust.**
//!
//! `AcceptDef` scores an interaction as
//!
//! ```text
//! base + affinity·a + trust·(t − 0.5) + agreeableness·(g − 0.5) + pleasure·p + ingroup
//! ```
//!
//! so trust is *centred* on 0.5: above it helps, below it hurts. `Tie::new` starts every stranger at
//! exactly 0.5, which is the neutral point.
//!
//! Nothing in the starter pack ever raises it. `TieDelta` carries a `trust` field and every interaction's
//! `on_accept` and `on_reject` can set one, but the pack sets **none** — measured, the word `trust` does not
//! occur in `assets/starter.society.ron` at all. The only thing that moves trust is `witness_reaction`, which
//! applies `trust_gain · harm`, and the pack's deeds are `help 0.5`, `gift 0.6`, `insult -0.4`,
//! `steal -0.7` — so a village that gossips about a theft moves everyone's trust *down* and nothing moves it
//! back up.
//!
//! Measured over 20 000 ticks with 24 people, the trust a target holds toward the person approaching it:
//!
//! | interaction | trust |
//! |---|---|
//! | `chat` | 0.27 |
//! | `compliment` | 0.20 |
//! | `gossip` | 0.24 |
//! | `insult` | 0.13 |
//!
//! Every one below the neutral 0.5, so the trust term subtracts on every acceptance draw and roughly **47% of
//! interactions are refused**. A village where nobody's trust ever recovers does not have a trust model; it has
//! a decay.
mod common;

use bevy::prelude::*;
use bevy_society::{
    Autonomous, Home, InteractionResolved, Person, Place, PlaceTags, SocietyDefs, Ties,
};
use common::{headless, settings, spawn_person};

/// The mean trust each interaction is evaluated against, after `ticks` of village life.
fn mean_trust(defs: SocietyDefs, ticks: usize) -> (f32, f32, f32) {
    let mut day = settings();
    day.seconds_per_day = 64.0;
    day.start_hour = 15.0;
    day.tuning.travel_timeout_hours = 4.0;
    let mut app = headless(defs, day);
    let at = [(0.0_f32, 0.0), (6.5, -5.0), (-6.5, -5.0), (6.5, 5.0)];
    let places: Vec<Entity> = at
        .iter()
        .map(|(x, z)| {
            app.world_mut()
                .spawn((
                    Name::new("place"),
                    Place { capacity: 8 },
                    PlaceTags(vec![
                        "gathering".to_owned(),
                        "food".to_owned(),
                        "work".to_owned(),
                    ]),
                    Transform::from_xyz(*x, 0.0, *z),
                ))
                .id()
        })
        .collect();
    for i in 0..24 {
        let column = i / 8;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(at[column].0, at[column].1),
        );
        app.world_mut().entity_mut(person).insert((
            Autonomous,
            Person,
            Home(places[column % places.len()]),
        ));
    }

    let mut sum = 0.0_f64;
    let mut count = 0u32;
    let mut early = 0.0_f64;
    let mut early_n = 0u32;
    let (mut accepted, mut refused) = (0u32, 0u32);
    for _ in 0..ticks {
        app.update();
        let resolved: Vec<(Entity, Entity, bool)> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
            .drain()
            .map(|r| (r.actor, r.target, r.accepted))
            .collect();
        for (actor, target, ok) in resolved {
            if ok {
                accepted += 1;
            } else {
                refused += 1;
            }
            if let Some(tie) = app
                .world()
                .get::<Ties>(target)
                .and_then(|ties| ties.get(actor))
            {
                sum += f64::from(tie.trust);
                count += 1;
                if (count as usize) < 200 {
                    early += f64::from(tie.trust);
                    early_n += 1;
                }
            }
        }
    }
    let trust = (sum / f64::from(count.max(1))) as f32;
    let rate = accepted as f32 / (accepted + refused).max(1) as f32;
    let first = (early / f64::from(early_n.max(1))) as f32;
    (trust, rate, first)
}

/// The pack with every `TieDelta::trust` set to zero — the state before this iteration.
fn pack_without_trust_deltas() -> SocietyDefs {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/starter.society.ron"
    ))
    .expect("the starter pack is in the repository");
    let stripped = source
        .replace(
            "actor_to_target: (trust: 0.05),",
            "actor_to_target: (affinity: 0.0),",
        )
        .replace(
            "target_to_actor: (trust: 0.05),",
            "target_to_actor: (affinity: 0.0),",
        )
        .replace(
            "actor_to_target: (trust: 0.03),",
            "actor_to_target: (affinity: 0.0),",
        )
        .replace(
            "target_to_actor: (trust: 0.03),",
            "target_to_actor: (affinity: 0.0),",
        )
        .replace(
            "target_to_actor: (affinity: 0.1, trust: 0.02),",
            "target_to_actor: (affinity: 0.1),",
        );
    SocietyDefs::from_ron(&stripped).expect("the stripped pack parses")
}

#[test]
fn a_successful_exchange_raises_trust_against_a_pack_that_has_no_such_delta() {
    let with = mean_trust(SocietyDefs::starter().expect("the pack parses"), 12_000);
    let without = mean_trust(pack_without_trust_deltas(), 12_000);
    println!(
        "with trust deltas:    mean {:.3}, accept {:.3}",
        with.0, with.1
    );
    println!(
        "without trust deltas: mean {:.3}, accept {:.3}",
        without.0, without.1
    );

    assert!(
        with.0 > without.0,
        "an accepted exchange must raise trust: the pack with deltas reaches {:.3} and without them \
         {:.3}. Before this iteration the starter pack set **no** `TieDelta::trust` anywhere, so the \
         only thing that moved trust was `witness_reaction`'s `trust_gain * harm`, which is negative \
         for every deed the pack actually uses",
        with.0,
        without.0
    );
    assert!(
        with.1 > without.1,
        "raising trust must also make people more willing to accept: accept rate {:.3} against {:.3}. \
         `AcceptDef` centres trust on 0.5, so a village below it refuses more than it accepts",
        with.1,
        without.1
    );
}
