# Plan: build the contact exchange, measure it, and revert

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics — a host-facing component, escalated. This iteration took the
root cause established last time (arrival-driven exchange) and built the fix it implied.

## Design, as built

`talk_in_contact` in the **`Interact`** set — not `Relate`, which runs *after* `Interact` and would serve a
request a tick late. It walks `Perception.contact`, one direction per pair by `PersonIndex` slot order, and
queues `Interact { interaction: "chat" }` when `Ties::last_spoke_day` is older than
`contact_talk_cooldown_hours`.

`Tie::last_spoke_day` is a **new field** because `last_contact_day` is stamped every tick two people are near
and cannot serve as a cooldown.

## Measured

| | before | after |
|---|---|---|
| co-located pair, 20 000 ticks | **30** | **89** |
| cooldown behaviour | — | 1 920-tick first gap, then held |
| village friends | **22** | **267** |
| village rivals | **33** | **0** |

At 12 h: 182 friends, 3 rivals. At 48 h: 72 friends, 12 rivals.

## Why it was reverted

267 friend pairs in a village of 24 is 97% of all possible pairs. Even 48 hours triples the baseline. The
contact pass adds 13 742 requests against ~7 500 baseline interactions — it nearly triples all social contact —
because in a village everyone is always in contact.

The rate cannot be tuned to work: high enough to bridge a 40 m gap is high enough to make everyone friends.

## Touched

- **Reverted**: `src/core.rs`, `src/relationships.rs`.
- `docs/loop/needs-decision/how-can-two-groups-ever-meet.md` — the measurement, the four-cooldown sweep, and
  three options.
- `CHANGELOG.md` (a `Changed` entry recording the finding, not a behaviour change), `docs/loop/log.md`.

## New `Tuning` fields

None shipped.
