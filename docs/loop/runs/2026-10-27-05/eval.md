# Eval
Scenario: one person, one unreachable tavern under `Locomotion::Host` (see `evidence/narratives/stuck-walker.md`). Regenerate: the setup in `tests/stall.rs::app`, printing `Doing::label` changes and `ActivityEnded` per tick for `stall_timeout_hours` None and Some(1.0) over 1200 ticks.

| setting | endings in 7.0 game-hours | first ending |
|---|---|---|
| None | 0 | never |
| Some(1.0) | 7 (interrupted) | 09.01 h, 1.00 h after the start |

Default `None` changes no code path except one `Option` test per travelling person per tick, so every existing scenario is unchanged by construction; the full suite (below in the PR) confirms it. Perf: not measurable on a load-12 host; nothing runs at all when the timeout is `None` beyond an `is_some()` check.
