# Perf

## The change

One addition and one subtraction on the pleasure return path, per person per society tick. No allocation, no
lookup, no new query.

At 24 people and one society tick per 1/64 s this is below measurement; at 5 000 people it is 5 000 float
additions against a decision pass that already runs 3.6M consideration evaluations.

## Behaviour

| | before | after |
|---|---|---|
| `village-evening` mean pleasure | −0.154 | **−0.122** |
| `village-evening` mean needs | 0.596 | 0.603 |
| `village-evening` travelling | unchanged | unchanged |

Needs moved by +0.007, which is noise from the RNG stream shifting — mood feeds `SocialProof` and a
moodier village makes slightly different choices. Recorded rather than hidden: the change is not strictly
mood-only in downstream choice, but the difference is inside the run-to-run spread the scenario already has.
