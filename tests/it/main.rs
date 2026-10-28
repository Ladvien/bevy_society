//! **The integration tests, one binary.**
//!
//! Every file in this directory is a module of this crate, so `cargo test` builds and links ONE test
//! binary for all of them instead of one per file. That is the point: each of the 74 binaries it
//! replaces re-linked the whole Bevy umbrella, and at `CARGO_BUILD_JOBS=3` those links were most of
//! the 22 minutes the `--no-default-features` gate spent compiling.
//!
//! **A new test goes here:** create `tests/it/<name>.rs` and add `mod <name>;` below, in order. The
//! `registration` tests fail if a file here has no `mod` line (an undeclared file is never compiled, so
//! its tests would never run) or if a new file lands directly in `tests/`.
//!
//! The tests share a process under plain `cargo test`, so none of them may install anything global (a
//! tracing subscriber, a panic hook, an environment variable). A test that must stays its own binary in
//! `tests/` and is named, with the reason, in `registration::SEPARATE`. Under `cargo nextest` every test
//! has its own process regardless.
//!
//! Select one file's tests with the module name: `cargo test --test it reciprocity::`.

// The shared harness stays in `tests/common/` because the separate binaries still use it.
#[path = "../common/mod.rs"]
mod common;

mod appraisal_balance;
mod behavior;
mod benchmark_reporting;
mod candidate_bias;
mod clock;
mod cluster_reach;
mod commotion;
mod contact_gossip;
mod core;
mod crowded_arrival;
mod crowded_place;
mod deceased;
mod deed_response;
mod defs;
mod degenerate;
mod destination;
mod determinism;
mod dialogue;
mod distance;
mod doing_label;
mod dormant;
mod drift;
mod economy;
mod exposure_equilibrium;
mod failed_place;
mod field_scores;
mod fields;
mod fields_unscored;
mod friendship_bound;
mod groups;
mod guide_blocks;
mod knowledge_bound;
mod label_hysteresis;
mod leaf;
mod leisure;
mod locomotion;
mod mind;
mod mood_offset;
mod nearest;
mod need_budget;
mod need_cycle;
mod obstacles;
mod opinion_convergence;
mod opinions;
mod perception;
mod profession;
mod readme;
mod reciprocity;
mod reconciliation;
mod registration;
mod rejection;
mod relationships;
mod report_deed;
mod reputation_patient;
mod rules;
mod rumor;
mod rumor_book;
mod save_upgrade;
mod scenario_spec;
mod script_economy;
mod signals;
mod sleep;
mod snub;
mod stall;
mod steering;
mod telling;
mod trace;
mod trade;
mod travel;
mod travel_cost;
mod trust_rises;
mod tuning_covers_rates;
mod village_evening;
mod village_layout;
mod witness_dilution;
