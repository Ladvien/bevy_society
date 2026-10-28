mod common;
use bevy::prelude::*;
use bevy_society::*;
use common::{headless, spawn_person};
fn run(stall: Option<f32>) {
    let defs = SocietyDefs::from_ron(r#"(needs: [(id: "food", decay_per_day: 0.9)], activities: [(id: "eat", target: Place("food"), duration_hours: 0.75, considerations: [(input: Need("food"), curve: Linear(-1.0, 1.0))], satisfies: [("food", 1.2)])])"#).unwrap();
    let mut s = SocietySettings { locomotion: Locomotion::Host, tick_secs: 1.0/64.0, seconds_per_day: 64.0, ..SocietySettings::default() };
    s.tuning.travel_timeout_hours = None; s.tuning.stall_timeout_hours = stall;
    let mut app = headless(defs, s);
    let home = app.world_mut().spawn((Name::new("home"), Place{capacity:4}, PlaceTags(vec![]), Transform::default())).id();
    app.world_mut().spawn((Name::new("tavern"), Place{capacity:8}, PlaceTags(vec!["food".into()]), Transform::from_xyz(8.0,0.0,0.0)));
    let p = spawn_person(&mut app, "Bram", Vec2::ZERO);
    app.world_mut().entity_mut(p).insert((Home(home), Autonomous));
    let mut last = String::new();
    for t in 0..1200 {
        app.update();
        let n = app.world().resource::<SocietyClock>().now_days;
        for e in app.world_mut().resource_mut::<bevy::ecs::message::Messages<ActivityEnded>>().drain() { println!("{:?}|tick {t}|{:.2} h|ended {} completed={}", stall, n*24.0, e.activity, e.completed); }
        let d = app.world().get::<Doing>(p).map(|d| format!("{}", d.label(app.world().resource::<SocietyDefs>()))).unwrap_or("idle".into());
        if d != last { println!("{:?}|tick {t}|{:.2} h|now {d}", stall, n*24.0); last = d; }
    }
}
#[test] fn ev() { run(None); run(Some(1.0)); }
