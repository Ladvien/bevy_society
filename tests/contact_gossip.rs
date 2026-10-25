//! **People speak when they have news** (plan D7b).
//!
//! The `how-can-two-groups-ever-meet` decision measured the obvious version of this and rejected it: queueing a
//! `chat` for every pair in contact nearly tripled all social contact (13 742 requests over 17 game-days against
//! a 7 500 baseline) and turned a village of 24 into 267 friend pairs — 97% of every possible pair. The fault
//! was the gate, not the rate: a `chat` is offered whether or not either person has anything to say.
//!
//! This file pins the gate that replaced it. [`Knowledge::best_telling`] only yields a rumor that is `alive`,
//! whose subject is not the listener, that the teller still spreads, and that the listener does not already
//! know — so a pair with no news between them says nothing at all.
//!
//! What the test asserts, and why each is a plausible consumer-visible bug:
//!
//! 1. A spreader who shares a place passes the rumor to someone in contact. If this fails the feature does
//!    nothing.
//! 2. A person with no rumor never speaks on contact. This is the clique guard: it is the whole reason the
//!    earlier attempt was reverted.
//! 3. Once the listener knows, the pair stops speaking. Otherwise contact gossip becomes a per-tick metronome
//!    and we have re-derived the failed rate by another route.
mod common;

use bevy::prelude::*;
use bevy_society::{
    Home, Knowledge, Place, PlaceTags, PlantRumor, RumorBook, SocietyDefs, SocietySettings,
};
use common::{headless, spawn_person};

/// Two people standing inside `contact_radius` of each other, in one place, so `perceive` puts each in the
/// other's contact list and nothing has to walk anywhere.
fn pair(seconds_per_day: f32) -> (App, Entity, Entity) {
    let mut settings: SocietySettings = common::settings();
    settings.seed = 5;
    settings.seconds_per_day = seconds_per_day;
    settings.tuning.contact_gossip_cooldown_days = 0.0;
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), settings);
    let place = app
        .world_mut()
        .spawn((
            Name::new("tavern"),
            Place { capacity: 8 },
            PlaceTags(vec!["gathering".to_owned()]),
            Transform::default(),
        ))
        .id();
    let a = spawn_person(&mut app, "Teller", Vec2::ZERO);
    let b = spawn_person(&mut app, "Listener", Vec2::new(1.0, 0.0));
    // **Deliberately not `Autonomous`.** `decide_activities` needs that marker, so leaving it off freezes the
    // decision model entirely and the *only* thing that can pass a rumor here is `contact_gossip`. Without
    // this the starter pack's own `gossip` activity delivers the rumor and the test proves nothing.
    for person in [a, b] {
        app.world_mut().entity_mut(person).insert(Home(place));
    }
    (app, a, b)
}

fn knows(app: &App, person: Entity, id: bevy_society::RumorId) -> bool {
    app.world()
        .get::<Knowledge>(person)
        .is_some_and(|known| known.get(id).is_some())
}

