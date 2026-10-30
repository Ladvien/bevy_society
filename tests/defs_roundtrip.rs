//! A definitions file written with `to_ron` reads back as the same file, so an editor can save what
//! it changed. `DefsFile` has no `PartialEq`, so equality is the text: the second write must equal
//! the first, and the reloaded file must still compile.

use bevy_society::{DefsFile, SocietyDefs};

fn round_trip(file: &DefsFile) {
    let first = file.to_ron().expect("writes");
    let back = DefsFile::from_ron(&first).expect("reads back");
    assert_eq!(back.to_ron().expect("writes again"), first);
    SocietyDefs::compile(back).expect("reloaded file compiles");
}

#[test]
fn starter_pack_survives_a_save_and_reload() {
    round_trip(&DefsFile::starter().expect("starter"));
}

#[test]
fn the_village_pack_survives_a_save_and_reload() {
    let mut file = DefsFile::starter().expect("starter");
    let village = DefsFile::from_ron(include_str!("../examples/assets/village.society.ron"))
        .expect("village");
    file.merge(village).expect("merges");
    round_trip(&file);
}

#[test]
fn an_edited_trigger_keeps_its_condition_area_and_cooldown() {
    let mut file = DefsFile::from_ron(
        r#"(triggers: [(
            id: "poke",
            on: Custom("poke"),
            if: Some("true"),
            then: "fx.emit(\"met\", #{})",
            once: true,
            cooldown_hours: 2.5,
            area: Some((min: (0.0, 1.0), max: (10.0, 11.0))),
        )])"#,
    )
    .expect("parses");
    file.triggers[0].then = "fx.emit(\"edited\", #{})".to_owned();
    let back = DefsFile::from_ron(&file.to_ron().expect("writes")).expect("reads");
    let t = &back.triggers[0];
    assert_eq!(t.then, "fx.emit(\"edited\", #{})");
    assert_eq!(t.condition.as_deref(), Some("true"));
    assert!(t.once);
    assert_eq!(t.cooldown_hours, 2.5);
    assert_eq!(t.area, file.triggers[0].area);
}
