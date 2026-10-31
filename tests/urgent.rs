//! **A rank that follows a need.** `ActivityDef::urgent` raises an activity's rank while one need is low, so a hungry
//! person's meal is in a tier above everything a fed person would weigh against it (Maslow's prepotency, the Sims'
//! need buckets, Dill's ranks): `docs/loop/runs/2026-10-30-29/`.
mod common;

use bevy::prelude::*;
use bevy_society::{Autonomous, DecisionTrace, Doing, Home, Needs, Place, PlaceTags, SocietyDefs};

/// `eat` answers hunger with a score that falls as the person is fed; `play` scores `1.0` whoever asks, so it out-scores
/// a hungry person's meal and the only thing that can put the meal first is its rank. `eat_extras` is spliced into
/// `eat` (an `urgent` clause, a `rank`), and `eat_target` is where it happens.
fn defs(eat_target: &str, eat_extras: &str, play_extras: &str) -> SocietyDefs {
    SocietyDefs::from_ron(&format!(
        r#"(
            needs: [(id: "food", decay_per_day: 0.0)],
            activities: [
                (id: "eat", target: {eat_target}, duration_hours: 0.5, {eat_extras}
                 considerations: [(input: Need("food"), curve: Logistic(-10.0, 0.35))],
                 satisfies: [("food", 1.0)], interaction: None, barks: []),
                (id: "play", target: Itself, duration_hours: 0.5, {play_extras}
                 considerations: [(input: Need("food"), curve: Linear(0.0, 1.0))],
                 satisfies: [], interaction: None, barks: []),
            ],
        )"#
    ))
    .expect("the definitions parse")
}

const URGENT: &str = r#"urgent: Some((need: "food", below: 0.25, rank: 1)),"#;

/// One person per entry of `food`, at that level, and what each chose first. `table` puts a place tagged `table` in the
/// world, which is where `Place("table")` activities happen.
fn choices(defs: &SocietyDefs, food: &[f32], table: bool) -> (Vec<Option<String>>, bevy::app::App) {
    let mut app = common::headless(defs.clone(), common::settings());
    app.insert_resource(DecisionTrace::with_capacity(64, 4));
    let bench = app
        .world_mut()
        .spawn((
            Name::new("table"),
            Place { capacity: 64 },
            if table {
                PlaceTags(vec!["table".to_owned()])
            } else {
                PlaceTags(vec!["bench".to_owned()])
            },
            Transform::default(),
        ))
        .id();
    let people: Vec<Entity> = food
        .iter()
        .enumerate()
        .map(|(i, &level)| {
            let person = common::spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32, 0.0));
            app.world_mut().entity_mut(person).insert((
                Autonomous,
                Home(bench),
                Needs(vec![level]),
            ));
            person
        })
        .collect();
    common::run(&mut app, 4);
    let chose = people
        .iter()
        .map(|&person| {
            app.world()
                .get::<Doing>(person)
                .map(|doing| defs.activity_name(doing.activity).to_owned())
        })
        .collect();
    (chose, app)
}

fn count(chose: &[Option<String>], name: &str) -> usize {
    chose.iter().filter(|c| c.as_deref() == Some(name)).count()
}

/// **A hungry person takes the meal every time, and a fed one never does.** Twenty-four people at `food` 0.1: `eat`
/// scores 0.92 against `play`'s 1.0, so without a rank it is a draw and about half of them play; with `urgent` every one
/// eats. At 0.9 `eat` scores 0.0003 and falls under the tier cut, rank or no rank, so the line changes nothing for the
/// fed.
#[test]
fn a_hungry_person_eats_where_a_fed_one_does_not() {
    let hungry = [0.1_f32; 24];
    let (control, _) = choices(&defs("Itself", "", ""), &hungry, true);
    assert!(
        count(&control, "play") > 0,
        "without a rank the hungry draw between the two: {control:?}"
    );

    let (urgent, _) = choices(&defs("Itself", URGENT, ""), &hungry, true);
    assert_eq!(count(&urgent, "eat"), 24, "all twenty-four eat: {urgent:?}");

    let fed = [0.9_f32; 24];
    let (urgent_fed, _) = choices(&defs("Itself", URGENT, ""), &fed, true);
    assert_eq!(count(&urgent_fed, "play"), 24, "{urgent_fed:?}");
}

