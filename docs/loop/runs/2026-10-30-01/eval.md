# Eval

## Before / after, same seed, same scenario

### Village, 4200 ticks (24 people, 28 rumors)

| activity | person-ticks before | after | change |
|---|---|---|---|
| `chat` | 8746 | **9312** | +6.5% |
| `gossip` | 15050 | 14160 | −6% |
| `snub` | 6004 | **1617** | **−73%** |
| `work` | 14601 | 17086 | +17% |
| `eat` | 14824 | 16931 | +14% |

Social outcomes: 91 friends, 0 rivals, reach 0.500.

### Cluster, 1200 ticks (12 people, 279 rumors)

| activity | before | after |
|---|---|---|
| `chat` | 128 | 140 |
| `gossip` | 3093 | 3186 |

Essentially unchanged, and the reason is measured: 279 rumors means `tellable` is true for almost
everybody, so the gate never closes. Recorded rather than tuned further.

## The expression gap this iteration closed

`ctx.knows(a, rumor)` needs a rumor id an authored file cannot name; nothing in `ctx` reported
anything about a person's knowledge. `ctx.tellable(a)` answers "does `a` know a rumor it is still
spreading", which is exactly the condition a gossip definition needs.

Pinned by `ctx_tellable_reports_whether_a_person_has_a_rumor_to_spread`: the same definitions choose
`doze` before a rumor exists and `tell` after one is planted.

## The capture

`{"rumor_reach": 0.360, "friends": 80, "rivals": 37, "barks": 3648}` — all four thresholds met.

## Gates

| Command | Result |
|---|---|
| `cargo test` | 17 binaries ok, `tests/rumor.rs` now 6 tests |
| `cargo test --no-default-features` | 17 binaries ok |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean, inventory still zero |

## Dead-code inventory

Unchanged at zero. `ctx.tellable` is called from the starter pack, so it is reachable by
construction — and the test proves the path.
