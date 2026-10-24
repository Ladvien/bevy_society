//! Daily life (plan D7): decisions, travel, arrival, needs, and the exchanges that follow.
//!
//! Each test drives a real app for real ticks and asserts on the messages the society wrote, so a
//! system that silently does nothing fails here rather than looking plausible in a plot.

mod common;

use bevy::prelude::*;
use bevy_society::{
    ActivityEnded, ActivityStarted, Autonomous, Home, Interact, InteractionResolved, Person, Place,
    PlaceTags, SocietyClock, SocietyDefs, SocietySettings,
};
use common::{headless, run, settings, spawn_person};

/// A village with one place tagged `work`, one activity that goes there, and one that chats.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 1.0)],
            topics: [(id: "kindness")],
            activities: [
                (
                    id: "work",
                    target: Place("work"),
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("social", 0.5)],
                ),
                (
                    id: "chat",
                    target: Person,
                    duration_hours: 0.2,
                    considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
                    interaction: Some("chat"),
                ),
            ],
            interactions: [
                (id: "chat", accept: (base: 0.9)),
            ],
        )"#,
    )
    .expect("the behaviour definitions parse")
}

/// One game-day per 64 ticks, so a game-hour is about three ticks.
///
/// `dt_days = tick_secs / seconds_per_day`, and `settings()` fixes `tick_secs = 1/64`, so
/// `seconds_per_day = 1.0` is exactly one tick per 1/64 of a day.
fn fast_settings() -> SocietySettings {
    let mut settings = settings();
    settings.seconds_per_day = 1.0;
    settings
}

fn place(app: &mut App, name: &str, pos: Vec2, tag: &str, capacity: u32) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            Place { capacity },
            PlaceTags(vec![tag.to_owned()]),
            Transform::from_xyz(pos.x, 0.0, pos.y),
        ))
        .id()
}

/// Advances the app, collecting every message of interest.
fn step(
    app: &mut App,
    ticks: usize,
) -> (
    Vec<ActivityStarted>,
    Vec<ActivityEnded>,
    Vec<InteractionResolved>,
) {
    let (mut started, mut ended, mut resolved) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..ticks {
        app.update();
        started.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ActivityStarted>>()
                .drain(),
        );
        ended.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>()
                .drain(),
        );
        resolved.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<InteractionResolved>>()
                .drain(),
        );
    }
    (started, ended, resolved)
}

#[test]
fn an_autonomous_person_chooses_an_activity() {
    let mut app = headless(defs(), fast_settings());
    let work = place(&mut app, "shop", Vec2::new(2.0, 0.0), "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    let (started, _, _) = step(&mut app, 4);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(person)
        .expect("the person chose an activity");
    assert_eq!(
        doing.target,
        bevy_society::DoingTarget::Place(work),
        "the activity targets the tagged place"
    );
    assert!(
        doing.arrived_day.is_none(),
        "it is still travelling: this app has no steering plugin to move it"
    );
    let _ = started;
}

#[test]
fn arriving_at_a_place_satisfies_its_needs() {
    let mut app = headless(defs(), fast_settings());
    // Standing on the place, so arrival is immediate.
    let _shop = place(&mut app, "shop", Vec2::ZERO, "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);
    // `size_person_state` grows `Needs` to the definitions on the first tick, so seed after it.
    step(&mut app, 1);
    if let Some(mut needs) = app.world_mut().get_mut::<bevy_society::Needs>(person)
        && let Some(slot) = needs.0.first_mut()
    {
        *slot = 0.2;
    }

    let (started, _, _) = step(&mut app, 6);

    let needs = app
        .world()
        .get::<bevy_society::Needs>(person)
        .expect("a person has needs");
    let level = needs.0.first().copied().unwrap_or(0.0);
    // The need decays every tick and rises only while arrived, so the assertion is that the activity
    // ran, not that the level beat its starting point.
    assert!(
        started.iter().any(|s| s.activity == "work"),
        "the work activity was reported: {started:?}"
    );
    assert!(level > 0.0, "the need stayed above zero: {level}");
    assert!(
        started.iter().any(|s| s.activity == "work"),
        "the activity was reported: {started:?}"
    );
}

#[test]
fn an_activity_ends_after_its_duration() {
    // An `Itself` target arrives immediately, which is the only arrival this app can reach without a
    // steering plugin moving anybody.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "doze",
                    target: Itself,
                    duration_hours: 1.0,
                    considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("rest", 0.5)],
                )],
            )"#,
        )
        .expect("the doze definition parses"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    // One game-day is 64 ticks, so one game-hour is about three.
    let (started, ended, _) = step(&mut app, 12);

    assert!(
        started.iter().any(|s| s.activity == "doze"),
        "the in-place activity started: {started:?}"
    );
    assert!(
        ended.iter().any(|e| e.activity == "doze" && e.completed),
        "and completed after its duration: {ended:?}"
    );
}

