//! **The harness runs each scenario at the settings its spec declares.**
//!
//! For eight iterations `examples/evidence.rs` hardcoded its own values and never read
//! `docs/loop/scenarios.yaml`. For `village-evening` it used `seconds_per_day = 4.0` where the file says
//! **64.0** — a day sixteen times shorter — so a four-hour activity lasted four ticks while a walk to
//! the next place took three hundred. Every "needs starve" and "the rumor never spreads" measurement in
//! those iterations was taken at that wrong day length.
//!
//! The same scenario at the settings it declares, 24 people over 17 game-days:
//!
//! | | hardcoded 4.0 s/day | spec 64.0 s/day |
//! |---|---|---|
//! | needs mean | [0.11, 0.21, 0.08, 0.11] | **[0.88, 0.77, 0.50, 0.81]** |
//!
//! These tests read the same file the harness does, so a spec that loses a key, or a harness that stops
//! reading it, fails here.
mod common;

use bevy_society::{DefsFile, SocietyDefs};

/// The spec reader's rules, restated: the file's `settings:` block for `name`, in `{ key: value }` form.
fn spec_line(name: &str) -> Option<String> {
    const YAML: &str = include_str!("../docs/loop/scenarios.yaml");
    let mut in_scenario = false;
    for line in YAML.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- name:") {
            in_scenario = rest.trim() == name;
            continue;
        }
        if in_scenario && trimmed.starts_with("settings:") {
            return Some(trimmed.to_owned());
        }
    }
    None
}

#[test]
fn every_scenario_declares_a_day_length() {
    const YAML: &str = include_str!("../docs/loop/scenarios.yaml");
    let names: Vec<&str> = YAML
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- name:"))
        .map(|rest| rest.trim())
        .collect();
    assert!(
        names.len() >= 7,
        "the file declares the scenarios: {names:?}"
    );
    for name in names {
        let line = spec_line(name).unwrap_or_else(|| panic!("{name} must declare settings"));
        assert!(
            line.contains("seconds_per_day"),
            "{name}'s settings must state its day length: {line}"
        );
    }
}

#[test]
fn the_village_scenarios_use_a_day_a_plausible_activity_can_live_in() {
    // At the crate's default 240 s a day, a 0.1-hour `snub` is 0.3 ticks. At 4 s a day a four-hour
    // activity is 4 ticks while an 8 m walk is 375. Both make travel dominate every activity, which is
    // how eight iterations mistook a harness setting for a model defect.
    for name in ["village-evening", "rumor-crosses-a-cluster"] {
        let line = spec_line(name).expect("the scenario declares settings");
        assert!(
            line.contains("seconds_per_day: 64.0"),
            "{name} should run a 64-second day, which makes a walk about one game-hour: {line}"
        );
        // And a timeout that reaches across the village at that day length: 20 game-hours is ~75 m.
        assert!(
            line.contains("travel_timeout_hours"),
            "{name} must state a travel timeout fitted to its day length: {line}"
        );
    }
}

#[test]
fn the_starter_pack_still_compiles_with_the_village_additions() {
    // The harness merges these two files; a spec change that broke the merge would otherwise surface
    // only when someone ran an example.
    let mut file = DefsFile::starter().expect("the starter pack parses");
    file.merge(
        DefsFile::from_ron(include_str!("../examples/assets/village.society.ron"))
            .expect("the village file parses"),
    )
    .expect("the two merge");
    SocietyDefs::compile(file).expect("the merged definitions compile");
}
