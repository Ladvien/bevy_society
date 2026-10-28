# review
Evidence: `evidence/{before,after}/seed{7,42,11}/{summary.json,narrative.txt}`; no matplotlib on this host, so the comparison is the table in `eval.md` rather than a PNG (recorded, not hidden).

1. **Behavior — GREAT.** Critique: the fix must grow the allowance with arrivals beyond the radius. `the_allowance_grows_with_arrivals_standing_beyond_the_radius` (8 arrived on a 2.0 m ring, walker at 3.0 m, radius 1.5) fails under the old count (0 inside 1.5 m) and passes now; empty disc still refuses at 2.2 m.
2. **Zoom — GREAT.** Critique: look for blow-up of the allowance. It is bounded by `radius + growth*sqrt(capacity)`; interrupted endings fall at all three seeds, none rises; rivals do not move in one direction (+6/-4/-9), so no hostility side effect is visible.
3. **Fresh eyes — GREAT.** `evidence/after/seed7/narrative.txt` still reads as a day (started/finished/accepted/refused pairs with reasons), and the change is explainable in one line: the crowd is who has arrived, not who is within a disc. Different-seed check: seeds 42 and 11 repeat the direction.
