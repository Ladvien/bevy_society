# Research: the demos were not showing the features

## What a host-facing demo owes

`docs/DEMOS.md` says each clip is "a different *view* of one simulation, not a different simulation". The
previous six failed that in the same way six times: they were views of the *same pixels*. Every clip showed
two dozen capsules standing still in three clusters, and the only thing distinguishing "mood" from
"opinions" from "ties" was a thin overlay that, on inspection, was drawing the wrong topic.

A visual review at frames 0 / 120 / 239 found, in order of severity:

| fault | evidence | cause |
|---|---|---|
| villagers drawn inside the buildings | half-bodies from a cream cube | places 3 m cubes, villagers spawned **0.6 m** from centre |
| nobody ever moved | identical clusters, frame 0 to 239 | place half-width **1.5 m** vs `interaction_radius` **1.5 m**: "arrived" meant standing inside the block |
| the same | no walking visible | at `seconds_per_day = 8` a **3 m walk cost 6.4 game-hours** against a 0.3-hour chat |
| `needs` one colour | 24 identical red halos | `NEED_CLIP = 1` is `rest`; `food` is 2 |
| `opinions` one colour | all-green bars | `OpinionBar(0)` is `kindness`; `honesty` is 1 |
| `village` showed no rumor | HUD 10/24, zero halos | rings read `RumorId(0)`, the first rumor of the run |
| `factions` two colours | millers out of shot | fixed camera + `play` targets `Wander(6.0)` |

## The one that is a crate defect, not a demo defect

`steer_movers` calls `bevy_orca::new_velocity(..., &[], ...)` — the wall parameter exists and the crate
fills it with nothing. `src/steering.rs` said so plainly: *"the crate knows no level geometry, so walls are
not passed to ORCA; games with walls drive movement themselves."* A host with a tavern therefore had **no
way to say it is solid**, and the person's `Transform` ended inside it.

`bevy_orca` already accepts hard wall half-planes as `(b, approach)` — a unit vector toward the solid and
the fastest the agent may still close on it — and its own comment states the division of labour:
*"Actual contact is still the caller's collision resolver."* So the solver needed nothing new; the crate
needed a component and a resolver.

This is the same shape as the `Distance` gap two iterations ago: a capability present in the dependency and
unreachable from the crate, with the absence of it visible only in a picture.

## Sources

- **van den Berg, Lin & Manocha 2011**, *Reciprocal n-body Collision Avoidance*, DOI
  `10.1109/TRO.2011.2120810` — already the crate's citation for `steering`; ORCA is what runs.
- **Ericson 2005**, *Real-Time Collision Detection*, §7.1 on uniform grids — read for the perception work
  last iteration, and its warning about a grid "too coarse with respect to object size" applies here in
  reverse: three capsules inside a 3 m box is a scene-construction error, not a geometry one.

## What believable output looks like

A person stands *beside* the stall it has come to, walks between stalls as the day turns, and is drawn in a
colour that identifies its faction at a glance. A feature clip shows a quantity that **changes**: halos that
differ between villagers, bars of visibly different length and colour, a need that visibly falls.
