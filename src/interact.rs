//! Game-facing social messages: the [`Interact`] request and the [`SocietyBark`] line.
//!
//! The exchange that answers an [`Interact`] runs in the interaction system (plan D7). This module
//! defines the messages and the mapping from a person's emotions to the [`Feeling`] a bark carries.

use bevy::ecs::entity::Entity;
use bevy::ecs::message::Message;
use bevy::reflect::Reflect;

use crate::mind::{EmotionKind, Emotions};

/// Strongest-emotion intensity below which a person shows no feeling.
pub const FEELING_THRESHOLD: f32 = 0.2;

/// Request that `actor` performs the named interaction on `target`.
///
/// Written by games and by script effects (`fx.interact`). The interaction names an
/// `InteractionDef`; an unknown name is logged and ignored. The target need not be idle, and an
/// autonomous target's current activity is not cancelled.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct Interact {
    /// The person who starts the exchange.
    pub actor: Entity,
    /// The person it is directed at.
    pub target: Entity,
    /// Name of the `InteractionDef` to run.
    pub interaction: String,
}

/// A line a person says aloud, or thinks, for the game to show.
///
/// Games without `bevy_speech_bubbles` render it themselves; with the `dialogue` feature it becomes
/// a `bevy_speech_bubbles::Bark`.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct SocietyBark {
    /// The person who speaks or thinks.
    pub speaker: Entity,
    /// The rendered line, with templates already filled in.
    pub text: String,
    /// True for a thought (activity barks), false for speech.
    pub thought: bool,
    /// The speaker's feeling when the line was written.
    pub feeling: Feeling,
}

/// The feeling a bark carries, mapped from the speaker's strongest emotion.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Feeling {
    /// No emotion reaches `FEELING_THRESHOLD`.
    #[default]
    Neutral,
    /// Joy, pride, gratitude, liking or admiration.
    Joy,
    /// Anger, reproach or disliking.
    Anger,
    /// Distress or shame.
    Sadness,
    /// Fear.
    Fear,
    /// Hope.
    Calm,
}

impl Feeling {
    /// The feeling a person shows: its strongest emotion, when at least `FEELING_THRESHOLD`, mapped
    /// to a bark feeling (Joy/Pride/Gratitude/Liking/Admiration → Joy; Anger/Reproach/Disliking →
    /// Anger; Distress/Shame → Sadness; Fear → Fear; Hope → Calm).
    pub fn from_emotions(emotions: &Emotions) -> Self {
        let Some(strongest) = emotions.strongest() else {
            return Self::Neutral;
        };
        if strongest.intensity < FEELING_THRESHOLD {
            return Self::Neutral;
        }
        match strongest.kind {
            EmotionKind::Joy
            | EmotionKind::Pride
            | EmotionKind::Gratitude
            | EmotionKind::Liking
            | EmotionKind::Admiration => Self::Joy,
            EmotionKind::Anger | EmotionKind::Reproach | EmotionKind::Disliking => Self::Anger,
            EmotionKind::Distress | EmotionKind::Shame => Self::Sadness,
            EmotionKind::Fear => Self::Fear,
            EmotionKind::Hope => Self::Calm,
        }
    }
}