#[test]
fn a_spreader_who_shares_a_place_tells_someone_who_does_not_know() {
    let (mut app, teller, listener) = pair(64.0);
    app.update();

    app.world_mut().write_message(PlantRumor {
        origin: teller,
        subject: teller,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    app.update();
    let id = app
        .world()
        .resource::<RumorBook>()
        .rumors
        .iter()
        .rposition(|rumor| rumor.origin == teller)
        .map(|index| bevy_society::RumorId(index as u32))
        .expect("the rumor was planted");
    assert!(
        knows(&app, teller, id),
        "the origin must know its own rumor"
    );
    assert!(!knows(&app, listener, id), "the listener starts ignorant");

    for _ in 0..400 {
        app.update();
        if knows(&app, listener, id) {
            return;
        }
    }
    panic!("contact gossip never passed the rumor to a person standing 1 m away");
}

#[test]
fn a_person_with_no_news_says_nothing_on_contact() {
    // Two people with no rumor between them: nothing to tell, so no exchange is queued at all. This is the
    // guard the earlier contact-chat attempt lacked, and what keeps a village from becoming a clique.
    let (mut app, a, b) = pair(64.0);
    for _ in 0..60 {
        app.update();
    }
    let rumored = app.world().resource::<RumorBook>().rumors.len();
    assert_eq!(rumored, 0, "no rumor exists, so nothing can be told");
    for person in [a, b] {
        let known = app
            .world()
            .get::<Knowledge>(person)
            .map_or(0, |knowledge| knowledge.0.len());
        assert_eq!(known, 0, "silence must leave both people ignorant");
    }
}

#[test]
fn an_informed_neighbour_never_costs_the_teller_its_spread() {
    // **The measured invariant, and the reason a listener-side check is not optional.** Every telling to
    // someone who already knows *stifles* the teller (Daley–Kendall). Pacing the contact pass therefore
    // decides how fast the spreader pool is burned: measured on the village at the same seed, a half-day
    // cooldown dropped rumor reach to **0.375** against a **0.583** baseline, while two days lifted it to
    // **0.875**. The check below is what keeps a contact-tell from ever spending a spreader on a neighbour
    // who is already informed, and this test is what stops it being deleted as "redundant".
    let (mut app, teller, listener) = pair(64.0);
    app.update();
    app.world_mut().write_message(PlantRumor {
        origin: teller,
        subject: teller,
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    app.update();
    let id = app
        .world()
        .resource::<RumorBook>()
        .rumors
        .iter()
        .rposition(|rumor| rumor.origin == teller)
        .map(|index| bevy_society::RumorId(index as u32))
        .expect("the rumor was planted");

    // Let the contact pass deliver it, then keep them standing together for a long time. The teller must
    // remain a spreader throughout: it is only ever offered listeners who do not know.
    for _ in 0..400 {
        app.update();
        if knows(&app, listener, id) {
            break;
        }
    }
    assert!(knows(&app, listener, id), "the rumor must arrive");
    let still_spreads = |app: &App| {
        app.world()
            .get::<Knowledge>(teller)
            .and_then(|k| k.get(id))
            .is_some_and(|known| known.spreader)
    };
    assert!(
        still_spreads(&app),
        "the teller must not be stifled by a contact-tell: it was never offered a listener who already knew"
    );
    for _ in 0..600 {
        app.update();
    }
    assert!(
        still_spreads(&app),
        "standing next to an informed neighbour for 600 more steps must not burn the spreader either"
    );
}

#[test]
fn the_rumor_crosses_the_gap_only_when_the_two_groups_actually_meet() {
    // The pending `rumor-crosses-a-cluster` scenario asks for a crossing across 40 m. Contact gossip does not
    // deliver one, and this says so: the gate opens on news, not on distance, so two groups that never stand
    // together never exchange. Kept as a test so the scenario cannot be marked `measured` on a number nobody
    // checked.
    let mut settings: SocietySettings = common::settings();
    settings.seed = 11;
    settings.seconds_per_day = 64.0;
    settings.start_hour = 15.0;
    settings.contact_radius = 4.0;
    settings.tuning.travel_timeout_hours = Some(12.0);
    let mut app = headless(SocietyDefs::starter().expect("the pack parses"), settings);
    let mut villagers = Vec::new();
    for group in 0..2 {
        let place = app
            .world_mut()
            .spawn((
                Name::new(format!("camp{group}")),
                Place { capacity: 12 },
                PlaceTags(vec!["work".to_owned()]),
                Transform::from_xyz(group as f32 * 40.0, 0.0, 0.0),
            ))
            .id();
        for i in 0..6 {
            let person = spawn_person(
                &mut app,
                &format!("G{group}P{i}"),
                Vec2::new(group as f32 * 40.0, i as f32 * 0.5),
            );
            app.world_mut().entity_mut(person).insert(Home(place));
            villagers.push(person);
        }
    }
    for _ in 0..8 {
        app.update();
    }
    app.world_mut().write_message(PlantRumor {
        origin: villagers[0],
        subject: villagers[6],
        patient: None,
        deed: "steal".to_owned(),
        truth: true,
    });
    app.update();

    // A fixed number of updates, not a wall-clock duration.
    let mut peak = 0usize;
    for _ in 0..1_200 {
        app.update();
        peak = peak.max(
            villagers[6..]
                .iter()
                .filter(|entity| {
                    app.world()
                        .get::<Knowledge>(**entity)
                        .is_some_and(|known| known.0.iter().any(|k| k.rumor.0 == 0))
                })
                .count(),
        );
    }
    assert!(
        peak == 0,
        "contact gossip crossed a 40 m gap ({peak} of 6): it must not, because the two groups never share a \
         place. If this now passes, the scenario's observable can be re-stated as met"
    );
}
