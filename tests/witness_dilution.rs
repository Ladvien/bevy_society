//! **A deed seen by many should cost each witness less than a deed seen by one** — in every tie field, not
//! only in trust.
//!
//! `witness_reaction` applies a witnessed deed's effects at full magnitude to **every** witness within
//! perception, so the deed's aggregate cost grows linearly with the crowd. A trust *gain* from an interaction
//! reaches only the two parties, so the two move on different scales: one `chat` is `+0.10` in total while one
//! insult seen by three people is `-0.42`.
//!
//! Until 2026-10-26-13 the dilution covered **trust alone**. Affinity — the field `TieLabel::classify` reads to
//! decide `Friend` against `Rival` — and respect were written undiluted by every witness, so the crowd still
//! multiplied the part of a deed's cost that shapes the tie network and damped only the part that does not.
//!
//! The literature measures exactly this transmission. Sommerfeld, Krambeck, Semmann & Milinski 2007, *Gossip
//! as an alternative for direct observation in games of indirect reciprocity*, PNAS, DOI
//! `10.1073/pnas.0704598104`, on disk at `~/.cache/bevy_lights_corpus/markdown/10/10.1073_pnas.0704598104.md`:
//!
//! > "transmission by gossip had a **dampening effect** on the cooperation levels compared with acting on
//! > direct observation. The more extreme a gossip was, the less impact it had on the reader." (line 79)
//!
//! and, of the number of sources, citing Hess & Hagen: *"Hess and Hagen found an increase in **believability**
//! with an increasing number of sources"* (line 81) — believability, not magnitude. Information from many
//! observers is believed more and moves each recipient less, which is what a per-witness dilution models.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, Home, Person, Place, PlaceTags, SocietyDefs, Ties};
use common::{headless, settings, spawn_person};

/// The starter pack with `insult` accepted unconditionally, so a deed always fires.
fn defs_with_certain_acceptance() -> SocietyDefs {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/starter.society.ron"
    ))
    .expect("the starter pack is in the repository");
    // The `insult` interaction's accept block, replaced with a certain one.
    let patched = source.replace(
        "            id: \"insult\",\n            on_accept:",
        "            id: \"insult\",\n            accept: (base: 1.0, affinity: 0.0, trust: 0.0, agreeableness: 0.0, pleasure: 0.0, ingroup: 0.0),\n            on_accept:",
    );
    SocietyDefs::from_ron(&patched).expect("the patched pack parses")
}

/// What a witness's tie to the actor moved by, averaged over the bystanders, after one witnessed insult.
#[derive(Debug, Clone, Copy, Default)]
struct Lost {
    trust: f32,
    affinity: f32,
    respect: f32,
}

