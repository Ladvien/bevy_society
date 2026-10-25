//! **A label is a latitude, not a line — measured, and only where the noise is.**
//!
//! `TieLabelChanged` is the message a host acts on: "you are now friends", a conversation unlocked, a
//! quest begun. Measured on `village-evening` (seed 7, 24 people, 16.4 game-days) before this change it
//! fired **2 867** times — one every **8.2 game-minutes** — and 85 of the 276 ordered pairs changed more
//! than ten times on their own.
//!
//! Half of those were events and half were noise, and the two must not be confused:
//!
//! | transition | count | median margin from its bar | the step that flipped it |
//! |---|---|---|---|
//! | `Stranger -> Acquaintance` | 828 | **0.0027** | 0.0002 — a relaxation step |
//! | `Friend <-> Acquaintance` | 1 027 | 0.019 / 0.043 | 0.013 median — an accepted exchange |
//!
//! So the band is on the low end only. The friend and rival bars keep a single line, because a witnessed
//! deed that moves affinity by 0.01 in one tick *did* change the relationship and the host should hear
//! about it. The low end had no such events: familiarity merely decayed through the bar.
mod common;

use bevy::prelude::*;
use bevy_society::{
    PersonIndex, SocietyClock, SocietySettings, TieLabel, TieLabelChanged, Ties, Tuning,
};

#[test]
fn a_tie_at_the_bar_is_an_acquaintance_and_one_just_under_it_stays_one() {
    let tuning = Tuning::default();
    assert!(
        tuning.label_hysteresis > 0.0,
        "the shipped default must have a band, or the low end churns as it did"
    );
    let at_bar = tuning.stranger_familiarity;
    let just_under = at_bar - tuning.label_hysteresis / 2.0;

    // Promotion is unchanged: at the bar, an Acquaintance.
    assert_eq!(
        TieLabel::classify_from(TieLabel::Stranger, 0.0, at_bar, 0.5, &tuning),
        TieLabel::Acquaintance,
        "a stranger becomes an acquaintance at `stranger_familiarity`, as before"
    );

    // Demotion is held: the same familiarity, already an Acquaintance, stays one.
    assert_eq!(
        TieLabel::classify_from(TieLabel::Acquaintance, 0.0, just_under, 0.5, &tuning),
        TieLabel::Acquaintance,
        "familiarity inside the band must not drop the tie back to a stranger"
    );

    // And past the band it does fall, which is the point of having a band rather than none.
    let below = at_bar - tuning.label_hysteresis - 0.001;
    assert_eq!(
        TieLabel::classify_from(TieLabel::Acquaintance, 0.0, below, 0.5, &tuning),
        TieLabel::Stranger
    );
}

#[test]
fn a_friendship_is_held_only_within_its_band() {
    let tuning = Tuning::default();
    assert!(
        tuning.friend_band > 0.0,
        "the shipped default must hold a friendship through a shallow dip, or every refusal ends one"
    );
    let inside = tuning.friend_affinity - tuning.friend_band / 2.0;
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Friend,
            inside,
            tuning.friend_familiarity + 0.5,
            tuning.friend_trust + 0.2,
            &tuning
        ),
        TieLabel::Friend,
        "a dip inside the band keeps the friendship: measured, that dip is one refused approach"
    );
    let outside = tuning.friend_affinity - tuning.friend_band - 0.01;
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Friend,
            outside,
            tuning.friend_familiarity + 0.5,
            tuning.friend_trust + 0.2,
            &tuning
        ),
        TieLabel::Acquaintance,
        "a dip past the band ends it, so an insult or a refused approach still has a consequence"
    );
    // A friendship that has gone to rival affinity is a Rival, in one tick.
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Friend,
            -tuning.rival_affinity - 0.01,
            tuning.friend_familiarity + 0.5,
            tuning.friend_trust + 0.2,
            &tuning
        ),
        TieLabel::Rival,
        "a friendship that turned is a rivalry at once"
    );
}

#[test]
fn the_rival_bar_keeps_a_single_line() {
    let tuning = Tuning::default();
    let just_over = -tuning.rival_affinity + tuning.label_hysteresis / 2.0;
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Rival,
            just_over,
            tuning.rival_familiarity + 0.5,
            0.5,
            &tuning
        ),
        TieLabel::Acquaintance,
        "and the same at the rival bar"
    );
}