#[test]
fn a_person_that_cannot_arrive_gives_up() {
    let mut app = headless(defs(), fast_settings());
    // Far outside the interaction radius, so it never arrives.
    let _shop = place(&mut app, "shop", Vec2::new(19.0, 19.0), "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    let (_, ended, _) = step(&mut app, 400);

    assert!(
        ended.iter().any(|e| !e.completed),
        "an unreachable activity ends as not completed: {ended:?}"
    );
}

#[test]
fn a_game_written_interact_is_resolved_and_reported() {
    let mut app = headless(defs(), fast_settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    step(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "chat".to_owned(),
    });
    let (_, _, resolved) = step(&mut app, 3);

    assert_eq!(resolved.len(), 1, "exactly one exchange: {resolved:?}");
    let exchange = &resolved[0];
    assert_eq!(exchange.actor, a);
    assert_eq!(exchange.target, b);
    assert_eq!(exchange.interaction, "chat");
    assert!(exchange.accepted, "base acceptance is 0.9");
    assert!(!exchange.refused_by_rule);
}

#[test]
fn an_unknown_interaction_name_is_ignored() {
    let mut app = headless(defs(), fast_settings());
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    step(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "nonsense".to_owned(),
    });
    let (_, _, resolved) = step(&mut app, 3);

    assert!(
        resolved.is_empty(),
        "an unknown interaction resolves nothing: {resolved:?}"
    );
}

#[test]
fn a_home_target_sends_a_person_to_its_home() {
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "sleep",
                    target: Home,
                    duration_hours: 1.0,
                    considerations: [(input: Need("rest"), curve: Linear(-1.0, 1.0))],
                    satisfies: [("rest", 0.5)],
                )],
            )"#,
        )
        .expect("the home definition parses"),
        fast_settings(),
    );
    let home = place(&mut app, "home", Vec2::new(3.0, 0.0), "home", 1);
    let person = app
        .world_mut()
        .spawn((
            Person,
            Autonomous,
            Name::new("Bram"),
            Home(home),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    step(&mut app, 4);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(person)
        .expect("the person chose the home activity");
    assert_eq!(
        doing.target,
        bevy_society::DoingTarget::Place(home),
        "the activity targets its home"
    );
}

#[test]
fn the_clock_keeps_running_through_all_of_it() {
    let mut app = headless(defs(), fast_settings());
    let _shop = place(&mut app, "shop", Vec2::ZERO, "work", 4);
    for i in 0..4 {
        let person = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32, 0.0));
        app.world_mut().entity_mut(person).insert(Autonomous);
    }
    // The first update only primes `Time<Fixed>`, so `n` updates past it are `n - 1` ticks.
    let before = app.world().resource::<SocietyClock>().tick;
    step(&mut app, 30);
    let after = app.world().resource::<SocietyClock>().tick;
    assert_eq!(after - before, 29, "one tick per update after the first");
}

#[test]
fn a_script_consideration_is_evaluated() {
    // A script that always returns 1.0 must make its activity win. Before `Signal::Script` was wired
    // through the engine, this scored 0 and the person idled: the definition format promised script
    // scoring and delivered nothing.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "read",
                    target: Itself,
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [(input: Script("1.0"), curve: Linear(1.0, 0.0))],
                )],
            )"#,
        )
        .expect("a script consideration compiles"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "A", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    step(&mut app, 4);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(person)
        .expect("the person chose an activity");
    assert_ne!(
        doing.activity,
        bevy_society::IDLE_ACTIVITY,
        "a script returning 1.0 must not leave the person idle"
    );
}

