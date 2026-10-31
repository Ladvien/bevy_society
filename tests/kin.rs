//! **Kin: a relative is a fact on the tie, and what a relative gets** (`docs/research/kinship.md`).
//!
//! Hamilton 1964 (DOI `10.1016/0022-5193(64)90038-4`) gives the coefficient of relatedness `r`; Curry,
//! Roberts & Dunbar 2013 (DOI `10.1111/j.2044-8295.2012.02119.x`) found that people are more altruistic
//! toward kin than their emotional closeness alone would predict, and Burton-Chellew & Dunbar 2015 (DOI
//! `10.1093/beheco/aru165`) that the support a person expects ranks by degree of relatedness. So the
//! tests ask what a host sees: a gift goes to the relative first, a nearer relative before a farther one,
//! a relative is never forgotten and is mourned without a friendship behind them, and a society with no
//! relatives is the society it was.
mod common;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy_society::{
    Appraise, Autonomous, Deceased, EmotionKind, GoodId, Home, InteractionResolved, Kin, PersonKey,
    Place, PlaceKey, PlaceTags, PlacementFile, ScriptEvent, ScriptValue, SocietyDefs, Stock, Tie,
    TieLabel, Ties, Tuning,
};
use common::{headless, run, settings, spawn_person};

/// A gift of bread, scored by the surplus alone, so a stranger and a relative holding the same store tie
/// exactly unless `kin_premium` says otherwise.
const GIFT: &str = r#"(
    goods: [(id: "bread")],
    needs: [(id: "fun", decay_per_day: 0.0)],
    activities: [(id: "give", target: Person, duration_hours: 0.2,
        considerations: [(input: Surplus("bread"), curve: Linear(1.0, 0.0))],
        satisfies: [], interaction: Some("gift"), barks: [])],
    interactions: [(id: "gift", on_accept: (actor_gives: [("bread", 1.0)]))],
)"#;

/// A person who calls on whoever the pack scores by `Kinship` and does nothing else.
const VISIT: &str = r#"(
    needs: [(id: "fun", decay_per_day: 0.0)],
    activities: [(id: "visit", target: Person, duration_hours: 0.2,
        considerations: [(input: Kinship, curve: CURVE)],
        satisfies: [], interaction: Some("call"), barks: [])],
    interactions: [(id: "call", on_accept: (actor_to_target: (trust: 0.01)))],
)"#;

/// A bench and `holds.len()` people at it, 0.3 apart, holding the given bread. The first is the giver.
fn gift_village(
    seed: u64,
    premium: f32,
    relations: &[(usize, Option<Kin>)],
    holds: &[f32],
) -> (App, Vec<Entity>) {
    let mut settings = settings();
    settings.seed = seed;
    settings.tuning.kin_premium = premium;
    let mut app = headless(SocietyDefs::from_ron(GIFT).expect("parses"), settings);
    let bench = app
        .world_mut()
        .spawn((
            Name::new("bench"),
            Place { capacity: 8 },
            PlaceKey(0),
            PlaceTags(vec!["bench".to_owned()]),
            Transform::default(),
        ))
        .id();
    let mut people = Vec::new();
    for (i, held) in holds.iter().enumerate() {
        let entity = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.3, 0.0));
        app.world_mut().entity_mut(entity).insert((
            Autonomous,
            Home(bench),
            PersonKey(i as u64),
            Stock(vec![*held]),
        ));
        people.push(entity);
    }
    // No warm-up update: the first gift can resolve inside one, and `first_receiver` reads the stores before it.
    for (other, kin) in relations {
        let mut ties = app.world_mut().get_mut::<Ties>(people[0]).expect("ties");
        ties.entry(people[*other]).kin = *kin;
    }
    (app, people)
}