/// **`below` is a strict line.** A person exactly at it is not urgent (the draw between meal and play is back); one
/// under it is.
#[test]
fn the_rank_is_taken_strictly_under_the_line() {
    let pack = defs("Itself", URGENT, "");
    let (at, _) = choices(&pack, &[0.25_f32; 24], true);
    assert!(
        count(&at, "play") > 0,
        "at 0.25 the rank is not taken: {at:?}"
    );
    let (under, _) = choices(&pack, &[0.249_f32; 24], true);
    assert_eq!(count(&under, "eat"), 24, "{under:?}");
}

/// **When the urgent option cannot happen, the lower tier decides** (Graham: "unless they fail to find anything that can
/// solve their hunger"). With no place tagged `table`, `eat` has no candidate, so the hungry person plays; with one, the
/// hungry person goes to eat.
#[test]
fn a_meal_with_nowhere_to_eat_it_falls_through() {
    let pack = defs(r#"Place("table")"#, URGENT, "");
    let hungry = [0.1_f32; 12];
    let (no_table, _) = choices(&pack, &hungry, false);
    assert_eq!(count(&no_table, "play"), 12, "{no_table:?}");
    let (table, _) = choices(&pack, &hungry, true);
    assert_eq!(count(&table, "eat"), 12, "{table:?}");
}

/// **A rank equal to another's shares its tier, and a higher one wins it.** `play` carries a static rank of 1: an urgent
/// rank of 1 puts the meal beside it (drawn by score, so both turn up), an urgent rank of 2 puts the meal over it.
#[test]
fn urgency_competes_with_a_static_rank_by_tier() {
    let hungry = [0.1_f32; 24];
    let beside = defs("Itself", URGENT, "rank: 1,");
    let (shared, _) = choices(&beside, &hungry, true);
    assert!(
        count(&shared, "eat") > 0 && count(&shared, "play") > 0,
        "one tier, two options: {shared:?}"
    );

    let above = defs(
        "Itself",
        r#"urgent: Some((need: "food", below: 0.25, rank: 2)),"#,
        "rank: 1,",
    );
    let (won, _) = choices(&above, &hungry, true);
    assert_eq!(count(&won, "eat"), 24, "{won:?}");
}

/// **The trace says it was the tier.** The chosen `eat` is recorded at rank 1 under a `play` that scored higher at rank
/// 0, and the one-line account names both ranks.
#[test]
fn the_decision_trace_records_the_tier() {
    let pack = defs("Itself", URGENT, "");
    let (chose, app) = choices(&pack, &[0.1_f32], true);
    assert_eq!(chose[0].as_deref(), Some("eat"));
    let trace = app.world().resource::<DecisionTrace>();
    let decision = trace.recent().next().expect("one decision was recorded");
    let eat = decision.chosen().expect("a choice");
    assert_eq!((eat.activity.as_str(), eat.rank), ("eat", 1));
    let play = decision
        .options
        .iter()
        .find(|option| option.activity == "play")
        .expect("play scored above zero");
    assert_eq!(play.rank, 0);
    assert!(play.score > eat.score, "{decision:?}");
    let text = decision.describe(&[]);
    assert!(text.contains("rank 1") && text.contains("rank 0"), "{text}");
}

/// **Every way to write the line wrong is refused at load, naming the activity.**
#[test]
fn a_malformed_urgent_clause_is_an_error() {
    let wrong = |clause: &str| {
        let Err(err) = SocietyDefs::from_ron(&format!(
            r#"(
                needs: [(id: "food", decay_per_day: 0.0)],
                activities: [
                    (id: "eat", target: Itself, duration_hours: 0.5, {clause}
                     satisfies: [], interaction: None, barks: []),
                ],
            )"#
        )) else {
            panic!("the clause was accepted: {clause}");
        };
        err.to_string()
    };
    let unknown = wrong(r#"urgent: Some((need: "thirst", below: 0.25, rank: 1)),"#);
    assert!(
        unknown.contains("thirst") && unknown.contains("eat"),
        "{unknown}"
    );
    let high = wrong(r#"urgent: Some((need: "food", below: 1.5, rank: 1)),"#);
    assert!(high.contains("below") && high.contains("eat"), "{high}");
    let zero = wrong(r#"urgent: Some((need: "food", below: 0.0, rank: 1)),"#);
    assert!(zero.contains("below"), "{zero}");
    let flat = wrong(r#"rank: 1, urgent: Some((need: "food", below: 0.25, rank: 1)),"#);
    assert!(flat.contains("rank") && flat.contains("eat"), "{flat}");
}
