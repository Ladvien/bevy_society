//! **Host signals are utility inputs and a `ctx` call.**
//!
//! Sanctuary owns facts daily life must weigh — whether a person has a day job today, is posted on
//! the night watch, is grieving, is afflicted — and none of them is a need: a need decays and an
//! activity pays it, so abusing one as a flag makes the flag fade and makes the activity that pays
//! it look like the cure. `Signals` is the carrier instead: a per-person vector the crate only ever
//! reads, sized from a `signals` list exactly as `Needs` is sized from `needs`.
//!
//! The three acceptance criteria of #4 are pinned here: a `Signal("x")` consideration through a
//! `Step(0.5)` curve scores 0 at 0 and 1 at 1; `ctx.signal(actor, "x")` in a `Script` consideration
//! sees a value the host wrote; and an undeclared signal is a load-time `UnknownId`, not a silent 0.

mod common;

use bevy::app::App;
use bevy::math::Vec2;
use bevy_society::{
    Autonomous, DecisionTrace, DefsError, Doing, IDLE_ACTIVITY, Signals, SocietyClock, SocietyDefs,
    SocietySettings,
};
use common::{headless, run, settings, spawn_person};

/// Sixteen game-seconds per day, so one tick at 64 Hz is 1/1024 of a game-day and the reserved
/// one-game-hour idle expires in about 43 ticks rather than 640.
fn fast_settings() -> SocietySettings {
    SocietySettings {
        seconds_per_day: 16.0,
        ..settings()
    }
}

/// One declared signal `x` and one high-rank activity that reads it with `Step(0.5)`.
fn step_defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            signals: [(id: "x")],
            activities: [(
                id: "work",
                target: Itself,
                duration_hours: 1.0,
                rank: 1,
                considerations: [(input: Signal("x"), curve: Step(0.5))],
            )],
        )"#,
    )
    .expect("a signal consideration compiles")
}

/// The same shape, but the activity reads the signal through a Rhai expression, which is the
/// `ctx.signal(actor, "x")` path a scripted condition uses.
fn script_defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            signals: [(id: "x")],
            activities: [(
                id: "work",
                target: Itself,
                duration_hours: 1.0,
                rank: 1,
                considerations: [(input: Script("ctx.signal(actor, \"x\")"), curve: Step(0.5))],
            )],
        )"#,
    )
    .expect("a ctx.signal script compiles")
}

/// A headless app with one autonomous person and a trace, its signal `x` set to `value` before the
/// first tick — which is where a host writes it.
fn village(defs: SocietyDefs, value: f32) -> (App, bevy::ecs::entity::Entity) {
    let mut app = headless(defs, fast_settings());
    app.insert_resource(DecisionTrace::default());
    let person = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Autonomous, Signals(vec![value])));
    (app, person)
}

/// The score the last recorded decision gave `activity`, or `None` when the option did not score.
fn score_of(app: &App, person: bevy::ecs::entity::Entity, activity: &str) -> Option<f32> {
    let trace = app.world().resource::<DecisionTrace>();
    let decision = trace.last_for(person)?;
    decision
        .options
        .iter()
        .find(|option| option.activity == activity)
        .map(|option| option.score)
}

/// The signals the last recorded decision was scored with.
fn signals_of(app: &App, person: bevy::ecs::entity::Entity) -> Vec<f32> {
    app.world()
        .resource::<DecisionTrace>()
        .last_for(person)
        .map_or_else(Vec::new, |decision| decision.signals.clone())
}

#[test]
fn a_step_curve_over_a_signal_scores_zero_at_zero_and_one_at_one() {
    // **0 at 0.** Nothing scored, so the person takes the reserved idle activity and `work` appears
    // in no option — which is what a `Step(0.5)` curve returning 0 looks like in the trace.
    let (mut low, low_person) = village(step_defs(), 0.0);
    run(&mut low, 6);
    assert!(
        score_of(&low, low_person, "work").is_none(),
        "a signal of 0 through Step(0.5) must not score: {:?}",
        score_of(&low, low_person, "work")
    );
    let doing = low
        .world()
        .get::<Doing>(low_person)
        .expect("the person decided something");
    assert_eq!(doing.activity, IDLE_ACTIVITY, "so it idles");
    assert_eq!(
        signals_of(&low, low_person),
        vec![0.0],
        "the trace carries the signals the choice was scored with"
    );

    // **1 at 1.** The option scores 1.0 and is the one taken.
    let (mut high, high_person) = village(step_defs(), 1.0);
    run(&mut high, 6);
    let score = score_of(&high, high_person, "work").expect("a signal of 1 scores");
    assert!(
        (score - 1.0).abs() < 1e-6,
        "a signal of 1 through Step(0.5) must score exactly 1: {score}"
    );
    let doing = high
        .world()
        .get::<Doing>(high_person)
        .expect("the person decided something");
    assert_ne!(doing.activity, IDLE_ACTIVITY, "so it works");
    assert_eq!(signals_of(&high, high_person), vec![1.0]);
}