/// Who first gains a loaf from the giver, as an index into the village, or `None` if nobody did.
fn first_receiver(app: &mut App, people: &[Entity]) -> Option<usize> {
    let held = |app: &App, person: Entity| {
        app.world()
            .get::<Stock>(person)
            .map_or(0.0, |stock| stock.get(GoodId(0)))
    };
    let before: Vec<f32> = people.iter().map(|p| held(app, *p)).collect();
    for _ in 0..600 {
        run(app, 5);
        for (i, person) in people.iter().enumerate().skip(1) {
            if held(app, *person) > before[i] {
                return Some(i);
            }
        }
    }
    None
}

/// Over `seeds`, how often each of `receivers` got the first loaf.
fn tally(premium: f32, relations: &[(usize, Option<Kin>)], seeds: u64) -> [usize; 4] {
    let mut counts = [0; 4];
    for seed in 0..seeds {
        let (mut app, people) = gift_village(seed, premium, relations, &[4.0, 0.0, 0.0, 0.0]);
        if let Some(who) = first_receiver(&mut app, &people) {
            counts[who] += 1;
        }
    }
    counts
}

/// **A gift goes to the relative first**, and only because of `kin_premium`: with the premium off the same
/// two receivers tie on score and the draw splits them, so the preference is not an accident of ordering.
#[test]
fn a_gift_goes_to_the_relative_first() {
    let relations = [(1, Some(Kin::Sibling))];
    let with = tally(1.0, &relations, 16);
    assert_eq!(
        with[1], 16,
        "with the premium a sibling must get the first loaf every time: {with:?}"
    );
    let without = tally(0.0, &relations, 16);
    assert!(
        without[1] > 0 && without[2] > 0,
        "with no premium the two tie and the draw must pick each at least once: {without:?}"
    );
}

/// **A nearer relative before a farther one** (Burton-Chellew & Dunbar 2015: support ranks by `r`): the
/// premium is `1 + premium · 2r`, so a sibling (`r = 0.5`) outscores a cousin (`0.125`), and a cousin
/// outscores a stranger.
#[test]
fn the_premium_grows_with_relatedness() {
    let sibling_over_cousin = tally(1.0, &[(1, Some(Kin::Cousin)), (2, Some(Kin::Sibling))], 12);
    assert_eq!(
        sibling_over_cousin[2], 12,
        "the sibling must beat the cousin every time: {sibling_over_cousin:?}"
    );
    let cousin_over_stranger = tally(1.0, &[(1, Some(Kin::Cousin))], 12);
    assert_eq!(
        cousin_over_stranger[1], 12,
        "a cousin must still beat a stranger: {cousin_over_stranger:?}"
    );
}

/// **The premium is not a way to give to someone who has plenty.** A relative holding as much as the giver
/// leaves no surplus, so a scaled zero stays zero and the loaf goes to the stranger who has none.
#[test]
fn a_relative_with_a_full_shelf_is_not_given_to() {
    let (mut app, people) = gift_village(3, 4.0, &[(1, Some(Kin::Child))], &[3.0, 3.0, 0.0]);
    assert_eq!(
        first_receiver(&mut app, &people),
        Some(2),
        "the stranger with no bread must be the one given to"
    );
}

/// **A gift that only swaps two shelves is not favoured.** The giver holds 1.8, the sibling 0.8 and a stranger
/// 0.8: a loaf to either leaves the receiver holding what the giver did, and the loaf would be passed back. The
/// premium applies only where the giver holds more than the receiver by more than `Tuning::trade_reserve`, so
/// here the two tie as they do with no premium and the draw splits them. Without the test, favouring relatives
/// made 8-25 % of accepted shares a loaf returned within three hours (`docs/loop/runs/2026-10-31-05/eval.md`).
#[test]
fn a_gift_that_only_swaps_shelves_is_not_favoured() {
    let relations = [(1, Some(Kin::Sibling))];
    let mut counts = [0; 4];
    for seed in 0..16 {
        let (mut app, people) = gift_village(seed, 4.0, &relations, &[1.8, 0.8, 0.8]);
        if let Some(who) = first_receiver(&mut app, &people) {
            counts[who] += 1;
        }
    }
    assert!(
        counts[1] > 0 && counts[2] > 0,
        "a swap must not be steered to the relative: {counts:?}"
    );
}

