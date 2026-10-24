# Plan: an accepted conversation is an appraisal

## Why this over the runner-up

The runner-up was `Doing`'s travelling semantics (49.8% of person-ticks) — a component a host reads, so
escalated. The appraisal balance was the log's #1, and measuring it turned a ratio into a structural fact:
**every refusal is an appraisal and no acceptance is**.

That is a definitions-file change, which `AGENTS.md` explicitly sanctions ("changing the starter pack is data,
not a schema change"), and it is the last piece of the mood chain that began three iterations ago.

## Design

`chat` and `gossip` gain mutual `Liking` on accept — `0.2` and `0.15` respectively, design defaults, modest
against `insult`'s `Anger 0.5` so the negativity bias survives.

`Liking` rather than `Joy` because OCC's joy is a prospect emotion about an event, while what a conversation
gives is liking for the person. `Liking` already exists and already feeds the pleasure axis at `+0.40`.

## Measured

Appraisals over 20 000 ticks, 24 people:

| | before | after |
|---|---|---|
| positive appraisals | **443** | **1 491** |
| negative appraisals | 1 587 | 1 587 |
| negative:positive | **3.58 : 1** | **1.06 : 1** |

Mood, `village-evening` at three seeds:

| seed | mean pleasure before | **after** |
|---|---|---|
| 7 | -0.1224 | **-0.0657** |
| 42 | -0.1268 | **-0.0652** |
| 99 | -0.1102 | **-0.0696** |

Roughly halved at every seed. Needs unchanged (0.603 → 0.607).

Steady-state emotions at the end of a 20 000-tick run, **before** the change: Anger 14, Distress 4, **Joy 0**
— positive affect absent. **After:** Anger 14, Distress 25, **Joy 22, Liking 14**, and max pleasure **+0.237**.

## Honest limit

Mean pleasure is still negative (-0.066). Distress still outnumbers Joy at the end, because refusals are
frequent and each one is exactly one distress. The **gain** asymmetry is intentional and stays; what changed
is that the positive channel now fires at all.

## Touched

- `assets/starter.society.ron` — `Liking` on `chat` and `gossip` accept.
- `tests/appraisal_balance.rs` (new).
- `docs/research/personality-and-mood.md`, `CHANGELOG.md`, `docs/loop/log.md`.

## New `Tuning` fields

None. The intensities are per-interaction data, as every other `on_accept` effect is.

## Tests

| Test | What it proves |
|---|---|
| `tests/appraisal_balance.rs::an_accepted_conversation_is_an_appraisal` | `Distress` equals the refusal count **exactly** (1 216 of 1 216), and positives are at least one per three acceptances — **443 of 1 338 before**, 1 491 of 1 338 after |
