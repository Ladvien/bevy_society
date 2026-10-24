# Research: friendship has no diminishing return

## Why this

Two separate mechanisms produced a clique in as many iterations — a `help` activity (rivals 44 → 0) and a
contact-triggered exchange (friends 22 → 267). Two failures from one cause is a stronger signal than either,
so this iteration measured the cause instead of trying a third mechanism.

## Measured: friendships grow without saturating

`village-evening`, seed 7, 24 people, 17 game-days:

| tick | accepted interactions | friend-ties | max affinity |
|---|---|---|---|
| 9 599 | 1 352 | **25** | **0.999** |
| 19 199 | 2 586 | 40 | 1.000 |
| 28 799 | 3 947 | 65 | 1.000 |
| 38 399 | 5 313 | 80 | 0.999 |
| 47 999 | 6 577 | 104 | 1.000 |
| 57 599 | 7 844 | 113 | 1.000 |
| 67 199 | 9 347 | **115** | 1.000 |

Linear growth, **no saturation**, and affinity pinned at the clamp rather than at an equilibrium.

## Three hypotheses, three sweeps, three negatives

| knob | friends |
|---|---|
| `mere_exposure_per_day` 0.05 (shipped) | **22** |
| 0.02 | 15 |
| 0.01 | 15 |
| 0.005 | 16 |
| `balance_rate_per_day` 0.5 (shipped) | **22** |
| 0.05 | 25 |

Nearly flat. Exposure and structural balance are second-order. Exposure's own equilibrium is
`0.05 / (0.05 + 0.01) = 0.833`, above the Friend threshold of `0.4`, so proximity is *sufficient* for
friendship — but it is not what decides how many friendships a village has. **My first reading was wrong, and
the sweep is what disproved it.**

## What it is

**The number of exchanges, with no diminishing return.** On a single pair: a `Friend` after **45**
interactions, and affinity at **0.73** after 57.

Familiarity from exposure saturates (`1 − f`). **Affinity from interaction does not** — each accepted exchange
adds its `TieDelta::affinity`, relaxation removes 0.01 per game-day, and nothing makes the *n*-th conversation
worth less than the first. So friendship is a monotone function of contact volume, and tripling contact triples
friends.

## Sources

- **Zajonc 1968**, DOI `10.1037/h0025848` — mere exposure. It supplies the *mechanism* (repeated exposure
  increases liking) and **not a rate**. The crate cited it as though it supplied
  `mere_exposure_per_day = 0.05`; the doc is corrected here.
- **Heider 1946** (DOI `10.1080/00223980.1946.9917275`) and **Cartwright & Harary 1956**
  (DOI `10.1037/h0046049`) — structural balance, measured here and found second-order.

No new paper. The finding is that one rate saturates and its neighbour does not, which no citation could
have supplied.