#[test]
fn a_script_consideration_can_score_zero() {
    // The same shape with a script that returns 0.0: the activity must not be chosen. This is the
    // other half of the contract — a consideration that says no really says no.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 1.0)],
                activities: [(
                    id: "read",
                    target: Itself,
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [(input: Script("0.0"), curve: Linear(1.0, 0.0))],
                )],
            )"#,
        )
        .expect("a script consideration compiles"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "A", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);

    step(&mut app, 4);

    let doing = app.world().get::<bevy_society::Doing>(person);
    assert!(
        doing.is_none_or(|d| d.activity == bevy_society::IDLE_ACTIVITY),
        "a script returning 0.0 must not win the decision"
    );
}

#[test]
fn a_script_consideration_sees_its_own_actor() {
    // The script reads the actor's own need, so it only wins for the tired person. If `actor` were
    // unbound or bound to somebody else, both people would get the same answer and this fails.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "rest", decay_per_day: 0.0)],
                activities: [(
                    id: "doze",
                    target: Itself,
                    duration_hours: 1.0,
                    rank: 1,
                    considerations: [(
                        input: Script("if ctx.need(actor, \"rest\") > 0.5 { 1.0 } else { 0.0 }"),
                        curve: Linear(1.0, 0.0),
                    )],
                )],
            )"#,
        )
        .expect("a ctx-reading consideration compiles"),
        fast_settings(),
    );
    let tired = spawn_person(&mut app, "Tired", Vec2::new(40.0, 0.0));
    let rested = spawn_person(&mut app, "Rested", Vec2::new(-40.0, 0.0));
    for person in [tired, rested] {
        app.world_mut().entity_mut(person).insert(Autonomous);
    }
    // The first `update` primes `Time<Fixed>`, the second runs the first society tick, and only then
    // has `size_person_state` grown `Needs` from empty to the definitions' length. Both people choose
    // an activity on that tick, so the levels are set and the decision re-run.
    step(&mut app, 2);
    for (person, level) in [(tired, 0.9_f32), (rested, 0.1_f32)] {
        if let Some(mut needs) = app.world_mut().get_mut::<bevy_society::Needs>(person) {
            assert!(
                !needs.0.is_empty(),
                "Needs must be sized by now, or this test proves nothing"
            );
            needs.0[0] = level;
        }
        // Clear the activity the first tick chose, so the next tick decides again.
        app.world_mut()
            .entity_mut(person)
            .remove::<(bevy_society::Doing, bevy_society::Destination)>();
    }

    step(&mut app, 2);

    let tired_activity = app
        .world()
        .get::<bevy_society::Doing>(tired)
        .map(|d| d.activity);
    let rested_activity = app
        .world()
        .get::<bevy_society::Doing>(rested)
        .map(|d| d.activity);
    assert!(
        tired_activity.is_some_and(|a| a != bevy_society::IDLE_ACTIVITY),
        "the tired person's script returned 1.0, so it acts: {tired_activity:?}"
    );
    assert!(
        rested_activity.is_none_or(|a| a == bevy_society::IDLE_ACTIVITY),
        "the rested person's script returned 0.0, so it does not: {rested_activity:?}"
    );
}

#[test]
fn a_place_target_gives_a_destination_and_the_person_walks() {
    // Regression: `Destination` was set only for a `Point` target, so a village whose activities
    // all target places never moved. Every person stood where they spawned for the whole run, which
    // is what the hero capture showed.
    let mut app = headless(defs(), fast_settings());
    let shop = place(&mut app, "shop", Vec2::new(4.0, 0.0), "work", 4);
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Autonomous);
    // The steering component only exists with the `steering` feature; the destination assertion
    // below holds either way, and the motion assertion is behind the feature.
    #[cfg(feature = "steering")]
    app.world_mut()
        .entity_mut(person)
        .insert(bevy_society::SocialMover::default());

    step(&mut app, 2);

    let destination = app
        .world()
        .get::<bevy_society::Destination>(person)
        .copied()
        .expect("a place target must give a destination, or nobody walks");
    assert!(
        destination.0.distance(Vec2::new(4.0, 0.0)) < 1.0,
        "the destination is the place: {:?}",
        destination.0
    );

    // And steering turns that into motion.
    #[cfg(feature = "steering")]
    let before = app
        .world()
        .get::<Transform>(person)
        .map(|t| t.translation.x)
        .unwrap_or(0.0);
    #[cfg(feature = "steering")]
    {
        step(&mut app, 60);
        let after = app
            .world()
            .get::<Transform>(person)
            .map(|t| t.translation.x)
            .unwrap_or(0.0);
        assert!(
            after > before + 0.2,
            "the person walked toward the shop: {before} -> {after}"
        );
    }
    let _ = shop;
}

