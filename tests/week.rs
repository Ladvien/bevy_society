//! **A week.** `Input::Week` is how far through the week it is, `Curve::Days(from, to)` is a window over it,
//! `SocietySettings::start_weekday` says which day game-day 0 is, and `ctx.weekday()` reads the same calendar in a script. The
//! research is `docs/research/weekly-rhythm.md`; the evidence that a village with a rest day spends it differently is the
//! harness's (`docs/loop/runs/2026-10-31-04/`). These tests pin the mechanisms:
//!
//! 1. an activity gated on `Days(0, 5)` is started on days 0-4 of the week and never on 5-6, on every day of two weeks, and a
//!    different `start_weekday` moves which game-days those are;
//! 2. a window may wrap past the end of the week (`Days(6, 1)`) and may start mid-day (`Days(4.75, 6)`);
//! 3. a body clock does not shift the week: a person twelve hours behind starts nothing on a rest day either;
//! 4. `ctx.weekday()` in a script gives the same schedule as the curve;
//! 5. a `Days` curve on the wrong input, or a window outside the week, is refused at load, naming the activity.
mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{
    ActivityStarted, Autonomous, Chronotype, DefsFile, Home, Place, SocietyClock, SocietyDefs,
    SocietySettings,
};
use common::{headless, run, settings, spawn_person};

/// 24 s a game-day is one game-hour a second, 64 updates.
const UPDATES_PER_DAY: usize = 24 * 64;

/// One activity, `toil`, one game-hour long at the person's own bed, scoring whatever `considerations` say and nothing else.
fn toil(considerations: &str) -> Result<SocietyDefs, bevy_society::DefsError> {
    let mut file = DefsFile::starter().expect("the starter pack parses");
    file.activities = DefsFile::from_ron(&format!(
        r#"(activities: [
            (id: "toil", target: Home, duration_hours: 1.0,
             considerations: [{considerations}],
             satisfies: [], interaction: None, barks: []),
        ])"#
    ))
    .expect("the activity parses")
    .activities;
    SocietyDefs::compile(file)
}

/// `(game-day, hour of day, person index)` of every start of `toil` over `days` game-days from midnight of day 0, for people with
/// the given chronotypes sharing one bed.
fn starts(
    defs: SocietyDefs,
    start_weekday: u8,
    days: usize,
    chronotypes: &[f32],
) -> Vec<(u32, f32, usize)> {
    let mut app = headless(
        defs,
        SocietySettings {
            seconds_per_day: 24.0,
            start_hour: 0.0,
            start_weekday,
            ..settings()
        },
    );
    let bed = app
        .world_mut()
        .spawn((
            Name::new("bed"),
            Place { capacity: 8 },
            Transform::default(),
        ))
        .id();
    let people: Vec<Entity> = chronotypes
        .iter()
        .enumerate()
        .map(|(i, offset)| {
            let person = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.3, 0.0));
            app.world_mut()
                .entity_mut(person)
                .insert((Autonomous, Home(bed), Chronotype(*offset)));
            person
        })
        .collect();
    let mut seen = Vec::new();
    for _ in 0..days * UPDATES_PER_DAY {
        run(&mut app, 1);
        let clock = *app.world().resource::<SocietyClock>();
        let started: Vec<ActivityStarted> = app
            .world_mut()
            .resource_mut::<Messages<ActivityStarted>>()
            .drain()
            .collect();
        for message in started.into_iter().filter(|m| m.activity == "toil") {
            if let Some(i) = people.iter().position(|p| *p == message.person) {
                seen.push((clock.day(), clock.hour(), i));
            }
        }
    }
    seen
}

/// The weekday of a game-day, worked out here and not by the crate: the test would be a tautology if it asked `SocietyClock`.
fn weekday(start_weekday: u8, day: u32) -> u32 {
    (u32::from(start_weekday) + day) % 7
}

fn assert_only_on(starts: &[(u32, f32, usize)], start_weekday: u8, allowed: &[u32], days: usize) {
    assert!(!starts.is_empty(), "nothing started at all");
    for &(day, hour, who) in starts {
        let wd = weekday(start_weekday, day);
        assert!(
            allowed.contains(&wd),
            "person {who} started toil on game-day {day} at {hour:.2} h, weekday {wd}, allowed {allowed:?}"
        );
    }
    for wd in allowed {
        let n = starts
            .iter()
            .filter(|(day, _, _)| weekday(start_weekday, *day) == *wd)
            .count();
        assert!(n >= 1, "nothing started on weekday {wd} in {days} days");
    }
}

