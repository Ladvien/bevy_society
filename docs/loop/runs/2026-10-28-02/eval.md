# Eval
Scenario: host-walked person who never moves, near tavern 8 m, far tavern 12 m, `stall_timeout_hours = 1`. Regenerate: `cargo +1.98.1 test --release --test failed_place`.

| setting | places walked in 4.5-5 game-hours |
|---|---|
| `None` | near, near, near, near (every window) |
| `Some(2.5)` | near, far, near (near forgiven first) |

Default `None`: no run changes (filter only runs when a memory exists; no RNG draw moved). No plots or narrative: no simulation default changed, so there is no before/after society to look at; the table above is the observed behaviour of the two tests. Perf: not measured, one `Query::get` per person per activity per decision.
