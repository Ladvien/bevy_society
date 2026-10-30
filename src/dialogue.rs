//! Conversations and barks through `bevy_speech_bubbles` (plan D11).
//!
//! Every system here runs in `Update`, not in the society sets. speech_bubbles is frame-driven:
//! its conversations open in `PreUpdate` and its barks are paced per frame, so a society tick
//! would only add latency. A `TalkTo` written during a tick is therefore handled on the next
//! frame, after that tick's `Rules` have run.

use std::cmp::Reverse;

use bevy::ecs::schedule::SystemCondition;
use bevy::prelude::*;
use bevy_speech_bubbles::{
    Bark, BubbleKind, ConversationEnded, ConversationLock, DialogueNodeEntered, DialogueScript,
    Emotion, Speaker, Speakers, StartConversation,
};

use crate::core::{
    PendingActions, Person, PersonId, PersonIndex, SocietyClock, SocietySettings,
    push_script_actions, require_core,
};
use crate::defs::{SocietyDefs, TriggerKind};
use crate::interact::{Feeling, SocietyBark};
use crate::mind::{Memories, Memory, MemoryKind};
use crate::relationships::{Ties, expose_toward};
use crate::rules::{TriggerEvent, TriggerQueue, id_of, report};
use crate::script::{Bindings, ScriptEngine, ScriptError};
use crate::snapshot::SnapshotCell;

/// Asks the society to start a conversation between two people. The host writes it, or a script
/// does through `fx.converse` (feature `dialogue`).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TalkTo {
    /// The person who starts the talk. Conversation rules bind this as `actor`.
    pub npc: Entity,
    /// The person spoken to. Conversation rules bind this as `target`.
    pub player: Entity,
}

/// The conversation this crate started and has not yet seen end.
#[derive(Debug, Clone)]
struct ActiveTalk {
    npc: Entity,
    player: Entity,
    conversation: String,
}

#[derive(Resource, Debug, Default)]
struct DialogueState {
    active: Option<ActiveTalk>,
}

/// Runs society conversations and barks through `bevy_speech_bubbles`. Add it after
/// [`SocietyPlugins`](crate::SocietyPlugins)' core, and add `SpeechBubblesPlugin::<YourCamera>`
/// to the app: this plugin checks for it in `finish()`.
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TalkTo>()
            .init_resource::<DialogueState>()
            .add_observer(insert_speaker)
            .add_observer(remove_speaker)
            .add_systems(
                Update,
                (
                    talk_to,
                    run_node_effects,
                    finish_conversations,
                    forward_barks,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                check_references.run_if(
                    resource_changed::<SocietyDefs>.or_eager(resource_changed::<DialogueScript>),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "DialoguePlugin");
        if !app.world().contains_resource::<Speakers>()
            || !app.world().contains_resource::<DialogueScript>()
        {
            panic!(
                "bevy_society: DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera> added to the app"
            );
        }
    }
}

/// The `Speakers` key of a person's barks.
fn speaker_key(id: PersonId) -> String {
    format!("society:{}", id.0)
}

/// Registers a speaker as soon as a person gets its id. Barks need the entry, and a bark for a
/// missing key is dropped by speech_bubbles.
fn insert_speaker(
    on: On<Add<PersonId>>,
    people: Query<(&PersonId, Option<&Name>)>,
    mut speakers: ResMut<Speakers>,
) {
    let Ok((id, name)) = people.get(on.entity) else {
        return;
    };
    speakers.0.insert(
        speaker_key(*id),
        Speaker {
            entity: Some(on.entity),
            name: name.map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned()),
            portrait: None,
        },
    );
}

/// Removes a person's speaker when the person is despawned.
fn remove_speaker(
    on: On<Remove<Person>>,
    people: Query<&PersonId>,
    mut speakers: ResMut<Speakers>,
) {
    if let Ok(id) = people.get(on.entity) {
        speakers.0.remove(&speaker_key(*id));
    }
}

fn display_name(names: &Query<Option<&Name>>, entity: Entity) -> String {
    names
        .get(entity)
        .ok()
        .flatten()
        .map_or_else(|| "someone".to_owned(), |n| n.as_str().to_owned())
}

