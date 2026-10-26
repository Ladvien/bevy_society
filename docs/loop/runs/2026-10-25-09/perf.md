# Performance: no model change

Nothing on the simulation's hot path changed this iteration: `src/reputation.rs`, `src/core.rs` and
`tests/witness_dilution.rs` are byte-identical to the commit the iteration started from (`8532a2f`), verified with
`git diff --stat HEAD`. The dilution change that was designed and measured was reverted before any commit.

The one addition is a flag in `examples/evidence.rs` (`--witness-dilution`), which is not part of the crate.

**No benchmark was run, and that is deliberate.** A run before and after identical code measures the machine, not
the change; the crate's own perf records show the within-run spread at 5 000 people has been 27-44% across
iterations. Last iteration's numbers stand: 1.579 / 1.604 ms a tick at 100 people and 533-535 ms at 5 000, with the
village-pack neighbour-store copy at about +1% at 5 000.

**One measurement worth carrying forward**: the three-population sweeps in this iteration run 30 evidence runs
(three populations, five seeds, two configurations) in about 10 minutes on this machine, which is what makes a
nine-seed cross-population comparison practical for the next iteration that changes the day.