/// **`Input::Kinship` reads `2r`** for a pack that wants to tilt an activity without the premium: a `Step`
/// at `0.9` admits only a first-degree relative, and at `0.2` a cousin (`2r = 0.25`) too, and never a
/// stranger, who reads 0.
#[test]
fn the_kinship_input_reads_twice_the_relatedness() {
    let called = |curve: &str| -> Vec<usize> {
        let text = VISIT.replace("CURVE", curve);
        let mut app = headless(SocietyDefs::from_ron(&text).expect("parses"), settings());
        let bench = app
            .world_mut()
            .spawn((
                Name::new("bench"),
                Place { capacity: 8 },
                PlaceTags(vec!["bench".to_owned()]),
                Transform::default(),
            ))
            .id();
        let mut people = Vec::new();
        for i in 0..4 {
            let entity = spawn_person(&mut app, &format!("P{i}"), Vec2::new(i as f32 * 0.3, 0.0));
            let mut commands = app.world_mut().entity_mut(entity);
            commands.insert((Home(bench), PersonKey(i as u64)));
            if i == 0 {
                commands.insert(Autonomous);
            }
            people.push(entity);
        }
        {
            let mut ties = app.world_mut().get_mut::<Ties>(people[0]).expect("ties");
            ties.entry(people[1]).kin = Some(Kin::Sibling);
            ties.entry(people[2]).kin = Some(Kin::Cousin);
            ties.entry(people[3]);
        }
        run(&mut app, 2);
        let mut hit = [0usize; 4];
        for _ in 0..800 {
            app.update();
            let resolved: Vec<InteractionResolved> = app
                .world_mut()
                .resource_mut::<Messages<InteractionResolved>>()
                .drain()
                .collect();
            for r in resolved {
                if let Some(i) = people.iter().position(|p| *p == r.target) {
                    hit[i] += 1;
                }
            }
        }
        hit.iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i)
            .collect()
    };
    assert_eq!(called("Step(0.9)"), vec![1], "only the sibling reads 1.0");
    assert_eq!(
        called("Step(0.2)"),
        vec![1, 2],
        "the cousin reads 0.25 and the stranger 0"
    );
}

/// **A relative is never forgotten.** The Dunbar cap drops the least familiar ties; a relative whose tie has
/// the lowest familiarity of all stays, and a stranger in the same state goes.
#[test]
fn a_relative_survives_the_dunbar_trim() {
    let mut world = World::new();
    let ids: Vec<Entity> = (0..4).map(|_| world.spawn_empty().id()).collect();
    let tuning = Tuning::default();
    let mut ties = Ties::default();
    for (i, target) in ids.iter().enumerate() {
        let tie = ties.entry(*target);
        // The relative (index 0) is the least familiar; the strangers are better known.
        tie.familiarity_peak = 0.2 + 0.2 * i as f32;
        tie.last_contact_day = 0.0;
    }
    ties.entry(ids[0]).kin = Some(Kin::Parent);
    ties.trim(0.0, &tuning, 2);
    assert!(
        ties.get(ids[0]).is_some(),
        "the relative with the weakest familiarity must stay"
    );
    let kept = ids.iter().filter(|e| ties.get(**e).is_some()).count();
    assert_eq!(
        kept, 3,
        "the cap of two counts the strangers only: the relative and the two best-known strangers stay"
    );
    assert!(
        ties.get(ids[1]).is_none(),
        "the least familiar stranger goes"
    );
}

