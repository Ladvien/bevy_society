# Research: a `Destination` nothing walked to

## The gap, in the crate's own words

`Destination`'s doc comment says:

> Inserted and updated while travelling and removed on arrival or when the activity ends. Games
> **without the steering plugin move persons themselves toward this point.**

They did not. `BehaviorPlugin` *writes* a `Destination` for every travel target; the only system that
*reads* it is `steer_movers`, inside the `steering` feature. With `--no-default-features` — a supported
and tested configuration — a person is given a destination and stands still.

## Measured

A person at the origin, `Destination(6.5, -5.0)`, 600 ticks:

```
t=  0 doing=None
t=  1 doing=eat dest=Some((6.5, -5.0)) pos=(0.0, 0.0)
t=600 doing=eat dest=Some((6.5, -5.0)) pos=(0.0, 0.0)
```

**Never moved a millimetre.** Consequently, in a 24-person village at the current 55-hour timeout:

| activity | target | started | completed | interrupted |
|---|---|---|---|---|
| chat, gossip, compliment, snub, wander | `Person`, `Point` | 339 | **339 (100%)** | 0 |
| eat | `Place` | 62 | 14 (23%) | 43 |
| relax | `Place` | 23 | 5 (22%) | 11 |
| sleep | `Home` | 79 | 24 (30%) | 48 |
| work | `Place` | 64 | **6 (9%)** | 53 |

Only **26 of 228** `Place`/`Home` activities ever arrived. `Person`-targeted activities succeed because
the *target* walks to the actor — the crate only had to move one of the two.

Note the timeout is **not** the cause here: 55 game-hours at these settings is 587 ticks, and at
0.0219 m/tick that is **12.8 m**, against a village whose longest walk is **8.2 m**. The previous
iteration's warning was right in general and already fixed; this is a different defect.

## The fix

`walk_toward_destination`, in `BehaviorPlugin`, every fixed step:

- moves each person with a `Destination` by `Tuning::walk_speed · delta_secs` toward it;
- **skips anyone carrying a `SocialMover`**, because that is the steering plugin's person and two
  systems writing one `Transform` would fight;
- keeps the body's own height, since `SocietyPlane::lift` puts the other axis at zero — right for a
  ground marker, wrong for a capsule.

`Tuning::walk_speed` (default 1.4 m/s) is new. It belongs on `Tuning` rather than only on
`SocialMover` because a rate must exist in **both** feature sets, and `steering.rs` is not compiled
without its feature.

## What believable output looks like

Every need should spend most of its time above zero. Measured after tick 200, so the initial fall from
1.0 is excluded:

| need | at zero, before | after |
|---|---|---|
| social | 39.3% | **2.0%** |
| rest | 43.6% | **0.8%** |
| food | 71.2% | **12.8%** |
| fun | 89.2% | **35.3%** |

and the mean of the four needs stops being a flat line at zero and starts oscillating
(`evidence/walk.png`).

## Literature

No new model. The walking speed 1.4 m/s is an ordinary human walking speed and is documented as a
design default on the field, not attributed to a paper. The mechanism the fix restores — an activity
has a travel phase and a satisfaction phase, and only arrival starts the second — is the one
`docs/research/needs-and-utility.md` already describes.
