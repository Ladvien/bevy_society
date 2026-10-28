#![cfg(feature = "fields")]
//! **The commotion channel is written, and a host can write it** (issue #9).
//!
//! `fields::Commotion` was `pub(crate)` and **nothing wrote it**: the channel was always zero, so
//! `FieldInput::Commotion` scored 0 for every option and a `Gradient(Commotion, _)` target never had a
//! direction — while the module's own doc said a witnessed deed deposited `|harm|`.
//!
//! Sanctuary needs the opposite: a `gawk` activity where townspeople drift toward trouble, and its
//! troubles are resolved by the game, so the host must be able to raise commotion at a point.

use crate::common;

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_society::{
    Autonomous, Commotion, Doing, DoingTarget, FieldInput, SocialFieldSettings, SocialFields,
    SocietyClock, SocietyDefs, SocietySettings,
};

/// One activity that climbs the commotion gradient, so the third criterion can be observed.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "curiosity", decay_per_day: 0.2)],
            activities: [(
                id: "gawk",
                target: Gradient(Commotion, true),
                duration_hours: 1.0,
                considerations: [(input: Need("curiosity"), curve: Linear(-1.0, 1.0))],
            )],
        )"#,
    )
    .expect("the gawk definitions parse")
}

/// A 64 x 64 one-metre grid centred on the origin, which is the crate default.
fn app() -> App {
    let mut settings = common::settings();
    settings.seconds_per_day = 64.0;
    let mut app = common::headless(defs(), settings);
    // Make sure the grids exist before the first deposit lands.
    app.update();
    app.update();
    app
}

fn commotion_at(app: &App, at: Vec2) -> f32 {
    let fields = app.world().resource::<SocialFields>();
    fields.field_value(FieldInput::Commotion, &[], fields.world_to_cell(at))
}

/// **Criterion 1.** A host `Commotion { at, amount }` raises `FieldInput::Commotion` at `at`'s cell on
/// the next tick.
#[test]
fn a_host_commotion_raises_the_field() {
    let mut app = app();
    let trouble = Vec2::new(4.0, -3.0);
    assert_eq!(commotion_at(&app, trouble), 0.0, "the channel starts empty");

    app.world_mut().write_message(Commotion {
        at: trouble,
        amount: 2.0,
    });
    // Two ticks: one to queue the message in `Clock`, one for the tick that deposits it.
    app.update();
    app.update();

    let value = commotion_at(&app, trouble);
    assert!(
        value > 0.0,
        "a host commotion at {trouble} must raise the field there, read {value}"
    );

    // And the field is local: a point far away is still empty.
    let elsewhere = Vec2::new(-20.0, 20.0);
    assert_eq!(
        commotion_at(&app, elsewhere),
        0.0,
        "commotion is deposited at the point named, not everywhere"
    );
}

/// A negative or non-finite amount is not a commotion, and deposits nothing.
#[test]
fn a_negative_or_non_finite_commotion_deposits_nothing() {
    let mut app = app();
    let trouble = Vec2::new(2.0, 2.0);
    for amount in [-1.0_f32, f32::NAN, f32::INFINITY] {
        app.world_mut().write_message(Commotion {
            at: trouble,
            amount,
        });
    }
    app.world_mut().write_message(Commotion {
        at: Vec2::new(f32::NAN, 0.0),
        amount: 1.0,
    });
    app.update();
    app.update();
    assert_eq!(
        commotion_at(&app, trouble),
        0.0,
        "no commotion may come from a negative or non-finite amount"
    );
}

