# 2026-10-27-04 — research

Bug #25 (Sanctuary): the crowded-arrival allowance `interaction_radius + crowded_arrival_growth * sqrt(crowd)` counted bodies inside `interaction_radius`, but a walker is admitted at the larger `allowed`, so admitted people stand outside the counted disc and the allowance never grows past the first arrival.

No new model: `crowded_arrival_growth` remains a design default (one body diameter, ORCA spacing; no paper). The change is which bodies are counted. Issue's option 1 chosen: persons whose `Doing` targets the place with `arrived_day` set. Reason over option 2 (fixed point on `allowed`): option 2 also counts walkers that merely queue within the widened ring, so a line of queuers would widen the allowance for the people behind them, which the issue's edge case forbids.
