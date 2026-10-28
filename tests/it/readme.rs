//! **The README's quick start is the crate's quick start.**
//!
//! It said "which is this code" and was not: it had lost the `TimeUpdateStrategy` that makes a
//! headless app advance, so a host pasting it got a society that took **zero** ticks in 600 updates
//! — measured, not guessed. Fixing the README by hand was not enough either: the first fix named a
//! `TimeUpdateStrategy` that Bevy does not re-export in its prelude, which only a compiling check
//! caught.
//!
//! So there is one source of truth: the doctest in `src/lib.rs`, which `cargo test --doc` compiles
//! **and runs**. This test asserts the README's block is that same text, byte for byte.

/// A fenced code block's contents, given the opening fence line.
fn block_after<'a>(haystack: &'a str, fence: &str) -> &'a str {
    let start = haystack
        .find(fence)
        .unwrap_or_else(|| panic!("no `{fence}` block"));
    let rest = &haystack[start + fence.len()..];
    let end = rest.find("```").expect("the block is closed");
    &rest[..end]
}

/// The canonical snippet: the running doctest in `src/lib.rs`, with its `//!` prefixes removed.
fn canonical() -> String {
    let lib = include_str!("../../src/lib.rs");
    block_after(lib, "//! ```\n")
        .lines()
        .map(|line| {
            line.strip_prefix("//! ")
                .or_else(|| line.strip_prefix("//!"))
                .unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_readme_quick_start_is_the_crate_doctest() {
    let readme = block_after(include_str!("../../README.md"), "```rust,no_run\n");
    assert_eq!(
        readme.trim(),
        canonical().trim(),
        "the README's quick start has drifted from the doctest in src/lib.rs; \
         edit the doctest and copy it into the README, not the other way round"
    );
}

#[test]
fn the_quick_start_pins_the_fixed_step() {
    // The one line a headless host cannot omit: without it `Time<Fixed>` has no clock and the
    // society takes zero ticks however many updates are run.
    assert!(
        canonical().contains("TimeUpdateStrategy::ManualDuration"),
        "the quick start must pin the fixed step, or a host gets a frozen society"
    );
}

#[test]
fn the_getting_started_guide_does_not_hand_a_host_a_panic() {
    // The guide's first block used to be `add_plugins(SocietyPlugins)` over `DefaultPlugins`, which
    // panics in `DialoguePlugin::finish` without a speech-bubbles camera — measured, not guessed:
    //
    //     bevy_society: DialoguePlugin needs bevy_speech_bubbles::SpeechBubblesPlugin::<YourCamera>
    //
    // This asserts the guide now warns about it and shows the disable, so a host reading the guide
    // top to bottom does not hit a panic.
    let guide = include_str!("../../docs/guide/getting-started.md");
    let block = block_after(guide, "```rust,no_run\n");
    assert!(
        block.contains("disable::<bevy_society::DialoguePlugin>"),
        "the getting-started guide must show how to leave dialogue out until the host has a camera"
    );
    assert!(
        guide.contains("panics in `finish`"),
        "the guide must say that DialoguePlugin panics without a speech-bubbles plugin"
    );
}
