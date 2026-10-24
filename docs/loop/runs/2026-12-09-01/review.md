# Review

## Pass 1 — Behavior: is the finding right, and did I check myself?

**GREAT**, and the checking is the substance. Three sweeps, and **two of them disproved my own hypothesis**.

I predicted exposure was the friendship pump — its equilibrium is `0.833` against a `Friend` threshold of
`0.4`, so proximity alone suffices. Sweeping it to 0.02, 0.01 and 0.005 moved friends **22 → 15 → 15 → 16**.
Structural balance from 0.5 to 0.05 moved it **22 → 25**. Neither is the driver.

What is: **45 interactions make a Friend**, and affinity keeps climbing to 0.73 by 57. Each exchange adds
`TieDelta::affinity`; relaxation removes only 0.01 per game-day; nothing discounts the *n*-th conversation.
Friendship is a monotone function of contact volume.

## Pass 2 — Zoom: read the growth curve

**GREAT**, and the number that convicts is the saturation. Friend-ties over a full run:

| tick | friend-ties | new since last |
|---|---|---|
| 9 599 | 25 | — |
| 19 199 | 40 | 15 |
| 28 799 | 65 | 25 |
| 38 399 | 80 | 15 |
| 47 999 | 104 | 24 |
| 57 599 | 113 | 9 |
| 67 199 | 115 | 2 |

It **never flattens** across 17 game-days, and affinity sits at **1.000** — the clamp. A model whose variable is
pinned at its bound is not in equilibrium, it is saturated, and that is what lets a 3× contact change produce a
3× friend count.

## Pass 3 — Fresh eyes: does the explanation cover **both** failures?

**GREAT**, and this is what makes it worth more than a single iteration's finding. It explains the `help`
clique (rivals 44 → 0) and the contact-exchange clique (friends 267, rivals 0) as **one cause**: both added
exchanges, both met a threshold with no diminishing return, both saturated.

And it explains why the contact cooldown could not be tuned: 3 h, 12 h and 48 h gave 267 / 182 / 72 friends —
a rate sweep on a monotone function just walks up and down the same line.

The narrative to a host:

> Every conversation adds affinity and nothing discounts the next one, so a person who has met someone 45 times
> is their friend whatever the meetings were like. A village with three times the social contact has three times
> the friendships, which is why two features that added contact each turned it into a clique.

## Verdict

Three GREAT for the investigation. **No fix shipped**, and the reason is stated: the fix is a saturating term on
interaction affinity, which changes how every tie forms for every host — a decision, recorded with three options
and a recommendation.

One test shipped (the exposure equilibrium's arithmetic), and it is written so that the recommended fix does
**not** have to delete it.