// ---- the curve through a real decision ------------------------------------------------------------------------------

#[test]
fn an_activity_open_on_days_zero_to_four_is_never_started_on_a_rest_day() {
    let defs = toil("(input: Week, curve: Days(0.0, 5.0))").expect("compiles");
    let seen = starts(defs, 0, 14, &[0.0]);
    assert_only_on(&seen, 0, &[0, 1, 2, 3, 4], 14);
}

#[test]
fn start_weekday_moves_which_game_days_are_rest_days() {
    for start in [3u8, 6, 9] {
        let defs = toil("(input: Week, curve: Days(0.0, 5.0))").expect("compiles");
        let seen = starts(defs, start, 14, &[0.0]);
        // 9 wraps: 9 % 7 = 2.
        assert_only_on(&seen, start, &[0, 1, 2, 3, 4], 14);
    }
}

#[test]
fn a_window_wraps_past_the_end_of_the_week() {
    let defs = toil("(input: Week, curve: Days(6.0, 1.0))").expect("compiles");
    let seen = starts(defs, 0, 14, &[0.0]);
    assert_only_on(&seen, 0, &[6, 0], 14);
}

#[test]
fn a_window_may_open_in_the_middle_of_a_day() {
    // Days(4.75, 6.0) opens at 18:00 on day 4 and closes at the end of day 5.
    let defs = toil("(input: Week, curve: Days(4.75, 6.0))").expect("compiles");
    let seen = starts(defs, 0, 14, &[0.0]);
    assert!(!seen.is_empty(), "nothing started");
    for &(day, hour, _) in &seen {
        let position = weekday(0, day) as f32 + hour / 24.0;
        assert!(
            (4.75..6.0).contains(&position),
            "started at week position {position:.3} (day {day}, {hour:.2} h)"
        );
    }
    let on_day_four = seen.iter().filter(|(d, ..)| weekday(0, *d) == 4).count();
    let on_day_five = seen.iter().filter(|(d, ..)| weekday(0, *d) == 5).count();
    assert!(
        on_day_four >= 1,
        "the evening of day 4 is inside the window"
    );
    assert!(on_day_five >= 1, "day 5 is inside the window");
}

#[test]
fn a_body_clock_does_not_move_the_week() {
    // A person twelve hours behind reads `Hour` as the wall clock less twelve; `Week` is the village's, so on a rest day they
    // start nothing either, not even in the twelve hours their own midnight would have let through.
    let defs = toil("(input: Week, curve: Days(0.0, 5.0))").expect("compiles");
    let seen = starts(defs, 0, 14, &[0.0, 12.0, -12.0]);
    assert_only_on(&seen, 0, &[0, 1, 2, 3, 4], 14);
    for who in 0..3 {
        assert!(
            seen.iter().any(|(_, _, i)| *i == who),
            "person {who} never worked"
        );
    }
}

#[test]
fn a_script_reads_the_same_calendar() {
    let defs = toil(
        r#"(input: Script("if ctx.weekday() >= 5 { 0.0 } else { 1.0 }"), curve: Linear(1.0, 0.0))"#,
    )
    .expect("compiles");
    let seen = starts(defs, 2, 14, &[0.0]);
    assert_only_on(&seen, 2, &[0, 1, 2, 3, 4], 14);
}

// ---- refused at load -------------------------------------------------------------------------------------------------

#[test]
fn a_days_curve_needs_the_week_input() {
    let Err(err) = toil("(input: Hour, curve: Days(0.0, 5.0))") else {
        panic!("a Days curve on the Hour input compiled");
    };
    let text = err.to_string();
    assert!(text.contains("toil") && text.contains("Week"), "{text}");
}

#[test]
fn a_window_outside_the_week_is_refused() {
    for window in ["Days(0.0, 8.0)", "Days(-1.0, 3.0)", "Days(1.0, 7.5)"] {
        let Err(err) = toil(&format!("(input: Week, curve: {window})")) else {
            panic!("{window} compiled");
        };
        let text = err.to_string();
        assert!(
            text.contains("toil") && text.contains("outside the week"),
            "{window}: {text}"
        );
    }
    // The ends of the week are fine, so is a window that takes the whole of it.
    toil("(input: Week, curve: Days(0.0, 7.0))").expect("0..=7 is the week");
}

#[test]
fn the_week_input_reads_through_any_curve() {
    // A ramp across the week is a legitimate use; only `Days` is tied to this input.
    toil("(input: Week, curve: Linear(1.0, 0.0))").expect("compiles");
}
