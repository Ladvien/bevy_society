# Dialogue

The `dialogue` feature (on by default) turns barks into `bevy_speech_bubbles` balloons and lets a
player hold a conversation with a villager. Without it, `SocietyBark` is still written and a game can
render it itself, so a host is never forced to take the dependency.

## Set it up

The host adds the speech-bubbles plugin, because it is generic over the camera marker:

```rust,ignore
app.add_plugins(bevy_speech_bubbles::SpeechBubblesPlugin::<MyCamera>::default());
```

`DialoguePlugin` checks for it in `finish()` and panics with
`bevy_society: DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera> added to the app`
when it is missing. That is one of the crate's two sanctioned panics: a silent no-balloon app would be
harder to find.

If a host needs the society without dialogue — a headless test, say — disable the plugin rather than
the feature:

```rust,ignore
app.add_plugins(SocietyPlugins.build().disable::<DialoguePlugin>());
```

## Barks

The society writes `SocietyBark { speaker, text, thought, feeling }`. With the feature on, each one
becomes a balloon:

| `SocietyBark` | speech_bubbles |
|---|---|
| `speaker` | the speaker key `society:<PersonId>` |
| `thought: false` | `BubbleKind::Speech` |
| `thought: true` | `BubbleKind::Thought` |
| `feeling` | `Emotion::Neutral` / `Joy` / `Anger` / `Sadness` / `Fear` / `Calm` |

Pacing is speech_bubbles' `BarkQueue`, so two people talking at once do not stack balloons.

Speakers are registered automatically: the plugin watches `PersonId` being added (core adds it after
`Person`, so watching `Person` would miss the id) and inserts
`Speakers["society:<id>"] = Speaker { entity, name, portrait: None }` with the person's `Name`, or
`"someone"`. It removes the entry when the person despawns.

## Talking to the player

Write `TalkTo { npc, player }`:

```rust,ignore
app.world_mut().write_message(TalkTo { npc: bram, player });
```

The plugin then:

1. Yields if `ConversationLock` is present — a conversation is already running — logging with
   `debug!` and doing nothing.
2. Picks the first `ConversationRuleDef` by `priority` descending and then definition order whose
   `if` holds, with `actor` = the non-player person and `target` = the player. A conditioned rule with
   no snapshot yet is skipped; an unconditioned rule can still match.
3. Binds `Speakers["npc"]` and `Speakers["player"]` to the two people, stores `ActiveTalk`, and writes
   `StartConversation { id }`.

If no rule matches it logs `bevy_society: no conversation rule matched for <name>` and starts nothing.

## Conversations and node effects

The conversations themselves live in the host's `DialogueScript`, where the two speakers are called
`npc` and `player`:

```ron
(
    conversations: {
        "greet": (
            start: "hello",
            nodes: {
                "hello": (
                    speaker: "npc",
                    line: "Morning, {player}.",
                    next: "ask",
                ),
                "ask": (
                    speaker: "npc",
                    line: "Did you hear about Bram?",
                    choices: [
                        (text: "He is a good man.", next: "compliment"),
                        (text: "I never trusted him.", next: "insult"),
                    ],
                ),
                "compliment": (speaker: "player", line: "He is a good man."),
                "insult": (speaker: "player", line: "I never trusted him."),
            },
        ),
    },
)
```

Validate it once with `bevy_speech_bubbles::validate_script` before inserting it: a malformed script is
a loud error at load rather than a half-broken conversation at runtime.

A `NodeEffectDef` runs when a node is entered:

```ron
(node_effects: [
    (conversation: "greet", node: "compliment", then: "fx.adjust(actor, target, \"affinity\", 0.1)"),
    (conversation: "greet", node: "insult", then: "fx.adjust(actor, target, \"affinity\", -0.3)"),
])
```

The effect's `fx` writes are queued like any other script's and applied at the end of the next tick, so
a choice cannot change what the same conversation reads.

When the conversation ends, both people record a `MemoryKind::Talked` memory (valence 0.1, importance
0.4) and gain exposure toward each other, so talking to the player is social contact like any other.

### Checking the two files agree

A conversation rule, a node effect or a `DialogueNode` trigger that names a conversation or node the `DialogueScript` lacks never fires. The plugin logs each one at `error!` when either resource changes. To get them as data (an editor marking the offending rows), call `dialogue_problems(&defs, &script)`: it returns a `DialogueProblem { kind, owner, conversation, node }` per dangling reference, in definition order, empty when the two agree.

## Scripts can open a conversation

`fx.converse(npc, player)` queues a conversation for the next application point. Without the feature it
raises the runtime error `fx.converse needs the \`dialogue\` feature` rather than silently doing
nothing — a script must not look like it worked when it did not.

## What it does not do

- The crate chooses the conversation, not the lines: every line is the host's.
- No lip sync, no voice, no portraits beyond a `Handle<Image>` passed through.
- No conversation-level memory of what was said before; a node effect is the hook for that, through
  `fx.remember`.