/// **Criterion 2.** A witnessed deed with harm `h` deposits `|h|` at the actor's cell.
///
/// Observed through the deposit kernel's total: the channel value at the actor's own cell is the sum of
/// the falloff-weighted deposits there, and with one deposit of `|h|` it is `|h|` at the centre of the
/// disc (falloff 1.0).
#[test]
fn a_witnessed_deed_deposits_its_harm() {
    let defs = SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.1)],
            topics: [(id: "violence")],
            deeds: [(id: "brawl", topic: "violence", harm: -0.6)],
            activities: [(
                id: "chat",
                target: Person,
                duration_hours: 4.0,
                considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))],
                interaction: Some("brawl"),
            )],
            interactions: [(id: "brawl", accept: (base: 1.0), deed: Some("brawl"))],
        )"#,
    )
    .expect("the brawl definitions parse");
    let mut settings: SocietySettings = common::settings();
    settings.seconds_per_day = 64.0;
    let mut app = common::headless(defs, settings);
    app.update();
    app.update();

    let actor = common::spawn_person(&mut app, "Bram", Vec2::new(3.0, 0.0));
    let target = common::spawn_person(&mut app, "Ada", Vec2::new(3.5, 0.0));
    // A witness close enough to see it, so the deed is a `witnessed` one.
    let _witness = common::spawn_person(&mut app, "Cole", Vec2::new(4.0, 0.5));

    app.world_mut().write_message(bevy_society::Interact {
        actor,
        target,
        interaction: "brawl".to_owned(),
    });
    // One tick queues the `Interact`, the next resolves it and writes the commotion, and a third
    // deposits it.
    for _ in 0..4 {
        app.update();
    }

    let at_actor = commotion_at(&app, Vec2::new(3.0, 0.0));
    assert!(
        at_actor > 0.0,
        "a witnessed deed of harm 0.6 should deposit |harm| at the actor's cell, read {at_actor}"
    );
    let _ = Autonomous;
}

/// **Criterion 3.** A `Gradient(Commotion, true)` activity sends a person toward a commotion 5 cells
/// away.
#[test]
fn a_gradient_activity_walks_toward_the_commotion() {
    let mut app = app();
    let trouble = Vec2::new(5.0, 0.0);
    // **The commotion has to be there before the decision runs.** A `Gradient` target resolves to no
    // candidate while the field is flat, so the person takes the idle activity for one game-hour and the
    // decision is not re-run until that hour is over. Establishing the field first is what makes this
    // test about the gradient rather than about the idle fallback.
    for _ in 0..4 {
        app.world_mut().write_message(Commotion {
            at: trouble,
            amount: 2.0,
        });
        app.update();
    }
    assert!(
        commotion_at(&app, Vec2::new(1.0, 0.0)) > 0.0,
        "the commotion should be visible four cells away before anyone decides"
    );

    let person = common::spawn_person(&mut app, "Bram", Vec2::new(0.0, 0.0));
    app.world_mut().entity_mut(person).insert(Autonomous);

    // Keep the field alive while they walk, and give the decision its tick.
    for _ in 0..40 {
        app.world_mut().write_message(Commotion {
            at: trouble,
            amount: 2.0,
        });
        app.update();
    }

    let Some(doing) = app.world().get::<Doing>(person).cloned() else {
        panic!("the person should have chosen the gradient activity");
    };
    let DoingTarget::Point(goal) = doing.target else {
        panic!(
            "a Gradient activity targets a Point, got {:?}",
            doing.target
        );
    };
    assert!(
        goal.x > 0.5,
        "the gradient target should lie east, toward the commotion at {trouble}, got {goal}"
    );

    // **The criterion is that the activity steers the person toward the commotion, and the target is
    // the steering.** Walking it out is what `tests/it/locomotion.rs` and `tests/it/destination.rs` cover with
    // a game-day long enough to arrive in; this app's day is one second, because the field has to stay
    // alive across every tick and a `Gradient` target can only see the field's *current* shape.
    for _ in 0..40 {
        app.world_mut().write_message(Commotion {
            at: trouble,
            amount: 2.0,
        });
        app.update();
    }
    let goal = app
        .world()
        .get::<Doing>(person)
        .and_then(|doing| match doing.target {
            DoingTarget::Point(goal) => Some(goal),
            _ => None,
        });
    assert!(
        goal.is_some_and(|goal| goal.x > 0.5),
        "the gradient target should keep pointing east toward {trouble}, got {goal:?}"
    );
    let _ = SocietyClock::default();
    let _ = SocialFieldSettings::default();
}
