//! **The crate boundary, enforced.**
//!
//! `bevy_society` is a society simulation any Bevy game can take. It depends on Bevy, on the
//! scripting and serialisation crates it is built on, and on the ladvien crates it consumes by
//! pinned git revision. It must never learn about a particular game: no game-specific names, no
//! engine crates beyond that table, and no hash collection iterated in simulation order.
//!
//! The source scan forbids more than crate names. `MainCamera`, `SquadMember` and `MenuState` are
//! specific temptations from the game this crate was extracted from. Naming one again would compile
//! and silently couple the crate to that game.

use std::path::{Path, PathBuf};

/// Everything this crate is allowed to depend on.
const ALLOWED_DEPS: &[&str] = &[
    "bevy",
    "rhai",
    "serde",
    "ron",
    "rand_chacha",
    "bevy_grimoire_rng",
    "bevy_speech_bubbles",
    "bevy_stigmergy",
    "bevy_orca",
];

/// Names that would mean the boundary has been crossed.
const FORBIDDEN_DEP_MARKERS: &[&str] = &[
    "avian",
    "foundation_vs_slop",
    "sanctuary",
    "MainCamera",
    "SquadMember",
    "MenuState",
];

/// The only source files that may name `HashMap` or `HashSet`. Each uses them for lookup only; the
/// simulation walks `PersonIndex::sorted`, never a hash map.
const HASH_ALLOWED: &[&str] = &[
    "src/core.rs",
    "src/defs.rs",
    "src/script.rs",
    "src/dialogue.rs",
];

fn crate_root() -> PathBuf {
    // Cargo runs a test binary with the cwd set to its own package root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `path` relative to the crate root, with `/` separators, so it can be compared with `HASH_ALLOWED`.
fn relative(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Strip `//` line comments and `/* */` blocks so a doc comment *about* Bevy is not mistaken for an
/// import of it.
///
/// Deliberately simple: it does not track string literals, so a `"//"` inside a string ends the line
/// early. That is acceptable because the output only feeds substring searches for crate names, and an
/// over-eager strip can only cause a false pass on a line that contains no dependency.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_block = false;
    for line in src.lines() {
        let mut rest = line;
        loop {
            if in_block {
                match rest.find("*/") {
                    Some(end) => {
                        in_block = false;
                        rest = &rest[end + 2..];
                    }
                    None => break,
                }
            } else {
                let line_at = rest.find("//");
                let block_at = rest.find("/*");
                match (line_at, block_at) {
                    (Some(l), b) if b.is_none_or(|b| l < b) => {
                        out.push_str(&rest[..l]);
                        break;
                    }
                    (_, Some(b)) => {
                        out.push_str(&rest[..b]);
                        in_block = true;
                        rest = &rest[b + 2..];
                    }
                    _ => {
                        out.push_str(rest);
                        break;
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}

#[test]
fn no_source_file_reaches_for_an_engine_or_a_game() {
    let src = crate_root().join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    assert!(
        files.len() >= 2,
        "expected to scan the whole crate, found only {} file(s) — has the layout moved?",
        files.len()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let code = strip_comments(&text);
        for (n, line) in code.lines().enumerate() {
            for marker in FORBIDDEN_DEP_MARKERS {
                if line.contains(marker) {
                    offenders.push(format!("{}:{} — {}", relative(path), n + 1, line.trim()));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "bevy_society must stay game-agnostic, but {} line(s) reference a forbidden name.\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

#[test]
fn the_dependency_list_stays_closed() {
    let manifest = std::fs::read_to_string(crate_root().join("Cargo.toml"))
        .expect("bevy_society must have a Cargo.toml");

    // Only the `[dependencies]` table. Dev-dependencies (the full Bevy umbrella, fairy, serde_json)
    // are examples and tests, not part of the library's graph.
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("bevy_society must declare a [dependencies] table");
    let deps = deps.split("\n[").next().unwrap_or(deps);

    for line in deps.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // `.` is a separator too, so `serde.workspace = true` reads as `serde`.
        let name = line.split(['=', ' ', '.']).next().unwrap_or("");
        // Continuation lines of a multi-line value (feature arrays, closing brackets) are not crate
        // names; skip them by shape.
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        assert!(
            ALLOWED_DEPS.contains(&name),
            "bevy_society declares `{name}`, which is not in its allowed set {ALLOWED_DEPS:?}. \
             Widening this is a design decision: add it to ALLOWED_DEPS in this test in the same \
             commit so the change is visible in review."
        );
        for marker in FORBIDDEN_DEP_MARKERS {
            assert!(
                !name.contains(marker),
                "bevy_society declares `{name}`, which contains the forbidden marker `{marker}`."
            );
        }
    }
}

#[test]
fn the_pure_model_is_engine_free() {
    let mut files = Vec::new();
    rust_sources(&crate_root().join("src/model"), &mut files);
    assert!(
        !files.is_empty(),
        "expected src/model to hold the engine-free math; has the layout moved?"
    );

    let mut offenders = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let code = strip_comments(&text);
        for (n, line) in code.lines().enumerate() {
            if line.contains("bevy") {
                offenders.push(format!("{}:{} — {}", relative(path), n + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "src/model/** must be plain math with no `bevy` in code, but {} line(s) name it.\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

#[test]
fn hash_collections_stay_lookup_only() {
    let mut files = Vec::new();
    rust_sources(&crate_root().join("src"), &mut files);

    let mut offenders = Vec::new();
    for path in &files {
        let rel = relative(path);
        if HASH_ALLOWED.contains(&rel.as_str()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let code = strip_comments(&text);
        for (n, line) in code.lines().enumerate() {
            if line.contains("HashMap") || line.contains("HashSet") {
                offenders.push(format!("{rel}:{} — {}", n + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "HashMap/HashSet may appear only in {HASH_ALLOWED:?} (lookup only); found {} line(s) \
         elsewhere. Simulation loops walk PersonIndex::sorted instead.\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}