/// **A relative is mourned with no friendship behind them.** The same tie (no label, a familiarity under
/// `bereavement_familiarity`, neutral affinity) is a grief when it is a sister's and nothing when it is a
/// stranger's.
#[test]
fn a_relative_grieves_without_a_friendship_and_a_stranger_does_not() {
    let mut app = headless(
        SocietyDefs::from_ron(
            r#"(needs: [(id: "social", decay_per_day: 0.2)],
                activities: [(id: "wander", target: Wander(1.0), duration_hours: 1.0,
                    considerations: [(input: Need("social"), curve: Linear(-1.0, 1.0))])])"#,
        )
        .expect("parses"),
        settings(),
    );
    app.update();
    app.update();
    let sister = spawn_person(&mut app, "Sister", Vec2::new(2.0, 0.0));
    let stranger = spawn_person(&mut app, "Stranger", Vec2::new(-2.0, 0.0));
    let dead = spawn_person(&mut app, "Ada", Vec2::new(3.0, 0.0));
    app.update();
    app.update();
    for (who, kin) in [(sister, Some(Kin::Sibling)), (stranger, None)] {
        let mut ties = app.world_mut().get_mut::<Ties>(who).expect("ties");
        let tie = ties.entry_with(dead, 0.0);
        tie.familiarity_peak = 0.05;
        tie.label = TieLabel::Acquaintance;
        tie.kin = kin;
    }
    app.world_mut().entity_mut(dead).insert(Deceased);
    let mut appraised: Vec<Appraise> = Vec::new();
    for _ in 0..4 {
        app.update();
        appraised.extend(app.world_mut().resource_mut::<Messages<Appraise>>().drain());
    }
    let grief = |who: Entity| {
        appraised
            .iter()
            .any(|a| a.person == who && a.kind == EmotionKind::Distress && a.toward == Some(dead))
    };
    assert!(grief(sister), "a sibling must grieve: {appraised:?}");
    assert!(
        !grief(stranger),
        "a stranger on the same footing must not: {appraised:?}"
    );
}

const AUTHORED: &str = r#"(
    persons: [
        (name: "Mara", ties: [(with: "Ada", affinity: 0.2, kin: Some(Child))]),
        (name: "Ada"),
        (name: "Bram", ties: [(with: "Ada", affinity: 0.1)]),
    ],
)"#;

/// **Authoring answers itself.** `Mara` lists `Ada` as her child; spawning makes Ada's tie to Mara (which
/// she did not list) with the converse relation, both ties known from the start, and leaves Bram's plain tie
/// alone.
#[test]
fn an_authored_relative_is_answered_on_the_other_side() {
    let mut app = headless(SocietyDefs::from_ron("()").expect("parses"), settings());
    let placed = PlacementFile::from_ron(AUTHORED)
        .expect("parses")
        .spawn(app.world_mut())
        .expect("spawns");
    let (mara, ada, bram) = (
        placed.person("Mara").unwrap(),
        placed.person("Ada").unwrap(),
        placed.person("Bram").unwrap(),
    );
    let world = app.world();
    let to_ada = world.get::<Ties>(mara).unwrap().get(ada).unwrap();
    let to_mara = world.get::<Ties>(ada).unwrap().get(mara).unwrap();
    assert_eq!(to_ada.kin, Some(Kin::Child));
    assert_eq!(to_mara.kin, Some(Kin::Parent));
    assert_eq!(to_mara.affinity, 0.0, "the answered tie is neutral");
    assert!(to_ada.familiarity_peak >= 1.0 && to_mara.familiarity_peak >= 1.0);
    assert_eq!(
        world.get::<Ties>(bram).unwrap().get(ada).unwrap().kin,
        None,
        "a tie nobody called family stays a stranger's"
    );
    assert!(world.get::<Ties>(ada).unwrap().get(bram).is_none());
}