/// Starts a conversation for each [`TalkTo`] while none is open. Ignored talks are logged at
/// `debug!`, because a host that talks to people every frame would otherwise flood the log.
#[expect(
    clippy::too_many_arguments,
    reason = "the caller owns this state anyway; bundling it into a struct would hide the read/write split"
)]
fn talk_to(
    mut talks: MessageReader<TalkTo>,
    mut starts: MessageWriter<StartConversation>,
    mut errors: MessageWriter<ScriptError>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    snapshot: Res<SnapshotCell>,
    engine: Res<ScriptEngine>,
    lock: Option<Res<ConversationLock>>,
    mut state: ResMut<DialogueState>,
    mut speakers: ResMut<Speakers>,
    names: Query<Option<&Name>>,
) {
    for talk in talks.read() {
        // `ActiveTalk` also covers the frame between writing `StartConversation` and the lock
        // appearing, so two talks in one batch cannot both start.
        if lock.is_some() || state.active.is_some() {
            debug!("bevy_society: TalkTo ignored, a conversation is already open");
            continue;
        }
        let Some(conversation) =
            pick_conversation(&defs, &engine, &snapshot, &index, *talk, &mut errors)
        else {
            warn!(
                "bevy_society: no conversation rule matched for {}",
                display_name(&names, talk.npc)
            );
            continue;
        };
        speakers.0.insert(
            "npc".to_owned(),
            Speaker {
                entity: Some(talk.npc),
                name: display_name(&names, talk.npc),
                portrait: None,
            },
        );
        speakers.0.insert(
            "player".to_owned(),
            Speaker {
                entity: Some(talk.player),
                name: display_name(&names, talk.player),
                portrait: None,
            },
        );
        state.active = Some(ActiveTalk {
            npc: talk.npc,
            player: talk.player,
            conversation: conversation.clone(),
        });
        starts.write(StartConversation { id: conversation });
    }
}

/// The first conversation rule whose `if` holds, by priority (highest first) and then definition
/// order. A rule with a condition needs a society snapshot, so before the first tick only
/// unconditioned rules can match. A condition that errors counts as false.
fn pick_conversation(
    defs: &SocietyDefs,
    engine: &ScriptEngine,
    snapshot: &SnapshotCell,
    index: &PersonIndex,
    talk: TalkTo,
    errors: &mut MessageWriter<ScriptError>,
) -> Option<String> {
    let mut order: Vec<usize> = (0..defs.conversations.len()).collect();
    // `sort_by_key` is stable, so equal priorities keep definition order.
    order.sort_by_key(|&i| Reverse(defs.conversations[i].priority));
    let bindings = Bindings {
        actor: id_of(index, Some(talk.npc)),
        target: id_of(index, Some(talk.player)),
        ..Bindings::default()
    };
    for i in order {
        let rule = &defs.conversations[i];
        let Some(condition) = rule.condition else {
            return Some(rule.conversation.clone());
        };
        let Some(snap) = snapshot.0.as_ref() else {
            continue;
        };
        match engine.eval_bool(&defs.scripts, condition, snap, bindings) {
            Ok(true) => return Some(rule.conversation.clone()),
            Ok(false) => {}
            Err(err) => report(defs, condition, err, errors),
        }
    }
    None
}

