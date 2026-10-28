//! **A test that is not declared never runs.**
//!
//! Every file in `tests/it/` is a module of one test binary, and a file that `main.rs` does not
//! `mod` is not compiled at all: it would sit in the repository looking like coverage while
//! asserting nothing. That is the failure a loop pass can fall into when it adds a test the way
//! the old layout allowed (drop `tests/<name>.rs` in and Cargo finds it). These tests fail instead.
//!
//! Two rules, both checked against the directory as it is on disk:
//!
//! 1. every `tests/it/<name>.rs` has a `mod <name>;` line in `tests/it/main.rs`;
//! 2. every other `tests/*.rs` is named in [`SEPARATE`], each with the reason it is its own binary.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Test files that stay their own binary in `tests/`, and why. A reason that is not about process-global
/// state is a reason to move the file into `tests/it/` instead.
///
/// Tests in `tests/it/` share one process under plain `cargo test`, so none of them may install anything
/// global. These do (a tracing subscriber, directly or through `LogPlugin`, which sets the process-wide
/// default), and a second installer in the same process silently gets nothing.
const SEPARATE: &[(&str, &str)] = &[
    (
        "bound_places",
        "captures log events through `LogPlugin`, which installs the process-wide subscriber",
    ),
    (
        "crowded_home",
        "installs a global tracing subscriber and captures through `LogPlugin`",
    ),
    (
        "save",
        "captures log events through `LogPlugin`, which installs the process-wide subscriber",
    ),
    ("unmade_goods", "installs a global tracing subscriber"),
    (
        "unreachable",
        "captures log events through `LogPlugin`, which installs the process-wide subscriber",
    ),
    (
        "unreachable_places",
        "installs a global tracing subscriber and captures through `LogPlugin`",
    ),
    (
        "warnings",
        "captures log events through `LogPlugin`, which installs the process-wide subscriber",
    ),
];

/// The names of the `.rs` files directly in `dir`, without the extension.
fn stems(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "rs"))
        .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect()
}

#[test]
fn every_file_in_tests_it_is_declared_in_main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");
    let declared: BTreeSet<String> = include_str!("main.rs")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("mod ")?.strip_suffix(';'))
        .map(str::to_owned)
        .collect();

    let undeclared: Vec<String> = stems(&dir)
        .into_iter()
        .filter(|name| name != "main" && !declared.contains(name))
        .collect();

    assert!(
        undeclared.is_empty(),
        "these files in tests/it/ have no `mod` line in tests/it/main.rs, so they are not compiled and \
         their tests never run: {}. Add `mod <name>;` for each.",
        undeclared.join(", ")
    );
}

#[test]
fn every_other_test_file_is_a_listed_exception() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let listed: BTreeSet<String> = SEPARATE
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    let on_disk = stems(&tests);

    let stray: Vec<&String> = on_disk.difference(&listed).collect();
    assert!(
        stray.is_empty(),
        "tests/{{{}}}.rs would build as separate binaries, one more link of the Bevy umbrella each. \
         Move the file to tests/it/, change `mod common;` to `use crate::common;`, and add `mod <name>;` \
         to tests/it/main.rs. If it must be its own process (it installs something global), add it to \
         SEPARATE in tests/it/registration.rs with the reason.",
        stray
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let gone: Vec<&String> = listed.difference(&on_disk).collect();
    assert!(
        gone.is_empty(),
        "SEPARATE names files that no longer exist in tests/: {}. Remove the entries.",
        gone.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