/// **Two directions that do not answer each other are refused**, naming the person, before anything spawns.
#[test]
fn two_ties_that_disagree_about_the_relation_are_refused() {
    let text = r#"(persons: [
        (name: "Mara", ties: [(with: "Ada", affinity: 0.2, kin: Some(Child))]),
        (name: "Ada", ties: [(with: "Mara", affinity: 0.2, kin: Some(Sibling))]),
    ])"#;
    let mut app = headless(SocietyDefs::from_ron("()").expect("parses"), settings());
    let err = PlacementFile::from_ron(text)
        .and_then(|file| file.spawn(app.world_mut()).map(|_| ()))
        .expect_err("a child's tie answered by a sibling's is a contradiction");
    let message = err.to_string();
    assert!(
        message.contains("Mara") && message.contains("Ada") && message.contains("Parent"),
        "the error must name both people and the relation expected: {message}"
    );
    assert_eq!(
        app.world_mut().query::<&Ties>().iter(app.world()).count(),
        0,
        "nothing spawns on a refused file"
    );
}

/// **The editor's round trip keeps the family.** Capturing the world back to a file writes the relation each
/// tie holds, and spawning that file again gives the same ties.
#[test]
fn capturing_the_world_keeps_the_relations() {
    let mut app = headless(SocietyDefs::from_ron("()").expect("parses"), settings());
    PlacementFile::from_ron(AUTHORED)
        .expect("parses")
        .spawn(app.world_mut())
        .expect("spawns");
    let file = PlacementFile::capture(app.world_mut()).expect("captures");
    let kin_of = |name: &str, with: &str| {
        file.persons
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.ties.iter().find(|t| t.with == with))
            .and_then(|t| t.kin)
    };
    assert_eq!(kin_of("Mara", "Ada"), Some(Kin::Child));
    assert_eq!(kin_of("Ada", "Mara"), Some(Kin::Parent));
    assert_eq!(kin_of("Bram", "Ada"), None);
    let again = PlacementFile::from_ron(&file.to_ron().expect("writes")).expect("reads");
    assert_eq!(file, again);
}

/// **Scripts can ask.** `ctx.kin(a, b)` is `r` read at the tick start, `0` for a stranger.
#[test]
fn a_script_reads_the_relatedness() {
    let mut app = headless(
        SocietyDefs::from_ron(
            r##"(triggers: [(id: "ask", on: Tick,
                then: "let p = ctx.persons(); fx.emit(\"kin\", #{ ab: ctx.kin(p[0], p[1]), ba: ctx.kin(p[1], p[0]), ac: ctx.kin(p[0], p[2]) })")])"##,
        )
        .expect("parses"),
        settings(),
    );
    let a = spawn_person(&mut app, "A", Vec2::ZERO);
    let b = spawn_person(&mut app, "B", Vec2::new(1.0, 0.0));
    spawn_person(&mut app, "C", Vec2::new(2.0, 0.0));
    app.update();
    {
        let mut ties = app.world_mut().get_mut::<Ties>(a).unwrap();
        ties.entry(b).kin = Some(Kin::AuntUncle);
        let mut ties = app.world_mut().get_mut::<Ties>(b).unwrap();
        ties.entry(a).kin = Some(Kin::NieceNephew);
    }
    let mut seen = None;
    for _ in 0..8 {
        app.update();
        for event in app
            .world_mut()
            .resource_mut::<Messages<ScriptEvent>>()
            .drain()
        {
            if event.name == "kin" {
                seen = Some(event.data);
            }
        }
    }
    let data = seen.expect("the trigger fired after the ties were set");
    let value = |key: &str| {
        data.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| match v {
                ScriptValue::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(f32::NAN)
    };
    assert_eq!(value("ab"), 0.25, "an aunt or uncle is r = 0.25");
    assert_eq!(value("ba"), 0.25, "and relatedness is symmetric");
    assert_eq!(value("ac"), 0.0, "a stranger is 0");
}

/// **A society with no relative is the society it was**, whatever the premium: nothing is multiplied for a
/// tie that names no kin, so the whole state hashes the same at 0 and at 1.
#[test]
fn the_premium_changes_nothing_where_nobody_is_related() {
    let state = |premium: f32| {
        let (mut app, _) = gift_village(7, premium, &[], &[4.0, 0.0, 0.0, 1.0]);
        run(&mut app, 900);
        common::hash::hash(&app)
    };
    assert_eq!(state(0.0), state(1.0));
}

