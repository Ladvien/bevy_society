//! Determinism (plan D12): the same seed and the same spawn sequence give the same society, bit for
//! bit, and a different seed gives a different one.
//!
//! The hash walks `PersonIndex.sorted` and reads each person's ties, opinions, knowledge and needs,
//! so it covers every model the crate keeps per person. It hashes raw `f32` bits rather than a
//! rounded value: a determinism guarantee that survives rounding is not the one the crate makes.

mod common;

use bevy::prelude::*;
use bevy_society::{
    Knowledge, Needs, Opinions, PersonId, PersonIndex, Personality, SocietyClock, Ties,
};

/// FNV-1a 64 over the whole society. Order is `PersonIndex.sorted`, so the hash is stable only when
/// the society is.
fn hash(app: &App) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn mix(state: &mut u64, value: u64) {
        for byte in value.to_le_bytes() {
            *state ^= u64::from(byte);
            *state = state.wrapping_mul(PRIME);
        }
    }

    let mut state = OFFSET;
    let index = app.world().resource::<PersonIndex>();
    let ids: &[(PersonId, Entity)] = &index.sorted;
    mix(&mut state, ids.len() as u64);
    for &(id, entity) in ids {
        mix(&mut state, u64::from(id.0));
        if let Some(ties) = app.world().get::<Ties>(entity) {
            mix(&mut state, ties.0.len() as u64);
            for tie in &ties.0 {
                let target = ids
                    .iter()
                    .find(|(_, e)| *e == tie.target)
                    .map_or(u64::MAX, |(id, _)| u64::from(id.0));
                mix(&mut state, target);
                mix(&mut state, u64::from(tie.affinity.to_bits()));
                mix(&mut state, u64::from(tie.trust.to_bits()));
                mix(&mut state, u64::from(tie.respect.to_bits()));
                mix(&mut state, u64::from(tie.familiarity_peak.to_bits()));
            }
        }
        if let Some(opinions) = app.world().get::<Opinions>(entity) {
            mix(&mut state, opinions.0.len() as u64);
            for opinion in &opinions.0 {
                mix(&mut state, u64::from(opinion.value.to_bits()));
            }
        }
        if let Some(knowledge) = app.world().get::<Knowledge>(entity) {
            mix(&mut state, knowledge.0.len() as u64);
            for known in &knowledge.0 {
                mix(&mut state, u64::from(known.rumor.0));
                mix(&mut state, u64::from(known.belief.to_bits()));
                mix(&mut state, known.spreader as u64);
            }
        }
        if let Some(needs) = app.world().get::<Needs>(entity) {
            mix(&mut state, needs.0.len() as u64);
            for level in &needs.0 {
                mix(&mut state, u64::from(level.to_bits()));
            }
        }
        // Personality is hashed too: it is the one model the seed reaches directly, through
        // `Personality::sample`, so a hash that skipped it could not tell two seeds apart until the
        // society had run long enough for a draw to change something else.
        if let Some(personality) = app.world().get::<Personality>(entity) {
            for trait_value in personality.as_array() {
                mix(&mut state, u64::from(trait_value.to_bits()));
            }
        }
        // Position and the current activity are hashed as well. A `Wander` decision draws its point
        // from `SocietyRng`, so position is the first place a seed difference shows up: a hash
        // limited to ties, opinions, knowledge and needs would call two different seeds identical
        // while their people were standing in different places doing different things.
        if let Some(transform) = app.world().get::<Transform>(entity) {
            mix(&mut state, u64::from(transform.translation.x.to_bits()));
            mix(&mut state, u64::from(transform.translation.z.to_bits()));
        }
        if let Some(doing) = app.world().get::<bevy_society::Doing>(entity) {
            mix(&mut state, u64::from(doing.activity.0));
            mix(&mut state, u64::from(doing.arrived_day.is_some()));
        }
    }
    state
}

fn run(app: &mut App, ticks: usize) -> u64 {
    for _ in 0..ticks {
        app.update();
    }
    // A society that never ticked would hash identically under any seed, so the clock is part of the
    // evidence that this test measured anything at all.
    assert!(
        app.world().resource::<SocietyClock>().tick > (ticks as u64) / 2,
        "the society must actually advance"
    );
    hash(app)
}

#[test]
fn the_same_seed_gives_the_same_society() {
    let mut first = common::village::village(common::village::village_defs(), 7, 24);
    let mut second = common::village::village(common::village::village_defs(), 7, 24);
    let a = run(&mut first, 3000);
    let b = run(&mut second, 3000);
    assert_eq!(
        a, b,
        "the same seed and spawn sequence give the same society"
    );
    assert_ne!(a, 0, "the hash covers a non-empty society");
}

#[test]
fn a_different_seed_gives_a_different_society() {
    // The seed reaches the hashed state through the per-tick draws: personality sampling at spawn
    // and, once the behaviour plugin decides activities, one draw per decision. Until then two
    // villages of the same spawn sequence hash alike, so this asserts the whole path is seeded.
    let mut seven = common::village::village(common::village::village_defs(), 7, 24);
    let mut eight = common::village::village(common::village::village_defs(), 8, 24);
    let a = run(&mut seven, 3000);
    let b = run(&mut eight, 3000);
    assert_ne!(a, b, "the seed is part of the society");
}
