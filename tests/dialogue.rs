//! Conversations and barks through `bevy_speech_bubbles` (plan D11). The app is built here rather
//! than through `common::headless`, because these tests need the speech bubbles plugin the
//! headless harness leaves out.

#![cfg(feature = "dialogue")]

mod common;

use std::time::Duration;

use bevy::MinimalPlugins;
use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::ecs::message::Messages;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::math::Vec2;
use bevy::prelude::{Image, Mesh, StandardMaterial};
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;
use bevy_society::{Feeling, PersonId, SocietyBark, SocietyDefs, SocietyPlugins, TalkTo, Ties};
use bevy_speech_bubbles::{
    Bark, BubbleKind, ConversationEnded, DialogSkin, DialogueNodeEntered, DialogueScript,
    Presenter, Speaker, Speakers, SpeechBubblesPlugin, validate_script,
};

use bevy::prelude::*;

#[derive(Component)]
struct TestCamera;

/// Three conversations: two greetings that a rule picks between, and one with a node effect target.
const SCRIPT: &str = r#"
(
  conversations: {
    "greet": (
      start: "hi",
      nodes: {
        "hi": Line(speaker: "narrator", text: "Hello.", next: None),
      },
    ),
    "greet_friend": (
      start: "hi",
      nodes: {
        "hi": Line(speaker: "narrator", text: "Friend!", next: None),
      },
    ),
    "t": (
      start: "hello",
      nodes: {
        "hello": Line(speaker: "narrator", text: "Hi.", next: Some("insult")),
        "insult": Line(speaker: "narrator", text: "Rude.", next: None),
      },
    ),
  },
)
"#;

fn skin() -> DialogSkin {
    let slicer = bevy::sprite::TextureSlicer {
        border: BorderRect::all(4.0),
        ..default()
    };
    DialogSkin {
        frame: Handle::default(),
        frame_slicer: slicer.clone(),
        paper: Handle::default(),
        name_tag: Handle::default(),
        name_tag_slicer: slicer,
        caret: [Handle::default(), Handle::default()],
        body_font: TextFont::default(),
        title_font: TextFont::default(),
        text_color: Color::BLACK,
        choice_color: Color::BLACK,
        choice_hover_color: Color::WHITE,
        padding: 8.0,
        width: 320.0,
        portrait_size: 48.0,
        chars_per_second: 0.0,
        bottom_margin: 16.0,
    }
}

/// Definitions with the given conversation rules and node effects, as RON list bodies.
fn defs(conversations: &str, node_effects: &str) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        "(conversations: [{conversations}], node_effects: [{node_effects}])"
    ))
    .expect("the definitions parse")
}

/// A finished app with the whole society, speech bubbles on a dialog box, and the script loaded.
fn app(defs: SocietyDefs) -> App {
    let script: DialogueScript = ron::from_str(SCRIPT).expect("script parses");
    validate_script(&script).expect("script validates");
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 64.0,
        )))
        .insert_resource(common::settings())
        .insert_resource(defs)
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<bevy::input::mouse::MouseButton>>()
        .add_plugins(SpeechBubblesPlugin::<TestCamera>::default())
        .add_plugins(SocietyPlugins)
        .insert_resource(script)
        .insert_resource(Presenter::DialogBox)
        .insert_resource(skin());
    app.world_mut().resource_mut::<Speakers>().0.insert(
        "narrator".into(),
        Speaker {
            entity: None,
            name: "Narrator".into(),
            portrait: None,
        },
    );
    app.finish();
    app.cleanup();
    app
}

/// Advances `updates` updates, collecting every `T` written during them.
fn updates_collecting<T: Message>(app: &mut App, updates: usize) -> Vec<T> {
    let mut out = Vec::new();
    for _ in 0..updates {
        app.update();
        out.extend(app.world_mut().resource_mut::<Messages<T>>().drain());
    }
    out
}

/// Presses and releases a key over one update, the way a player's keyboard does.
fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
}

fn set_affinity(app: &mut App, from: Entity, to: Entity, value: f32) {
    app.world_mut()
        .get_mut::<Ties>(from)
        .expect("the person has ties")
        .entry_with(to, value);
}

fn affinity(app: &App, from: Entity, to: Entity) -> f32 {
    app.world()
        .get::<Ties>(from)
        .expect("the person has ties")
        .get(to)
        .expect("the tie exists")
        .affinity
}

/// The conversation rules: a friend greeting above the plain one, which matches everyone.
const GREETINGS: &str = r#"
    (conversation: "greet_friend", priority: 2, if: Some("ctx.affinity(actor, target) > 0.5")),
    (conversation: "greet", priority: 0),
"#;

