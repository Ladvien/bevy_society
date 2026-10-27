# Perf

The change adds one `add_affinity` call to `patient_reaction`, which runs once per accepted deed (not per tick, not per
tie). No allocation, no iteration, no draw. The machine ran at load average 22–30 for the whole iteration (other
repos' CI runners), so wall-clock comparisons carry the 22–54% spreads recorded in `2026-10-27-02/perf.md`; a benchmark
here would distinguish nothing. Cost is per deed and bounded by one clamp. `examples/benchmark` also still builds an
unsittable home for every population (log item 4), so its numbers do not describe `village-evening`.
