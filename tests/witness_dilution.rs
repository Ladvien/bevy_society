//! **A deed seen by many should cost each witness less than a deed seen by one.**
//!
//! `witness_reaction` applies `trust_gain * harm` at full magnitude to **every** witness within perception,
//! so a deed's aggregate cost grows linearly with the crowd. A trust *gain* from an interaction reaches
//! only the two parties, so the two move on different scales: one `chat` is `+0.10` in total while one insult
//! seen by three people is `-0.42`.
//!
//! The literature measures exactly this transmission. Sommerfeld, Krambeck, Semmann & Milinski 2007, *Gossip
//! as an alternative for direct observation in games of indirect reciprocity*, PNAS, DOI
//! `10.1073/pnas.0704598104`:
//!
//! > "transmission by gossip had a **dampening effect** on the cooperation levels compared with acting on
//! > direct observation. The more extreme a gossip was, the less impact it had on the reader."
//!
//! and, of the number of sources, citing Hess & Hagen: *"Hess and Hagen found an increase in **believability**
//! with an increasing number of sources"* — believability, not magnitude. Information from many observers is
//! believed more and moves each recipient less, which is what a per-witness dilution models.
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

/// Puts `witnesses` bystanders within perception of two people, has them insult each other, and reports the
/// mean trust a witness loses toward the actor after one deed.
fn trust_lost_per_witness(witnesses: usize) -> f32 {
    let mut day = settings();
    day.seconds_per_day = 64.0;
    day.start_hour = 15.0;
    // **Force acceptance.** A refused insult fires no deed, so the measurement would be of the refusal
    // path instead. `accept.base = 1.0` and every other weight zero makes the draw certain.
    day.tuning.rejection_distress = 0.0;
    let mut app = headless(defs_with_certain_acceptance(), day);

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
            .insert((Autonomous, Person, Home(place)));
        crowd.push(person);
    }
    let actor = crowd[0];
    app.update();
    app.update();

    let before: Vec<f32> = crowd
        .iter()
        .map(|w| {
            app.world()
                .get::<Ties>(*w)
                .and_then(|ties| ties.get(actor))
                .map(|tie| tie.trust)
                .unwrap_or(0.5)
        })
        .collect();

    // One `steal` by the actor against the second person, with everyone else watching.
    app.world_mut().write_message(bevy_society::Interact {
        actor,
        target: crowd[1],
        interaction: "insult".to_owned(),
    });
    for _ in 0..4 {
        app.update();
    }

    let mut total = 0.0_f32;
    let mut count = 0u32;
    for (i, watcher) in crowd.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if let Some(tie) = app.world().get::<Ties>(*watcher).and_then(|t| t.get(actor)) {
            total += before[i] - tie.trust;
            count += 1;
        }
    }
    total / count.max(1) as f32
}

#[test]
fn more_witnesses_means_less_trust_lost_by_each() {
    let one = trust_lost_per_witness(1);
    let five = trust_lost_per_witness(5);
    println!("trust lost per witness: with 1 bystander {one:+.4}, with 5 {five:+.4}");

    assert!(
        one > 0.0,
        "a witnessed deed must cost the witness something: {one:+.4}"
    );
    assert!(
        five < one,
        "the same deed seen by five takes {five:+.4} from each and seen by one takes {one:+.4}. \
         Without dilution the aggregate cost grows with the crowd while a good deed's reward cannot, \
         which is why trust only ever falls in a village"
    );
}
