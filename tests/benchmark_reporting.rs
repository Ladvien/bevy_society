//! **The benchmark reports a spread, or its numbers are not measurements.**
//!
//! `docs/guide/concepts.md` carried `13.5 ms` at 1 000 people for nine iterations, and it could not be
//! reproduced: the fastest window measurable on this hardware is **28.4 ms**. The reason it survived is that
//! `examples/benchmark` reported a **single mean** — a statistic that, on a machine shared with other work,
//! describes the load rather than the code. Measured on one binary in one afternoon:
//!
//! | persons | fastest window | median of 9 | spread |
//! |---|---|---|---|
//! | 100 | **0.74** | 1.08 | 103% |
//! | 1 000 | **28.4** | 60.4 | **100%** |
//! | 2 000 | **100.6** | 140.1 | 49% |
//!
//! The fastest window reproduced to **0.4%** across separate invocations while the median moved 18%. So the
//! tool now takes `--reps` and prints `ms_per_tick_min`, `ms_per_tick_median` and `spread_pct`, and this file
//! pins that contract: a future change that drops back to one number, or stops printing the spread, fails here.
//!
//! The tool is built and run rather than unit-tested because the whole subject is *what it reports*.
use std::process::Command;

/// Runs the benchmark and returns its single JSON line.
fn run(reps: usize) -> String {
    // `cargo run` keeps this correct under `--release`, `--no-default-features` and a custom target dir,
    // rather than guessing where the binary was placed.
    let output = Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "--example",
            "benchmark",
            "--",
            "--people",
            "40",
            "--ticks",
            "12",
            "--settle",
            "40",
            "--reps",
            &reps.to_string(),
        ])
        .output()
        .expect("the benchmark example must build and run");
    assert!(
        output.status.success(),
        "the benchmark exited with {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .last()
        .unwrap_or_default()
        .to_owned()
}

/// Reads a numeric field out of the tool's flat JSON line.
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let needle = format!("\"{key}\": ");
    let start = line
        .find(&needle)
        .unwrap_or_else(|| panic!("the benchmark line has no `{key}`: {line}"))
        + needle.len();
    let rest = &line[start..];
    let end = rest
        .find([',', '}'])
        .unwrap_or_else(|| panic!("`{key}` has no terminator: {line}"));
    rest[..end].trim()
}

#[test]
fn the_benchmark_reports_minimum_median_and_spread() {
    let line = run(3);
    for key in [
        "ms_per_tick_min",
        "ms_per_tick_median",
        "ms_per_tick_max",
        "spread_pct",
    ] {
        let value: f64 = field(&line, key)
            .parse()
            .unwrap_or_else(|_| panic!("`{key}` is not a number in: {line}"));
        assert!(value >= 0.0, "`{key}` must be non-negative: {line}");
    }

    let min: f64 = field(&line, "ms_per_tick_min").parse().unwrap_or(0.0);
    let median: f64 = field(&line, "ms_per_tick_median").parse().unwrap_or(0.0);
    let max: f64 = field(&line, "ms_per_tick_max").parse().unwrap_or(0.0);
    let spread: f64 = field(&line, "spread_pct").parse().unwrap_or(0.0);

    assert!(
        min <= median && median <= max,
        "min {min}, median {median}, max {max} are not ordered: the spread statistic would be meaningless"
    );
    // And the spread must actually describe the three windows, not be a constant the tool prints.
    let expected = if median > 0.0 {
        (max - min) / median * 100.0
    } else {
        0.0
    };
    assert!(
        (spread - expected).abs() < 0.2,
        "spread_pct says {spread} but the windows imply {expected:.1}: the figure does not describe what it \
         reports"
    );
}

#[test]
fn one_repetition_is_allowed_and_still_reports_the_field() {
    // `--reps 1` is the degenerate case, and it must still print the spread keys rather than omitting them —
    // a consumer parsing the line should not have to special-case it.
    let line = run(1);
    assert!(
        line.contains("\"ms_per_tick_min\""),
        "even a single repetition must report the spread fields: {line}"
    );
    let min: f64 = field(&line, "ms_per_tick_min").parse().unwrap_or(-1.0);
    let max: f64 = field(&line, "ms_per_tick_max").parse().unwrap_or(-1.0);
    assert!(
        (min - max).abs() < f64::EPSILON,
        "with one repetition min and max must coincide, got {min} and {max}"
    );
}