/// **Determinism with a family**: two runs of the same relatives from the same seed are the same society.
#[test]
fn a_society_with_relatives_is_deterministic() {
    let state = || {
        let (mut app, _) = gift_village(
            11,
            1.0,
            &[(1, Some(Kin::Sibling)), (2, Some(Kin::Cousin))],
            &[4.0, 0.0, 0.0, 1.0],
        );
        run(&mut app, 900);
        common::hash::hash(&app)
    };
    assert_eq!(state(), state());
}

/// **The default tie is no relative**, and a relative's coefficient is the published one.
#[test]
fn the_published_coefficients() {
    assert_eq!(Tie::new(Entity::PLACEHOLDER).kin, None);
    assert_eq!(Tie::new(Entity::PLACEHOLDER).relatedness(), 0.0);
    for (kin, r) in [
        (Kin::Parent, 0.5),
        (Kin::Child, 0.5),
        (Kin::Sibling, 0.5),
        (Kin::HalfSibling, 0.25),
        (Kin::Grandparent, 0.25),
        (Kin::Grandchild, 0.25),
        (Kin::AuntUncle, 0.25),
        (Kin::NieceNephew, 0.25),
        (Kin::Cousin, 0.125),
    ] {
        assert_eq!(kin.relatedness(), r, "{kin:?}");
        assert_eq!(kin.converse().converse(), kin, "{kin:?} has a converse");
        assert_eq!(
            kin.converse().relatedness(),
            r,
            "relatedness is symmetric: {kin:?}"
        );
    }
}

/// **A save carries the family**, and one from before kin existed loads as no relatives: the field is
/// absent in an old file and `#[serde(default)]` makes it `None`.
#[cfg(feature = "serde")]
#[test]
fn a_save_carries_the_relations_and_an_old_one_loads_without() {
    use bevy_society::{capture, restore};

    let relations = [(1, Some(Kin::Sibling)), (2, Some(Kin::Cousin))];
    let kin_of = |app: &App, people: &[Entity], who: usize| {
        app.world()
            .get::<Ties>(people[0])
            .and_then(|ties| ties.get(people[who]))
            .and_then(|tie| tie.kin)
    };

    let (mut original, _) = gift_village(1, 1.0, &relations, &[4.0, 0.0, 0.0]);
    run(&mut original, 4);
    let json = serde_json::to_string(&capture(original.world_mut())).expect("serialises");

    // A fresh society with no relatives, then the save.
    let (mut reloaded, people) = gift_village(1, 1.0, &[], &[4.0, 0.0, 0.0]);
    run(&mut reloaded, 4);
    assert_eq!(kin_of(&reloaded, &people, 1), None);
    restore(
        reloaded.world_mut(),
        serde_json::from_str(&json).expect("reads"),
    )
    .expect("the save applies");
    assert_eq!(kin_of(&reloaded, &people, 1), Some(Kin::Sibling));
    assert_eq!(kin_of(&reloaded, &people, 2), Some(Kin::Cousin));

    // The same file as an older release wrote it: no `kin` key anywhere.
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("kin");
                map.values_mut().for_each(strip);
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut old: serde_json::Value = serde_json::from_str(&json).expect("reads");
    strip(&mut old);
    assert!(
        !old.to_string().contains("\"kin\""),
        "the old file has no kin"
    );
    let (mut third, people) = gift_village(1, 1.0, &relations, &[4.0, 0.0, 0.0]);
    run(&mut third, 4);
    assert_eq!(kin_of(&third, &people, 1), Some(Kin::Sibling));
    restore(
        third.world_mut(),
        serde_json::from_value(old).expect("an old save still parses"),
    )
    .expect("the old save applies");
    assert_eq!(
        kin_of(&third, &people, 1),
        None,
        "a save with no kin field restores every tie as no relative"
    );
}