#[test]
fn a_talk_between_friends_starts_the_friend_conversation() {
    let mut app = app(defs(GREETINGS, ""));
    let a = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    let b = common::spawn_person(&mut app, "Bram", Vec2::new(1.0, 0.0));
    set_affinity(&mut app, a, b, 0.9);
    // A snapshot exists after the first tick; the conditioned rule needs one.
    common::run(&mut app, 2);

    app.world_mut().write_message(TalkTo { npc: a, player: b });
    let entered = updates_collecting::<DialogueNodeEntered>(&mut app, 2);

    assert!(
        entered
            .iter()
            .any(|e| e.conversation == "greet_friend" && e.node == "hi"),
        "{entered:?}"
    );
    assert!(
        entered.iter().all(|e| e.conversation != "greet"),
        "{entered:?}"
    );
}

#[test]
fn walking_to_an_insult_node_lowers_affinity_through_its_node_effect() {
    let effects = r#"
        (conversation: "t", node: "insult", then: "fx.adjust(actor, target, \"affinity\", -0.3)"),
    "#;
    let rules = r#"(conversation: "t", priority: 0),"#;
    let mut app = app(defs(rules, effects));
    let a = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    let b = common::spawn_person(&mut app, "Bram", Vec2::new(1.0, 0.0));
    set_affinity(&mut app, a, b, 0.5);
    common::run(&mut app, 2);

    app.world_mut().write_message(TalkTo { npc: a, player: b });
    common::run(&mut app, 2);
    let before = affinity(&app, a, b);

    tap(&mut app, KeyCode::Space);
    let entered: Vec<DialogueNodeEntered> = app
        .world_mut()
        .resource_mut::<Messages<DialogueNodeEntered>>()
        .drain()
        .collect();
    assert!(entered.iter().any(|e| e.node == "insult"), "{entered:?}");

    // Node effects are queued in Update and apply at the next society tick.
    common::run(&mut app, 2);
    let after = affinity(&app, a, b);
    assert!(
        (before - after - 0.3).abs() < 0.01,
        "affinity went from {before} to {after}"
    );
}

#[test]
fn a_society_bark_is_forwarded_under_the_speakers_person_key() {
    let mut app = app(defs("", ""));
    let a = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    common::run(&mut app, 1);
    let id = app
        .world()
        .get::<PersonId>(a)
        .expect("the person has an id")
        .0;

    app.world_mut().write_message(SocietyBark {
        speaker: a,
        text: "Hi there".into(),
        thought: false,
        feeling: Feeling::Joy,
    });
    let barks = updates_collecting::<Bark>(&mut app, 1);

    assert_eq!(barks.len(), 1, "{barks:?}");
    assert_eq!(barks[0].speaker, format!("society:{id}"));
    assert_eq!(barks[0].text, "Hi there");
    assert_eq!(barks[0].kind, BubbleKind::Speech);
}

#[test]
fn despawning_a_person_removes_its_speaker() {
    let mut app = app(defs("", ""));
    let a = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    common::run(&mut app, 1);
    let key = format!(
        "society:{}",
        app.world()
            .get::<PersonId>(a)
            .expect("the person has an id")
            .0
    );
    assert!(app.world().resource::<Speakers>().0.contains_key(&key));

    app.world_mut().entity_mut(a).despawn();
    common::run(&mut app, 1);

    assert!(!app.world().resource::<Speakers>().0.contains_key(&key));
}

#[test]
fn a_finished_conversation_is_reported_and_frees_the_talk() {
    let rules = r#"(conversation: "greet", priority: 0),"#;
    let mut app = app(defs(rules, ""));
    let a = common::spawn_person(&mut app, "Ada", Vec2::ZERO);
    let b = common::spawn_person(&mut app, "Bram", Vec2::new(1.0, 0.0));
    common::run(&mut app, 1);

    app.world_mut().write_message(TalkTo { npc: a, player: b });
    common::run(&mut app, 2);
    tap(&mut app, KeyCode::Space);
    let ended: Vec<ConversationEnded> = app
        .world_mut()
        .resource_mut::<Messages<ConversationEnded>>()
        .drain()
        .collect();
    assert!(ended.iter().any(|e| e.id == "greet"), "{ended:?}");

    // The talk is free again: a second request starts a new conversation.
    app.world_mut().write_message(TalkTo { npc: a, player: b });
    let entered = updates_collecting::<DialogueNodeEntered>(&mut app, 2);
    assert!(
        entered.iter().any(|e| e.conversation == "greet"),
        "{entered:?}"
    );
}

#[test]
#[should_panic(
    expected = "bevy_society: DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera> added to the app"
)]
fn dialogue_without_speech_bubbles_panics_at_finish() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .insert_resource(common::settings())
        .insert_resource(SocietyDefs::default())
        .add_plugins(SocietyPlugins);
    app.finish();
}
