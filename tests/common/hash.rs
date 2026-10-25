//! **One hash of a whole society**, shared by every test that has to say "these two runs are the
//! same society".
//!
//! The hash walks `PersonIndex.sorted` and reads each person's ties, opinions, knowledge, needs,
//! personality, position and current activity, so it covers every model the crate keeps per person.
//! It hashes raw `f32` bits rather than a rounded value: a determinism guarantee that survives
//! rounding is not the one the crate makes.

#![allow(dead_code)]

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::transform::components::Transform;
use bevy_society::{Knowledge, Needs, Opinions, PersonId, PersonIndex, Personality, Ties};

/// FNV-1a 64 over the whole society. Order is `PersonIndex.sorted`, so the hash is stable only when
/// the society is.
pub fn hash(app: &App) -> u64 {
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