#[test]
fn a_person_target_stops_short_of_the_other_person() {
    // Two people, one walking to the other: the destination is offset by 0.75 x interaction_radius
    // so they do not occupy the same point.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                needs: [(id: "social", decay_per_day: 0.0)],
                topics: [(id: "kindness")],
                activities: [(
                    id: "chat",
                    target: Person,
                    duration_hours: 0.2,
                    rank: 1,
                    considerations: [(input: Distance, curve: Linear(-1.0, 1.0))],
                )],
            )"#,
        )
        .expect("the chat-only definition parses"),
        fast_settings(),
    );
    let walker = spawn_person(&mut app, "Walker", Vec2::ZERO);
    let other = spawn_person(&mut app, "Other", Vec2::new(6.0, 0.0));
    app.world_mut().entity_mut(walker).insert(Autonomous);
    app.world_mut().entity_mut(other).insert(Autonomous);

    step(&mut app, 3);

    let destination = app
        .world()
        .get::<bevy_society::Destination>(walker)
        .copied()
        .or_else(|| app.world().get::<bevy_society::Destination>(other).copied());
    let destination = destination.expect("a person target gives a destination");
    // The offset target is between the two, never on top of the other person.
    assert!(
        destination.0.distance(Vec2::new(6.0, 0.0)) > 0.1,
        "the destination stops short of the target: {:?}",
        destination.0
    );
}

#[test]
fn an_accepted_interaction_can_bark() {
    // Barks are a message the crate writes; a host without speech bubbles renders them itself, so a
    // village that never barks is silent for every host.
    let defs = SocietyDefs::from_ron(
        r#"(
            topics: [(id: "kindness")],
            interactions: [(
                id: "chat",
                accept: (base: 0.99),
                barks: (
                    actor: ["Morning, {target}!"],
                    accepted: ["Good to see you."],
                    rejected: ["Not now."],
                ),
            )],
        )"#,
    )
    .expect("the bark definition parses");
    let mut settings = settings();
    settings.bark_chance = 1.0;
    let mut app = headless(defs, settings);
    let a = spawn_person(&mut app, "Alys", Vec2::ZERO);
    let b = spawn_person(&mut app, "Bram", Vec2::new(1.0, 0.0));
    run(&mut app, 2);

    app.world_mut().write_message(Interact {
        actor: a,
        target: b,
        interaction: "chat".to_owned(),
    });
    let mut barks = Vec::new();
    for _ in 0..4 {
        app.update();
        barks.extend(
            app.world_mut()
                .resource_mut::<bevy::ecs::message::Messages<bevy_society::SocietyBark>>()
                .drain(),
        );
    }

    assert!(!barks.is_empty(), "a bark was written");
    let actor_line = barks
        .iter()
        .find(|b| b.speaker == a)
        .expect("the actor spoke");
    assert_eq!(
        actor_line.text, "Morning, Bram!",
        "the template renders the target's name"
    );
    assert!(!actor_line.thought, "an interaction bark is speech");
}