/// Puts `witnesses` bystanders within perception of two people, has the first insult the second with
/// everyone watching, and reports the mean change each bystander's tie to the actor took.
///
/// Every bystander holds a `kindness` opinion of `+0.8`, which is what makes `approval` non-zero and therefore
/// what makes the affinity and respect writes measurable at all: with a neutral opinion the approval is `0`
/// and only trust moves.
fn lost_per_witness(witnesses: usize) -> Lost {
    let defs = defs_with_certain_acceptance();
    let kind = defs
        .opinions([("kindness", 0.8)])
        .expect("the pack defines the kindness topic");
    let mut day = settings();
    day.seconds_per_day = 64.0;
    day.start_hour = 15.0;
    // **Force acceptance.** A refused insult fires no deed, so the measurement would be of the refusal
    // path instead. `accept.base = 1.0` and every other weight zero makes the draw certain.
    day.tuning.rejection_distress = 0.0;
    // **Isolate the deed's own affinity write from every other mover.** Five other systems nudge a tie's
    // affinity on the same ticks — mere exposure at `mere_exposure_per_day`, relaxation toward zero,
    // structural balance (`close_the_triad`) and the dyad balance — and together they were worth `+0.06`
    // against the deed's `-0.12` in a first draft of this fixture, so an end-to-end difference measured
    // all five and called it the deed. With these at zero the change in the tie **is** the write under
    // test, which is what lets the single-witness case be asserted exactly rather than approximately.
    day.tuning.mere_exposure_per_day = 0.0;
    day.tuning.affinity_relax_per_day = 0.0;
    day.tuning.balance_rate_per_day = 0.0;
    day.tuning.dyad_balance_per_day = 0.0;
    day.tuning.contact_gain = 0.0;
    let mut app = headless(defs, day);

    let place = app
        .world_mut()
        .spawn((
            Name::new("place"),
            Place { capacity: 32 },
            PlaceTags(vec!["gathering".to_owned()]),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    // Actor and patient together at the origin; `witnesses` more packed around them, all inside the 8 m
    // perception radius.
    let mut crowd = Vec::new();
    for i in 0..(2 + witnesses) {
        let angle = i as f32 * std::f32::consts::TAU / (2 + witnesses).max(1) as f32;
        let person = spawn_person(
            &mut app,
            &format!("P{i}"),
            Vec2::new(0.3 * angle.cos(), 0.3 * angle.sin()),
        );
        app.world_mut()
            .entity_mut(person)
            .insert((Autonomous, Person, Home(place), kind.clone()));
        crowd.push(person);
    }
    let actor = crowd[0];
    app.update();
    app.update();

    let before: Vec<(f32, f32, f32)> = crowd
        .iter()
        .map(|w| {
            app.world()
                .get::<Ties>(*w)
                .and_then(|ties| ties.get(actor))
                .map_or((0.5, 0.0, 0.0), |tie| {
                    (tie.trust, tie.affinity, tie.respect)
                })
        })
        .collect();

    // One insult by the actor against the second person, with everyone else watching.
    app.world_mut().write_message(bevy_society::Interact {
        actor,
        target: crowd[1],
        interaction: "insult".to_owned(),
    });
    for _ in 0..4 {
        app.update();
    }

    let mut lost = Lost::default();
    let mut count = 0u32;
    for (i, watcher) in crowd.iter().enumerate() {
        // Index 0 is the actor and index 1 is the **patient**, which is not a witness: `patient_reaction`
        // moves its trust and nothing else. Averaging it in halved every figure in the first draft of this
        // fixture — exactly `0.0600` against the `0.1200` the write is worth.
        if i < 2 {
            continue;
        }
        if let Some(tie) = app.world().get::<Ties>(*watcher).and_then(|t| t.get(actor)) {
            lost.trust += before[i].0 - tie.trust;
            lost.affinity += before[i].1 - tie.affinity;
            lost.respect += before[i].2 - tie.respect;
            count += 1;
        }
    }
    let n = count.max(1) as f32;
    Lost {
        trust: lost.trust / n,
        affinity: lost.affinity / n,
        respect: lost.respect / n,
    }
}

#[test]
fn more_witnesses_means_less_trust_lost_by_each() {
    let one = lost_per_witness(1);
    let five = lost_per_witness(5);
    println!(
        "trust lost per witness: with 1 bystander {:+.4}, with 5 {:+.4}",
        one.trust, five.trust
    );

    assert!(
        one.trust > 0.0,
        "a witnessed deed must cost the witness something: {:+.4}",
        one.trust
    );
    assert!(
        five.trust < one.trust,
        "the same deed seen by five takes {:+.4} from each and seen by one takes {:+.4}. \
         Without dilution the aggregate cost grows with the crowd while a good deed's reward cannot, \
         which is why trust only ever falls in a village",
        five.trust,
        one.trust
    );
}

#[test]
fn more_witnesses_means_less_affinity_lost_by_each() {
    let one = lost_per_witness(1);
    let five = lost_per_witness(5);
    println!(
        "affinity lost per witness: with 1 bystander {:+.4}, with 5 {:+.4}",
        one.affinity, five.affinity
    );

    // The fixture's witnesses hold `kindness +0.8`, so an `insult` (harm −0.4) is disapproved of and every
    // write is negative. If this is zero the fixture is not measuring the deed path at all.
    assert!(
        one.affinity > 0.0,
        "a disliked deed must move a witness's affinity: {:+.4}",
        one.affinity
    );
    assert!(
        five.affinity < one.affinity,
        "the same insult takes {:+.4} of affinity from each of five witnesses and {:+.4} from one. \
         Affinity is what `TieLabel::classify` reads, so undiluted it makes the crowd multiply exactly \
         the cost that decides friend against rival while a compliment reaches one person — measured, \
         +160.4 from 1 604 compliments against -241.2 from 7 920 witness reactions",
        five.affinity,
        one.affinity
    );
}

#[test]
fn more_witnesses_means_less_respect_lost_by_each() {
    let one = lost_per_witness(1);
    let five = lost_per_witness(5);
    println!(
        "respect lost per witness: with 1 bystander {:+.4}, with 5 {:+.4}",
        one.respect, five.respect
    );

    assert!(
        one.respect > 0.0,
        "a disliked deed must move a witness's respect: {:+.4}",
        one.respect
    );
    assert!(
        five.respect < one.respect,
        "respect is the standing others confer (`Status`), so an undiluted crowd write makes one deed seen \
         by five cost five times the standing of one seen by one: {:+.4} against {:+.4}",
        five.respect,
        one.respect
    );
}

#[test]
fn a_deed_seen_by_one_bystander_moves_the_tie_as_it_always_did() {
    // **The bit-identity guard.** `witnesses^0.5` is `1.0` at one witness, so a deed seen by exactly one
    // bystander must write exactly the undiluted numbers. Any rewrite that changes the single-witness case
    // would move every scenario and would be invisible in a 24-person village, where the mean crowd is 3-6.
    let one = lost_per_witness(1);
    // The pack's `insult` is `harm -0.4`; `kindness +0.8` makes `approval = 0.8 * -1 = -0.8`. So the affinity
    // write is `0.15 * -0.8 = -0.12` and the respect write is `0.1 * -0.8 = -0.08`, both lost.
    assert!(
        (one.affinity - 0.12).abs() < 1e-4,
        "one bystander must lose exactly the undiluted 0.15 x 0.8 = 0.12 of affinity, lost {:+.5}",
        one.affinity
    );
    assert!(
        (one.respect - 0.08).abs() < 1e-4,
        "one bystander must lose exactly the undiluted 0.10 x 0.8 = 0.08 of respect, lost {:+.5}",
        one.respect
    );
}

#[test]
fn a_witness_that_objects_still_objects() {
    // Dilution is magnitude, never direction or the decision to object. At six witnesses the exponent-0.5
    // factor is 1/sqrt(6) = 0.408, so a witness that disapproved before still disapproves now, and the
    // `NormViolated` bar still reads the **undiluted** approval.
    let six = lost_per_witness(6);
    assert!(
        six.affinity > 0.0 && six.respect > 0.0,
        "every witness must still judge the insult harshly: affinity {:+.4}, respect {:+.4}",
        six.affinity,
        six.respect
    );
    assert!(
        six.affinity < 0.12 * 0.5,
        "six witnesses must each lose less than half the single-witness cost, lost {:+.4}",
        six.affinity
    );
}
