//! **What the person a deed was done to keeps of it.**
//!
//! `patient_reaction` moved trust and emotions and no affinity at all until 2026-10-27-03, and the
//! consequence was measured rather than argued: on `village-evening` (seed 7, 12 000 ticks) every accepted
//! `insult` left the victim's affinity for its insulter at **+0.034** — positive, set by mere exposure, beside
//! the insulter's +0.131. A person who has just been insulted went on liking the person who insulted them.
//!
//! The write is `witness_affinity · severity`, where `severity` is the deed's harm scaled by
//! `Tuning::negativity_ratio` when it is negative: Baumeister et al. 2001 (DOI `10.1037/1089-2680.5.4.323`),
//! reporting Gottman's "bad events are on average five times as powerful as good ones".
//!
//! These tests pin the mechanism at the tie, through a host-reported deed — the path a host uses for
//! everything the society cannot see. `tests/report_deed.rs` already pins the witness side.

mod common;

use bevy::prelude::*;
use bevy_society::{Opinion, Opinions, ReportDeed, SocietyDefs, Tie, Ties, Tuning};
use common::{headless, spawn_person};

/// One helpful deed and one harmful one, both on the same topic.
fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(
        r#"(
            needs: [(id: "social", decay_per_day: 0.2)],
            topics: [(id: "kindness")],
            deeds: [
                (id: "insult", topic: "kindness", harm: -0.4),
                (id: "help", topic: "kindness", harm: 0.5),
            ],
        )"#,
    )
    .expect("the test definitions parse")
}

fn app(tuning: Tuning) -> App {
    let mut settings = common::settings();
    settings.seconds_per_day = 64.0;
    settings.tuning = tuning;
    let mut app = headless(defs(), settings);
    app.update();
    app.update();
    app
}

/// The victim's affinity for `actor`. `Tie::new` is 0.0, so a missing tie reads as neutral.
fn affinity_of(app: &App, victim: Entity, actor: Entity) -> f32 {
    app.world()
        .get::<Ties>(victim)
        .and_then(|ties| ties.get(actor))
        .copied()
        .map(|tie: Tie| tie.affinity)
        .unwrap_or(0.0)
}

/// Runs one host-reported deed with `witness` as its only observer, and returns
/// `(the victim's affinity for the actor, the witness's affinity for the actor)` immediately after the deed.
///
/// The value is read **before and after** and reported as the difference, because two people standing a metre
/// apart are also gaining affinity from mere exposure (`Tuning::mere_exposure_per_day`) on the same ticks. The
/// raw value is not the deed's effect; the step is.
/// One deed's own effect on `(the victim, the witness)`, with the three ticks' other drift subtracted.
///
/// The three people are far enough apart that mere exposure never fires, but something else still moves an
/// existing tie by about `-7e-6` over three ticks whether a deed happens or not — the same value in both runs,
/// so it is not the write. The control removes it and the numbers below are then exact.
fn deed(tuning: Tuning, which: &str) -> (f32, f32) {
    let with_deed = run(tuning.clone(), Some(which));
    let control = run(tuning, None);
    (with_deed.0 - control.0, with_deed.1 - control.1)
}

/// One run of the three-person village for three ticks, with or without a deed inside it, returning the
/// change in `(the victim's affinity for the actor, the witness's affinity for the actor)` across those ticks.
///
/// **The three people are 40 m apart so that no mere exposure happens at all.** That is what makes the raw
/// change the deed's own write: two people inside `contact_radius` gain affinity from every tick they share
/// (`Tuning::mere_exposure_per_day`, scaled by `1 - |affinity|`), and subtracting a control run does not cancel
/// it — the exposure on the ticks *after* the deed is itself a function of the affinity the deed just wrote, so
/// a control leaves a residual that is not the deed. Measured, that residual was 1.5e-5 on a write of 0.12.
/// A `ReportDeed` carries its own witness list and does not require proximity, so distance costs the test
/// nothing.
fn run(tuning: Tuning, which: Option<&str>) -> (f32, f32) {
    let (mut app, actor, patient, witness) = village(tuning);
    let before = (
        affinity_of(&app, patient, actor),
        affinity_of(&app, witness, actor),
    );

    if let Some(which) = which {
        app.world_mut().write_message(ReportDeed {
            actor,
            patient: Some(patient),
            deed: which.to_owned(),
            witnesses: vec![witness],
        });
    }
    // One tick queues it, the next resolves it, the third lets anything downstream settle.
    app.update();
    app.update();
    app.update();

    (
        affinity_of(&app, patient, actor) - before.0,
        affinity_of(&app, witness, actor) - before.1,
    )
}