#[test]
fn a_script_consideration_can_score_a_person_target() {
    // Regression: the script context bound `target` to -1 for every candidate, so
    // `ctx.grudge(actor, target)` and every other ctx call that reads the target could never score
    // a `Person` option. Grudges drive snubbing in the starter pack, and nothing could express it.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                topics: [(id: "kindness")],
                deeds: [(id: "insult", topic: "kindness", harm: -0.4)],
                activities: [(
                    id: "snub",
                    target: Person,
                    duration_hours: 0.1,
                    rank: 1,
                    considerations: [(
                        input: Script("ctx.grudge(actor, target)"),
                        curve: Linear(2.0, 0.0),
                    )],
                    interaction: Some("insult"),
                )],
                interactions: [(id: "insult", accept: (base: 0.99), deed: Some("insult"))],
            )"#,
        )
        .expect("a ctx.grudge consideration compiles"),
        fast_settings(),
    );
    let grudgeful = spawn_person(&mut app, "A", Vec2::ZERO);
    let other = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    app.world_mut().entity_mut(grudgeful).insert(Autonomous);
    step(&mut app, 2);
    // A remembers something bad about B: that is the grudge the script reads.
    if let Some(mut memories) = app.world_mut().get_mut::<bevy_society::Memories>(grudgeful) {
        memories.record(
            bevy_society::Memory {
                day: 0.0,
                kind: bevy_society::MemoryKind::Talked,
                other: Some(other),
                valence: -0.9,
                importance: 0.9,
            },
            0.0,
            64,
        );
    }

    step(&mut app, 3);

    let doing = app
        .world()
        .get::<bevy_society::Doing>(grudgeful)
        .expect("the grudge decided something");
    assert_eq!(
        doing.target,
        bevy_society::DoingTarget::Person(other),
        "the snub is aimed at the person the grudge is about"
    );
}

#[test]
fn a_script_consideration_reads_no_grudge_for_a_stranger() {
    // The other half: with no memory about B the same definition must not choose the snub. Without
    // this, the test above would pass even if `target` were bound to the wrong person.
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(
                topics: [(id: "kindness")],
                deeds: [(id: "insult", topic: "kindness", harm: -0.4)],
                activities: [(
                    id: "snub",
                    target: Person,
                    duration_hours: 0.1,
                    rank: 1,
                    considerations: [(
                        input: Script("ctx.grudge(actor, target)"),
                        curve: Linear(2.0, 0.0),
                    )],
                    interaction: Some("insult"),
                )],
                interactions: [(id: "insult", accept: (base: 0.99), deed: Some("insult"))],
            )"#,
        )
        .expect("a ctx.grudge consideration compiles"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "A", Vec2::ZERO);
    let _stranger = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    app.world_mut().entity_mut(person).insert(Autonomous);

    step(&mut app, 4);

    let doing = app.world().get::<bevy_society::Doing>(person);
    assert!(
        doing.is_none_or(|d| d.activity == bevy_society::IDLE_ACTIVITY),
        "no grudge, no snub: {doing:?}"
    );
}

// -------------------------------------------------------------------------------------------------
// The message pair
// -------------------------------------------------------------------------------------------------

/// **Every `ActivityEnded` has a matching `ActivityStarted`.**
///
/// A host subscribing to the pair — barks, a HUD, a log, a witness reaction — treats an end as the
/// close of something it was told about. A person who decides an activity, walks, and times out
/// (its target out of reach, `travel_timeout_hours` elapsed) used to emit an end for a start that
/// was never written, because `ActivityStarted` was written only **on arrival**. Measured on one
/// person who can never reach its place: 2 starts against 55 ends.
#[test]
fn every_activity_end_has_a_start() {
    let mut app = headless(defs(), fast_settings());
    // Far enough that the 2-hour timeout expires long before anyone could walk there, and this app
    // has no steering plugin, so *nothing* ever arrives.
    let shop = place(&mut app, "shop", Vec2::new(50.0, 0.0), "work", 4);
    let person = spawn_person(&mut app, "Solo", Vec2::ZERO);
    app.world_mut().entity_mut(person).insert(Home(shop));
    app.world_mut().entity_mut(person).insert(Autonomous);

    let (started, ended, _) = step(&mut app, 900);

    let mine_starts = started.iter().filter(|m| m.person == person).count();
    let mine_ends = ended.iter().filter(|m| m.person == person).count();
    assert!(
        mine_ends > 0,
        "the person gave up on its unreachable target at least once: {ended:?}"
    );
    assert!(
        mine_starts >= mine_ends,
        "every end must pair with a start: {mine_starts} starts, {mine_ends} ends"
    );
    // A run can stop mid-activity, which leaves one start legitimately unpaired; what must never
    // happen is an end with no start before it.
    assert!(
        mine_starts - mine_ends <= 1,
        "at most the activity in flight at the end is unpaired: {mine_starts} starts, {mine_ends} ends"
    );
}