/// Runs the node effects of every node the active conversation enters, and emits a
/// [`TriggerEvent::Dialogue`] for the node. The actions wait in [`PendingActions`] for the next
/// society tick, like every other script effect.
#[expect(
    clippy::too_many_arguments,
    reason = "the node effect needs the defs, the snapshot, the queue and the two people; a struct would hide which are read and which are written"
)]
fn run_node_effects(
    mut entered: MessageReader<DialogueNodeEntered>,
    defs: Res<SocietyDefs>,
    index: Res<PersonIndex>,
    snapshot: Res<SnapshotCell>,
    engine: Res<ScriptEngine>,
    state: Res<DialogueState>,
    mut pending: ResMut<PendingActions>,
    mut triggers: Option<ResMut<TriggerQueue>>,
    mut errors: MessageWriter<ScriptError>,
) {
    for msg in entered.read() {
        let Some(talk) = state.active.as_ref() else {
            continue;
        };
        if talk.conversation != msg.conversation {
            continue;
        }
        let actor = id_of(&index, Some(talk.npc));
        let target = id_of(&index, Some(talk.player));
        if let Some(queue) = triggers.as_mut() {
            queue.0.push(TriggerEvent::Dialogue {
                conversation: msg.conversation.clone(),
                node: msg.node.clone(),
                actor,
                target,
            });
        }
        let bindings = Bindings {
            actor,
            target,
            ..Bindings::default()
        };
        for effect in defs
            .node_effects
            .iter()
            .filter(|e| e.conversation == msg.conversation && e.node == msg.node)
        {
            let Some(snap) = snapshot.0.as_ref() else {
                warn!(
                    "bevy_society: {} skipped on {}/{}: no society snapshot yet",
                    ScriptEngine::owner(&defs.scripts, effect.then),
                    msg.conversation,
                    msg.node
                );
                continue;
            };
            match engine.run(&defs.scripts, effect.then, snap, bindings) {
                Ok(actions) => push_script_actions(&mut pending, actions),
                Err(err) => report(&defs, effect.then, err, &mut errors),
            }
        }
    }
}

/// Ends the active talk when speech_bubbles reports it done. Both people remember the talk and
/// get exposed to each other (plan D11).
fn finish_conversations(
    mut ended: MessageReader<ConversationEnded>,
    mut state: ResMut<DialogueState>,
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    mut people: Query<(&mut Ties, &mut Memories)>,
) {
    for end in ended.read() {
        if !state
            .active
            .as_ref()
            .is_some_and(|talk| talk.conversation == end.id)
        {
            continue;
        }
        let Some(talk) = state.active.take() else {
            continue;
        };
        let now = clock.now_days;
        let day = now as f32;
        // The tick's own length, not one derived from `seconds_per_day`: under `ClockSource::Host`
        // the host's clock decides how much game time a tick carries, so a conversation's effects
        // must scale by the same `dt_days` every other model uses.
        let dt = clock.dt_days;
        let Ok(
            [
                (mut npc_ties, mut npc_memories),
                (mut player_ties, mut player_memories),
            ],
        ) = people.get_many_mut([talk.npc, talk.player])
        else {
            warn!(
                "bevy_society: conversation {} ended for people who are gone",
                end.id
            );
            continue;
        };
        for (memories, other) in [
            (&mut npc_memories, talk.player),
            (&mut player_memories, talk.npc),
        ] {
            memories.record(
                Memory {
                    day,
                    kind: MemoryKind::Talked,
                    other: Some(other),
                    valence: 0.1,
                    importance: 0.4,
                },
                now,
                settings.memory_capacity,
            );
        }
        // No faction bias here: the conversation rule already picked the pair.
        let (npc_friends, player_friends) = (
            npc_ties.friend_count(&settings.tuning),
            player_ties.friend_count(&settings.tuning),
        );
        let tuning = &settings.tuning;
        expose_toward(
            &mut npc_ties,
            talk.player,
            None,
            day,
            dt,
            npc_friends,
            tuning,
        );
        expose_toward(
            &mut player_ties,
            talk.npc,
            None,
            day,
            dt,
            player_friends,
            tuning,
        );
    }
}

/// Re-emits society barks as speech_bubbles barks. A bark from a person who has no id is dropped.
fn forward_barks(
    mut barks: MessageReader<SocietyBark>,
    mut out: MessageWriter<Bark>,
    ids: Query<&PersonId>,
) {
    for bark in barks.read() {
        let Ok(id) = ids.get(bark.speaker) else {
            continue;
        };
        out.write(Bark {
            speaker: speaker_key(*id),
            kind: if bark.thought {
                BubbleKind::Thought
            } else {
                BubbleKind::Speech
            },
            emotion: emotion(bark.feeling),
            text: bark.text.clone(),
        });
    }
}