/// Three people far apart, so the only thing that moves their ties is the deed.
///
/// **The witness is given a strong opinion on the topic, and the patient none.** A witness's affinity write is
/// `witness_affinity · approval`, where `approval = norm · sign(harm)` and the norm is the witness's opinion —
/// and `Opinions::value` is `0.0` for a person whose `Opinions` were never sized. So a witness with a neutral
/// opinion has an `approval` of zero and its affinity does not move, which is the model's own choice: a
/// bystander who cares nothing about kindness has nothing to judge. The patient's write reads the deed's harm
/// directly and needs no opinion, which is exactly why it is written that way.
fn village(tuning: Tuning) -> (App, Entity, Entity, Entity) {
    let mut app = app(tuning);
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(40.0, 0.0));
    let witness = spawn_person(&mut app, "Cole", Vec2::new(80.0, 0.0));
    app.update();
    // One topic in this pack, held with conviction by the witness.
    app.world_mut()
        .entity_mut(witness)
        .insert(Opinions(vec![Opinion {
            value: 1.0,
            anchor: 1.0,
            stubbornness: 1.0,
        }]));
    app.update();
    (app, actor, patient, witness)
}

/// **The defect this iteration removed.** Before it, the victim's affinity for the person who hurt them was
/// whatever mere exposure had left there, and never negative.
#[test]
fn a_victim_dislikes_the_person_who_hurt_them() {
    let (victim, _) = deed(Tuning::default(), "insult");
    assert!(
        victim < 0.0,
        "the victim of an insult must dislike its insulter: {victim}"
    );
}

/// The other direction, because a write that only ever subtracts is a different and wrong model: a benefit
/// buys liking for the person who gave it.
///
/// Asserted as the **value**, not as `> 0.0`. A bare sign check passed against the reverted code on a
/// residual of `8.9e-11`, which is float dust rather than a write — the shape of test that looks like it bites
/// and does not.
#[test]
fn a_recipient_likes_the_person_who_helped_them() {
    let tuning = Tuning::default();
    let (victim, _) = deed(tuning.clone(), "help");
    let expected = tuning.witness_affinity * 0.5;
    assert!(
        (victim - expected).abs() < 1e-4,
        "the recipient of help must like its benefactor more: {victim}"
    );
}

/// The ratio's whole point: an injury moves the person it was done to **more** than it moves a bystander.
///
/// Asserted as a ratio against the deed's own numbers rather than against a literal, so the test states the
/// model instead of restating one output: the patient's write is `witness_affinity · harm · negativity_ratio`
/// and the witness's is `witness_affinity · approval` with `approval = norm · sign(harm) = -1` here.
#[test]
fn harm_moves_its_victim_more_than_a_bystander() {
    let tuning = Tuning::default();
    let (victim, witness) = deed(tuning.clone(), "insult");

    let expected_victim = tuning.witness_affinity * -0.4 * tuning.negativity_ratio;
    let expected_witness = -tuning.witness_affinity;
    assert!(
        (victim - expected_victim).abs() < 1e-4,
        "the victim's write is the harm scaled by the ratio: expected {expected_victim}, got {victim}"
    );
    assert!(
        (witness - expected_witness).abs() < 1e-4,
        "the witness's write is unchanged by the ratio: expected {expected_witness}, got {witness}"
    );
    assert!(
        victim < witness,
        "the victim must be moved further than a bystander: {victim} against {witness}"
    );
}

