# Review

## Pass 1 — Behavior: does it fix what the narrative showed?

**GREAT.** The defect was named from the narrative before any code: `snub, compliment, snub, compliment`
every six minutes, `snub` at **170 of 338** starts, a run of **ten** in a row — from an activity that
satisfies no need.

| | seed 11 | seed 42 |
|---|---|---|
| `snub` share, sum | **50.3%** | **48.4%** |
| `snub` share, max | **26.0%** | **32.6%** |
| longest run | 10 → **6** | 11 → **8** |
| top activity | `snub` → **`compliment`** | `snub` → **`gossip`** |

`evidence/after/cluster-narrative.txt` reads as a day: `work → wander → chat → snub → compliment → wander →
play → compliment → wander → compliment → snub → gossip`. The ten-in-a-row is gone at both seeds.

The refused-insult fix is proven separately and mechanically: **0 accepted, 1 refused, memory 0.700 →
0.300** (`tests/snub.rs`, written red first).

## Pass 2 — Zoom: read the plot closely

**GREAT.** `evidence/grudge.png`:

- Left panel: the snub share over 80 game-days. The green (max) trace sits **below** the red (sum) trace for
  its whole length, with the *same* shape — the daily cycle is unchanged, the level is not. No new
  oscillation, no clipping at 0 or 100, no ticks where the share jumps to a boundary.
- Middle: friends 4→6, rivals 9→5, reach 0.167→0.333 at seed 11.
- Right: the village's four needs with the two lines lying **on top of each other** — social 0.75/0.74, rest
  0.40, food 0.40, fun 0.63. The change did not touch the scenario it was not meant to.

The middle panel is the one to read sceptically, and Pass 3 is why.

## Pass 3 — Fresh eyes: a different seed

**GREAT, and it cost me a claim.** At seed 42 the friends:rivals and reach panels do **not** reproduce:
reach is **0.167 both before and after**, friends 5 → 6. I had the 0.167 → 0.333 doubling drafted as the
headline; the second seed says it is seed-specific. The snub reduction does reproduce (48.4% → 32.6%) and is
the only thing claimed.

The narrative to a host, one sentence:

> A person's grudge toward someone is the worst thing that person did to them, not a running total of every
> small slight — so a village of strangers stops spending half its evening insulting each other, and a
> refused insult no longer counts as an injury.

## Verdict

Three GREAT. Shipped.