fn emotion(feeling: Feeling) -> Emotion {
    match feeling {
        Feeling::Neutral => Emotion::Neutral,
        Feeling::Joy => Emotion::Joy,
        Feeling::Anger => Emotion::Anger,
        Feeling::Sadness => Emotion::Sadness,
        Feeling::Fear => Emotion::Fear,
        Feeling::Calm => Emotion::Calm,
    }
}

fn check_references(defs: Res<SocietyDefs>, script: Res<DialogueScript>) {
    for problem in dialogue_problems(&defs, &script) {
        error!("bevy_society: {problem}");
    }
}

/// What kind of definition names a conversation the `DialogueScript` does not hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueReferenceKind {
    /// A conversation rule (`ConversationRuleDef`).
    Rule,
    /// A node effect (`NodeEffectDef`).
    NodeEffect,
    /// A trigger that fires when a node is entered (`TriggerOn::DialogueNode`).
    Trigger,
}

/// One definition that names a conversation or node the `DialogueScript` does not define.
///
/// Returned by [`dialogue_problems`]; `Display` is the line the plugin logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueProblem {
    /// Which kind of definition holds the reference.
    pub kind: DialogueReferenceKind,
    /// The definition's name: the trigger id, or the script owner (`conversation rule 2`,
    /// `node effect t/hello`) a Rhai error would also report.
    pub owner: String,
    /// The conversation it names.
    pub conversation: String,
    /// The node it names; `None` for a conversation rule.
    pub node: Option<String>,
}

impl std::fmt::Display for DialogueProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target = match &self.node {
            Some(node) => format!("{}/{node}", self.conversation),
            None => self.conversation.clone(),
        };
        write!(
            f,
            "{} names {target}, which bevy_speech_bubbles' DialogueScript does not define",
            self.owner
        )
    }
}

/// Every conversation rule, node effect and node-entered trigger in `defs` that names a
/// conversation or node `script` does not define, in definition order. Empty when the two agree.
///
/// The plugin logs these once when either changes; an editor calls it directly to mark the
/// offending rows while a conversation graph is being authored.
pub fn dialogue_problems(defs: &SocietyDefs, script: &DialogueScript) -> Vec<DialogueProblem> {
    let node_missing = |conversation: &str, node: &str| {
        script
            .conversation(conversation)
            .is_none_or(|c| !c.nodes.contains_key(node))
    };
    let mut out = Vec::new();
    for rule in &defs.conversations {
        if script.conversation(&rule.conversation).is_none() {
            out.push(DialogueProblem {
                kind: DialogueReferenceKind::Rule,
                owner: rule.condition.map_or_else(
                    || "a conversation rule".to_owned(),
                    |id| ScriptEngine::owner(&defs.scripts, id).to_owned(),
                ),
                conversation: rule.conversation.clone(),
                node: None,
            });
        }
    }
    for effect in &defs.node_effects {
        if node_missing(&effect.conversation, &effect.node) {
            out.push(DialogueProblem {
                kind: DialogueReferenceKind::NodeEffect,
                owner: ScriptEngine::owner(&defs.scripts, effect.then).to_owned(),
                conversation: effect.conversation.clone(),
                node: Some(effect.node.clone()),
            });
        }
    }
    for trigger in &defs.triggers {
        if let TriggerKind::DialogueNode { conversation, node } = &trigger.on
            && node_missing(conversation, node)
        {
            out.push(DialogueProblem {
                kind: DialogueReferenceKind::Trigger,
                owner: trigger.id.clone(),
                conversation: conversation.clone(),
                node: Some(node.clone()),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_effect_on_a_missing_node_is_reported() {
        let defs = SocietyDefs::from_ron(
            r#"(node_effects: [(conversation: "t", node: "nowhere", then: "fx.emit(\"x\", #{})")])"#,
        )
        .expect("the definitions parse");
        let script: DialogueScript = ron::from_str(
            r#"(conversations: { "t": (start: "hello", nodes: {
                "hello": Line(speaker: "npc", text: "Hi.", next: None),
            }) })"#,
        )
        .expect("the script parses");
        let errors = dialogue_problems(&defs, &script);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].to_string().contains("t/nowhere"), "{errors:?}");
    }
}