/// A benefit is **unscaled** — the ratio applies to injuries and nothing else, or it is not a ratio.
#[test]
fn a_benefit_is_not_scaled_by_the_ratio() {
    let tuning = Tuning::default();
    let (victim, _) = deed(tuning.clone(), "help");
    let expected = tuning.witness_affinity * 0.5;
    assert!(
        (victim - expected).abs() < 1e-4,
        "a benefit pays `witness_affinity · harm`: expected {expected}, got {victim}"
    );
}

/// The ratio scales **the victim only**, linearly: doubling it doubles the victim's step and leaves the
/// witness's untouched.
///
/// The two sides are deliberately on different scales — a witness writes `witness_affinity · approval`, worth
/// 0.15 here because an insult against a strong opinion is `approval = -1`, while a victim writes
/// `witness_affinity · harm`, worth 0.06 because this deed's harm is `-0.4`. That is the model: a bystander
/// judges the *kind* of act, and the person it happened to is moved by *how much* it hurt. So the two are not
/// equal at any ratio, and a test claiming they are would be pinning a model the paper does not describe.
#[test]
fn the_ratio_scales_the_victim_and_not_the_witness() {
    // Two other first-order processes touch the same number on the same ticks, and both are switched off here
    // so that what is measured is the ratio and nothing else. `dyad_balance_per_day` pulls the tie toward the
    // pair's mean, so a larger write is pulled back harder; `affinity_relax_per_day` moves the tie toward zero
    // by a **constant** amount per tick whatever its size (`balance::relax_toward_zero`), so a write twice as
    // large loses the same absolute relaxation and the result is short of `2x` by that amount — measured 1.5e-5
    // on a step of 0.06. Neither is part of the claim; both are noise on top of it.
    let quiet = Tuning {
        dyad_balance_per_day: 0.0,
        affinity_relax_per_day: 0.0,
        ..Tuning::default()
    };
    let one = Tuning {
        negativity_ratio: 1.0,
        ..quiet.clone()
    };
    let two = Tuning {
        negativity_ratio: 2.0,
        ..quiet
    };
    let (victim_one, witness_one) = deed(one, "insult");
    let (victim_two, witness_two) = deed(two, "insult");

    assert!(
        (victim_two - 2.0 * victim_one).abs() < 1e-5,
        "the ratio must scale the victim linearly: {victim_one} then {victim_two}"
    );
    assert!(
        (witness_two - witness_one).abs() < 1e-6,
        "and leave the witness alone: {witness_one} then {witness_two}"
    );
}

/// `negativity_ratio = 0.0` switches the harm half off entirely: the victim takes nothing from an injury, and
/// still takes a benefit. That is the number a host sets for a society without resentment.
#[test]
fn a_ratio_of_zero_leaves_the_victim_untouched_by_harm() {
    let tuning = Tuning {
        negativity_ratio: 0.0,
        ..Tuning::default()
    };
    let (victim, _) = deed(tuning.clone(), "insult");
    assert!(
        victim.abs() < 1e-6,
        "a ratio of 0 leaves the victim's affinity alone: {victim}"
    );
    let (beneficiary, _) = deed(tuning, "help");
    assert!(
        beneficiary > 0.0,
        "and a benefit still counts: {beneficiary}"
    );
}

/// The write goes through `Tie::add_affinity` like every other one, so the ceiling bounds it: repeating a
/// theft drives the victim's affinity to `-affinity_ceiling` and no further.
#[test]
fn repeated_harm_stops_at_the_affinity_ceiling() {
    let tuning = Tuning::default();
    let (mut app, actor, patient, witness) = village(tuning.clone());

    for _ in 0..12 {
        app.world_mut().write_message(ReportDeed {
            actor,
            patient: Some(patient),
            deed: "insult".to_owned(),
            witnesses: vec![witness],
        });
        app.update();
        app.update();
    }

    let affinity = affinity_of(&app, patient, actor);
    assert!(
        (affinity + tuning.affinity_ceiling).abs() < 1e-4,
        "twelve insults must reach the floor and not pass it: {affinity} against {}",
        -tuning.affinity_ceiling
    );
}