#[test]
fn a_zero_band_is_the_old_single_bar() {
    let strict = Tuning {
        label_hysteresis: 0.0,
        friend_band: 0.0,
        ..Tuning::default()
    };
    for (affinity, familiarity) in [
        (0.0, strict.stranger_familiarity - 0.001),
        (0.0, strict.stranger_familiarity),
        (0.0, strict.stranger_familiarity + 0.5),
        (strict.friend_affinity, strict.friend_familiarity + 0.5),
        (0.0, 0.0),
    ] {
        for current in [
            TieLabel::Stranger,
            TieLabel::Acquaintance,
            TieLabel::Friend,
            TieLabel::Rival,
        ] {
            assert_eq!(
                TieLabel::classify_from(current, affinity, familiarity, 0.9, &strict),
                TieLabel::classify_with_trust(affinity, familiarity, 0.9, &strict),
                "with no band the label must not depend on what it was: {current:?} at \
                 affinity {affinity} familiarity {familiarity}"
            );
        }
    }
}

/// The band must not become a ceiling: a tie inside it still promotes the moment it qualifies.
#[test]
fn a_held_category_still_promotes() {
    let tuning = Tuning::default();
    // A tie that has gone from friendship to well past the rival bar is a **Rival**, in one tick: the
    // band holds a category while the value is inside it, and never prevents promotion to another.
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Friend,
            -tuning.rival_affinity - 0.01,
            0.9,
            0.9,
            &tuning
        ),
        TieLabel::Rival,
        "a friendship that turned is a rivalry at once, not an acquaintance after a band"
    );
    // A Stranger with enough affinity and trust is a Friend in the same tick, not an Acquaintance first.
    assert_eq!(
        TieLabel::classify_from(
            TieLabel::Stranger,
            tuning.friend_affinity + 0.01,
            tuning.friend_familiarity + 0.5,
            tuning.friend_trust + 0.1,
            &tuning
        ),
        TieLabel::Friend
    );
}

/// Every `TieLabelChanged` written in one run, so two runs can be compared.
fn label_sequence(app: &mut App, ticks: usize) -> Vec<(u64, Entity, Entity, TieLabel, TieLabel)> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        app.update();
        let tick = app.world().resource::<SocietyClock>().tick;
        for message in app
            .world_mut()
            .resource_mut::<Messages<TieLabelChanged>>()
            .drain()
        {
            out.push((tick, message.from, message.to, message.old, message.new));
        }
    }
    out
}

/// The village at `village-evening`'s own clock, with `band` as its label hysteresis.
fn village(seed: u64, band: f32) -> App {
    let mut app = common::village::village(common::village::village_defs(), seed, 24);
    let mut settings = SocietySettings {
        seed,
        seconds_per_day: 64.0,
        start_hour: 15.0,
        tick_secs: 1.0 / 64.0,
        ..SocietySettings::default()
    };
    settings.tuning.travel_timeout_hours = Some(4.0);
    settings.tuning.label_hysteresis = band;
    app.insert_resource(settings);
    app
}

/// `(friend pairs, rival pairs)` at the moment the run stops.
fn counts(app: &App) -> (u32, u32) {
    let (mut friends, mut rivals) = (0u32, 0u32);
    let index = app.world().resource::<PersonIndex>();
    for &(_, entity) in &index.sorted {
        let Some(ties) = app.world().get::<Ties>(entity) else {
            continue;
        };
        for tie in &ties.0 {
            match tie.label {
                TieLabel::Friend => friends += 1,
                TieLabel::Rival => rivals += 1,
                _ => {}
            }
        }
    }
    (friends / 2, rivals / 2)
}

