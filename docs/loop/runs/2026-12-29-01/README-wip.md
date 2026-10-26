# Superseded: this branch's change shipped upstream with a different shape

The nine-run evidence in `evidence/after/` is complete; `evidence/before/` is **empty**, so no before/after
comparison exists here and no verdict below is claimable from it.

`origin/main` shipped the same defect fixed as a **step** price (`Tuning::crowding_penalty`, 0.6, commit
`87f357d`, run `2026-10-25-10`) while this branch was building an **exponential** one
(`Tuning::place_crowding_penalty`, 1.0). The WIP's own table (1.09% / 1.75% / 1.24% hostile at 24 / 36 / 48)
is flatter than the shipped step's (2.90% / 6.10% / 3.94%), i.e. **further** from the 7-8% band the target
names, so the shipped shape is the better of the two by this branch's own numbers.

Branch preserved, not merged. The design record (Talamali Eq. 12 shape, `capacity` as `N_crit`, the
`Home`-is-not-priced asymmetry, the `capacity: 0` prohibition) is in `research.md` and `plan.md` here.
