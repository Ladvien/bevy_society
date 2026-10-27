mod common;
use bevy::prelude::*;
use bevy_society::{MemoryKind, ReportDeed, SocietyDefs, Tie, Ties};
use common::{headless, spawn_person};

fn defs() -> SocietyDefs {
    SocietyDefs::from_ron(r#"(
        needs: [(id: "social", decay_per_day: 0.2)],
        topics: [(id: "kindness")],
        deeds: [(id: "insult", topic: "kindness", harm: -0.4)],
        activities: [(id: "idle", target: Wander(1.0), duration_hours: 1.0, considerations: [], interaction: None)],
    )"#).expect("parses")
}

fn affinity_of(app: &App, from: Entity, to: Entity) -> f32 {
    app.world().get::<Ties>(from).and_then(|t| t.get(to)).copied()
        .map(|t: Tie| t.affinity).unwrap_or(0.0)
}

fn trust_of(app: &App, from: Entity, to: Entity) -> f32 {
    app.world().get::<Ties>(from).and_then(|t| t.get(to)).copied()
        .map(|t: Tie| t.trust).unwrap_or(0.5)
}

fn memories(app: &App, who: Entity) -> Vec<String> {
    app.world().get::<bevy_society::Memories>(who).map(|m| m.entries.iter()
        .map(|e| format!("{:?} v={:.3} i={:.3}", e.kind, e.valence, e.importance)).collect())
        .unwrap_or_default()
}

#[test]
fn probe() {
    let mut settings = common::settings();
    settings.seconds_per_day = 64.0;
    let mut app = headless(defs(), settings);
    app.update(); app.update();
    let actor = spawn_person(&mut app, "Bram", Vec2::ZERO);
    let patient = spawn_person(&mut app, "Ada", Vec2::new(1.0, 0.0));
    app.update();
    println!("BEFORE: patient->actor affinity {:.4} trust {:.4}", affinity_of(&app, patient, actor), trust_of(&app, patient, actor));
    println!("        patient memories {:?}", memories(&app, patient));
    app.world_mut().write_message(ReportDeed {
        actor, patient: Some(patient), deed: "insult".to_owned(),
        witnesses: vec![patient],
    });
    app.update(); app.update(); app.update();
    println!("AFTER:  patient->actor affinity {:.4} trust {:.4}", affinity_of(&app, patient, actor), trust_of(&app, patient, actor));
    println!("        patient memories {:?}", memories(&app, patient));
    println!("        actor has tie? {:?}", app.world().get::<Ties>(actor).map(|t| t.get(patient).copied()));
}
