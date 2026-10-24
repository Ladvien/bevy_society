# Eval

## Change

1. **`tests/village_evening.rs`** — the scenario's observable, asserted, for the first time.
2. **Both plant ticks stated.** The file says 300; the harness plants at 8 960. Both are now documented, with
   the measurement that shows they disagree in outcome.
3. **Villagers only.** The rumor must reach someone who can pass it on.

## Measured

| planted at | seeds spreading it | villagers reached |
|---|---|---|
| **300** (the file's claim) | **7 of 7** | 2–8 |
| 8 960 (the harness) | 8 of 10 | 1–5 |

## What was actually wrong

Not the gossip model. Three separate things:

1. **The observable was never asserted.** `grep` across `tests/` found nothing for `village-evening` despite
   `status: measured`. Four iterations of model change passed through it unverified.
2. **Two different plant ticks**, thirty times apart, one documented and one executed.
3. **A holder-count that included an agent that cannot act**, so a rumor reaching only the player would have
   satisfied an observable about being "told to at least one other person."

## What I got wrong on the way

Two wrong hypotheses, both measured and discarded: that `knowledge_capacity` eviction was the cause (nobody
holds more than **26** rumors against a cap of **64**), and that the test's 4 200-tick length at the harness's
8 960 tick would work (the plant never happened — **0 of 5**). The second was an obvious own goal; the first
was worth ruling out because it would have been a crate defect rather than a documentation one.

## Gates

| Command | Result |
|---|---|
| `cargo test` | **49** binaries |
| `cargo test --no-default-features` | 49 binaries |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |

## Dead-code inventory

Zero.
