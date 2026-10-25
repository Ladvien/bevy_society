# Review

## Pass 1 — Behavior: does the metric say something true?

**GREAT.** `became_rival` and `left_rival` count the transitions either side of `Rival`, and their difference is
exactly the standing ties: **272 − 188 = 84** directed, which is the 42 undirected pairs `rivals` reports. A
swapped-counter run gives **−84 against +84**, so the test's arithmetic catches the one way this metric could lie.

## Pass 2 — Zoom: read the ratio, not the count

**GREAT.** `evidence/reconciliation.png`, left: on all three seeds roughly a quarter of rivalries end, at ratios of
1.45 / 1.29 / 1.27. The count decelerates (second-half growth 0.68x the first), so it saturates.

Right: the only lever — `affinity_relax_per_day` — moves rivals and friends **together**, because relaxation pulls
all affinity toward zero. At 0.03 the village has ten rivalries and five friendships. That is not a fix; it is a
temperament change.

The thing worth saying plainly: the village is **not** broken. 15% rival pairs against 6.5% friend pairs is a
tense small village, and the mechanism that makes it tense also lets it recover. Before this iteration nobody could
have told you whether it recovered at all.

## Pass 3 — Fresh eyes: what should a host do with this?

**GREAT.** One sentence: *the village forms rivalries about 1.3x faster than it ends them, and if you want it
warmer you must lower both, because the crate forgets liking and disliking at the same rate.*

That last clause is the finding worth carrying forward: it names a **structural** limitation (one relaxation
variable serving two opposite affects) rather than a number to turn. Option 3 in the decision file —
separate forgetting rates — is the only shape that could fix the ratio without emptying the village, and it would
need a cited default rather than a reparameterisation.

## Verdict

Pass 1 **GREAT**. Pass 2 **GREAT**. Pass 3 **GREAT**.

## Not shipped, deliberately

- Any `Tuning` change. Every setting that reduces rivals also reduces friends, and picking one is choosing a
  village's temperament — recorded with three options in `needs-decision/`.
- A test asserting the ratio responds to `affinity_relax_per_day`. It was written, and it **passed at 0.0 and at
  0.02** — the fixture's dynamics do not depend on relaxation, whatever the village's do. Deleted rather than kept
  as false coverage, and replaced with the identity check, which is arithmetic and holds for any dynamics.
