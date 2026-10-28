//! **An idle person should be mildly pleased, not neutral or sad.**
//!
//! `Mood::step` returns a person to its personality baseline, and `pad::baseline` is **zero** for a
//! neutral personality — which is what `Personality::sample` produces, since it averages three draws and
//! traits centre on 0.5. So a person with no emotions at all settles at pleasure 0, and because
//! `pull_per_day` is 4.0 against `return_per_day` 1.0, that zero is a **ceiling**: measured on
//! `village-evening`, pleasure's maximum over every person and every tick is exactly **+0.000**, and
//! nobody is ever pleased.
//!
//! The evaluative space model names the missing term. Cacioppo, Cacioppo & Gollan 2013, *The negativity
//! bias: Conceptualization, quantification, and individual differences*, DOI
//! `10.1017/S0140525X13002537`:
//!
//! > "the intercept for the positive activation function (i.e., the approach motivation **at zero
//! > input**) is higher than the intercept for the negative activation function (producing the
//! > **positivity offset**), and the gain for the negative activation function is higher than for the
//! > positivity activation function (producing the **negativity bias**)."
//!
//! The crate has the negativity *gain* — anger from an insult is 0.5, joy from a compliment 0.3, and
//! distress outweighs joy 3:1 by count — and no positivity *intercept* at all. This file pins the
//! intercept.
use crate::common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Mood, Person, SocietyDefs};
use common::{headless, settings, spawn_person};

/// Runs one person in an empty field — no interactions, no emotions — and reports its mood.
fn idle_mood(positivity_offset: f32) -> Mood {
    let mut day = settings();
    day.seconds_per_day = 240.0;
    day.tuning.positivity_offset = positivity_offset;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), day);
    let person = spawn_person(&mut app, "P0", Vec2::ZERO);
    app.world_mut()
        .entity_mut(person)
        .insert((Autonomous, Person));
    // Two updates: the first sizes the person's state.
    app.update();
    app.update();
    for _ in 0..2000 {
        app.update();
    }
    app.world()
        .get::<Mood>(person)
        .copied()
        .expect("the mind plugin gave the person a mood")
}

#[test]
fn a_person_with_no_emotions_rests_above_neutral() {
    let with = idle_mood(0.1);
    println!(
        "positivity offset 0.1 -> idle pleasure {:+.4}",
        with.pleasure
    );
    assert!(
        with.pleasure > 0.0,
        "with no emotions at all a person settled at pleasure {:+.4}. The positivity offset is the \
         approach motivation at zero input, and it must leave the resting point above neutral",
        with.pleasure
    );
    // **Not** the full 0.1: the emotion pull is 4:1 against the return, so any live emotion dilutes the
    // resting point — at intensity `I` the steady state is `back * offset / (pull * I + back)`, which is
    // 0.056 at I = 0.2 and 0.020 at I = 1.0. A person alone in a field still has a little, so this only
    // checks the sign and that the offset is what produced it.
    assert!(
        with.pleasure < 0.1 + 1e-3,
        "the offset must not exceed its own value: {:+.4}",
        with.pleasure
    );
}

#[test]
fn a_zero_offset_leaves_the_old_behaviour() {
    let none = idle_mood(0.0);
    println!(
        "positivity offset 0.0 -> idle pleasure {:+.4}",
        none.pleasure
    );
    assert!(
        none.pleasure.abs() < 0.02,
        "an offset of zero must leave the previous neutral resting point: {:+.4}",
        none.pleasure
    );
}