/// **The band removes the low-end churn and moves nothing else.**
///
/// Measured at the scenario's own clock over 6 400 ticks — long enough for the low end to settle and for
/// friendships to begin, short enough to be a test rather than a run. The claim is threefold and each
/// part is what a host would notice: fewer messages, no `Stranger` message after a pair has met, and
/// identical friend and rival counts at the end.
#[test]
fn the_band_stops_the_low_end_flapping_and_moves_no_relationship() {
    const TICKS: usize = 6_400;
    let mut with = village(7, Tuning::default().label_hysteresis);
    let mut without = village(7, 0.0);
    let changed_with = label_sequence(&mut with, TICKS);
    let changed_without = label_sequence(&mut without, TICKS);

    let demotions_without = changed_without
        .iter()
        .filter(|(_, _, _, old, new)| *old == TieLabel::Acquaintance && *new == TieLabel::Stranger)
        .count();
    let demotions_with = changed_with
        .iter()
        .filter(|(_, _, _, old, new)| *old == TieLabel::Acquaintance && *new == TieLabel::Stranger)
        .count();
    assert!(
        demotions_without > 0,
        "the fixture must show the churn it exists to remove: {demotions_without} demotions without a band"
    );
    // One is not zero, and that is the measurement: a pair that genuinely went unseen for long enough
    // to decay past the band *did* become strangers again, at tick 6 016 of 6 400. A band must not make
    // a label a lie in the other direction.
    assert!(
        demotions_with < demotions_without / 20,
        "a band must make a demotion the exception rather than the rule: {demotions_with} with,          {demotions_without} without"
    );
    assert!(
        demotions_with <= 1,
        "measured, the low-end band leaves at most one genuine demotion in this window: {demotions_with}"
    );
    let (n_with, n_without) = (changed_with.len(), changed_without.len());
    assert!(
        n_with < n_without,
        "a band must reduce the messages ({n_with} with, {n_without} without)"
    );
    assert_eq!(
        counts(&with),
        counts(&without),
        "the band must not create or destroy a friendship or rivalry over this window"
    );
}

/// Same seed, same inputs, same labels — the band adds no draw and no iteration order.
#[test]
fn the_band_is_deterministic() {
    let sequence = |band: f32| {
        let mut app = village(42, band);
        label_sequence(&mut app, 3_200)
    };
    assert_eq!(
        sequence(Tuning::default().label_hysteresis),
        sequence(Tuning::default().label_hysteresis),
        "two runs at the same seed must produce the same label sequence"
    );
}

/// **A friendship is held through a shallow dip, and the held duration shows it.**
///
/// Measured on `village-evening` at the scenario's own clock, seed 7, 16.4 game-days: `friend_affinity`
/// 0.30 against a ceiling of 0.35 puts a friendship on a **0.05-wide shelf**, and
/// `rejection_affinity_penalty` — paid by anyone whose approach is refused, at a 38% refusal rate — is
/// **0.05**, so one refusal is exactly one shove off the shelf. With `friend_band` at 0.05 the median held
/// friendship goes **0.36 -> 1.01 game-days** and the endings halve, **while rivals stay at 42**.
///
/// Holding the friend bar *unconditionally* was measured to take rivals to 48 and was rejected last
/// iteration; this is the narrower claim that survived.
#[test]
fn a_friendship_is_held_through_a_shallow_dip_and_lasts_longer() {
    /// `(friendships that ended, median held in game-days, friend-label messages, rival pairs)`.
    fn run(band: f32) -> (usize, f32, usize, u32) {
        let mut app = village(7, Tuning::default().label_hysteresis);
        app.world_mut()
            .resource_mut::<SocietySettings>()
            .tuning
            .friend_band = band;
        let mut since: Vec<(Entity, Entity, f32)> = Vec::new();
        let mut held: Vec<f32> = Vec::new();
        let mut friend_messages = 0usize;
        for _ in 0..4_200 {
            app.update();
            let now = app.world().resource::<SocietyClock>().now_days as f32;
            for message in app
                .world_mut()
                .resource_mut::<Messages<TieLabelChanged>>()
                .drain()
            {
                if message.new == TieLabel::Friend {
                    since.push((message.from, message.to, now));
                }
                if message.old == TieLabel::Friend {
                    friend_messages += 1;
                    if let Some(i) = since
                        .iter()
                        .position(|(f, t, _)| *f == message.from && *t == message.to)
                    {
                        let (_, _, start) = since.remove(i);
                        held.push(now - start);
                    }
                }
            }
        }
        held.sort_by(f32::total_cmp);
        let median = if held.is_empty() {
            0.0
        } else {
            held[held.len() / 2]
        };
        (held.len(), median, friend_messages, counts(&app).1)
    }

    let (ended_without, median_without, _, rivals_without) = run(0.0);
    let (ended_with, median_with, messages_with, rivals_with) = run(Tuning::default().friend_band);

    assert!(
        ended_with < ended_without,
        "a band must reduce the endings ({ended_with} with, {ended_without} without)"
    );
    assert!(
        median_with > median_without,
        "and the ones that remain must last longer ({median_with:.2} vs {median_without:.2} game-days)"
    );
    assert_eq!(
        rivals_with, rivals_without,
        "the band must not create rivalries: {rivals_with} with, {rivals_without} without"
    );
    assert!(
        messages_with > 0,
        "friendships must still end, or the label has become permanent"
    );
}