#[test]
fn a_script_consideration_reads_the_signal_the_host_wrote() {
    // The value the host wrote before the tick is what `ctx.signal` reports, so the consideration
    // scores 1 and the activity is taken.
    let (mut app, person) = village(script_defs(), 1.0);
    run(&mut app, 6);
    let score = score_of(&app, person, "work").expect("ctx.signal saw the host's 1.0");
    assert!(
        (score - 1.0).abs() < 1e-6,
        "ctx.signal(actor, \"x\") must report the host's value: {score}"
    );

    // **A write between two ticks reaches the very next frame's decision**, because the snapshot
    // scripts read is rebuilt in `Perceive`, after the host's write and before the decision. The
    // `Doing` is cleared so the person re-decides on the next update: one frame, one write.
    let defs = app.world().resource::<SocietyDefs>().clone();
    let mut signals = app
        .world()
        .get::<Signals>(person)
        .cloned()
        .unwrap_or_default();
    signals.set(&defs, "x", 0.0).expect("`x` is declared");
    app.world_mut()
        .entity_mut(person)
        .insert(signals)
        .remove::<Doing>();
    run(&mut app, 1);
    assert!(
        score_of(&app, person, "work").is_none(),
        "the write to 0 must be seen by the next frame's decision: {:?}",
        score_of(&app, person, "work")
    );
    assert_eq!(signals_of(&app, person), vec![0.0]);
    assert!(
        app.world().resource::<SocietyClock>().tick > 1,
        "the clock ran, so the absence above is a decision and not a missing tick"
    );
}

#[test]
fn an_undeclared_signal_is_an_unknown_id_at_load() {
    let text = r#"(activities: [(
        id: "work",
        target: Itself,
        duration_hours: 1.0,
        considerations: [(input: Signal("day_job"), curve: Step(0.5))],
    )])"#;
    let result = SocietyDefs::from_ron(text);
    assert!(
        matches!(
            result,
            Err(DefsError::UnknownId { kind: "signal", ref id, ref owner })
                if id == "day_job" && owner == "activity 'work' consideration 0"
        ),
        "an undeclared signal must be reported with its path, not scored as 0: {:?}",
        result.err()
    );
}

#[test]
fn a_duplicate_signal_is_rejected_like_a_duplicate_need() {
    // Defined twice in one file.
    let twice = SocietyDefs::from_ron(r#"(signals: [(id: "x"), (id: "x")])"#);
    assert!(
        matches!(twice, Err(DefsError::Duplicate { kind: "signal", ref id }) if id == "x"),
        "a signal defined twice in one file is a duplicate: {:?}",
        twice.err()
    );

    // Merged from two files, which is the path a host's own file over the starter pack takes.
    let mut base =
        bevy_society::DefsFile::from_ron(r#"(signals: [(id: "x")])"#).expect("the base parses");
    let other =
        bevy_society::DefsFile::from_ron(r#"(signals: [(id: "x")])"#).expect("the addition parses");
    assert!(
        matches!(base.merge(other), Err(DefsError::Duplicate { kind: "signal", ref id }) if id == "x"),
        "merging a repeated signal is a duplicate"
    );
    // And `base` is unchanged, so it still compiles with `x` in slot 0.
    let defs = SocietyDefs::compile(base).expect("the base is unchanged");
    assert_eq!(defs.signal_id("x"), Some(bevy_society::SignalId(0)));
}

#[test]
fn signals_are_interned_in_definition_order() {
    let defs = SocietyDefs::from_ron(r#"(signals: [(id: "day_job"), (id: "on_watch")])"#)
        .expect("two signals compile");
    let day_job = defs.signal_id("day_job").expect("declared");
    let on_watch = defs.signal_id("on_watch").expect("declared");
    assert_eq!(
        (day_job, on_watch),
        (bevy_society::SignalId(0), bevy_society::SignalId(1))
    );
    assert_eq!(defs.signal_name(on_watch), "on_watch");
    assert_eq!(defs.signal_id("nonesuch"), None);
}

#[test]
fn a_signal_the_host_never_wrote_reads_zero_and_a_non_finite_write_reads_zero() {
    let defs = step_defs();
    let mut signals = Signals::default();
    let id = defs.signal_id("x").expect("`x` is declared");
    assert_eq!(
        signals.get(id),
        0.0,
        "a person spawned without `Signals` reads 0.0"
    );

    signals.set_id(id, 2.5);
    assert_eq!(signals.get(id), 2.5, "a host value is stored as written");
    signals.set_id(id, f32::NAN);
    assert_eq!(
        signals.get(id),
        0.0,
        "a non-finite value stores 0.0, so a consideration never sees a NaN"
    );

    // Growth: id 3 on an empty vector leaves 0.0 in the slots it skipped.
    let far = bevy_society::SignalId(3);
    let mut grown = Signals::default();
    grown.set_id(far, 1.0);
    assert_eq!(grown.0.len(), 4, "the vector grows to fit the id");
    assert_eq!(grown.get(id), 0.0, "the slots it skipped stay 0.0");
    assert_eq!(grown.get(far), 1.0);

    // The named setter resolves through the defs and rejects an unknown name.
    let mut named = Signals::default();
    named.set(&defs, "x", 0.75).expect("`x` is declared");
    assert_eq!(named.get(id), 0.75);
    assert!(matches!(
        named.set(&defs, "nonesuch", 1.0),
        Err(DefsError::UnknownId { kind: "signal", ref id, .. }) if id == "nonesuch"
    ));
}

#[test]
fn the_signals_vector_is_sized_to_the_definitions() {
    // A person spawned with no `Signals` of its own still gets one per declared signal, because
    // `Person` requires it and `size_person_state` sizes it — the same path `Needs` takes.
    let mut app = headless(
        SocietyDefs::from_ron(r#"(signals: [(id: "x"), (id: "y")])"#).expect("two signals compile"),
        fast_settings(),
    );
    let person = spawn_person(&mut app, "Alys", Vec2::ZERO);
    run(&mut app, 3);
    let signals = app
        .world()
        .get::<Signals>(person)
        .expect("`Person` requires `Signals`");
    assert_eq!(
        signals.0,
        vec![0.0, 0.0],
        "one zero slot per declared signal, and the crate never writes them"
    );
}
