# Research: a full place is a price, not a prohibition

## Why this

Top item carried from `2026-10-25-09`, restated by every iteration since: **`Place::capacity` is used only
to *filter* candidates before scoring, never enforced at arrival**. A workshop that is full has **no
candidate at all**, so the activity cannot be chosen — not "chosen less often". Measured on
`village-evening` with the harness's fixed four places (30 seats): `work` falls from **10.3% to 3.6%** of
person-ticks as the population goes 24 → 48, and rival density goes 7.8% → 20.1%. Scaling the seats
instead gives **0.00%** hostile, which is the other failure. The target is a density that holds roughly
flat, and neither extreme produces it.

## The literature

### 1. Congestion is a cost on a path, and the cost is exponential in the number of users

**Talamali, Bose, Haire, Xu, Marshall & Reina 2020**, *Sophisticated collective foraging with minimalist
agents: a swarm robotics test*, *Swarm Intelligence* **14**(1):25-56, DOI `10.1007/s11721-019-00176-9`.
**Verified against the publisher PDF** (SHURA open copy,
`https://shura.shu.ac.uk/25216/8/Talamali2019_Article_SophisticatedCollectiveForagin.pdf`) because the
home-still MCP returned 404 for every tool this session and the paper is **not in the corpus snapshot**
(`docs/research/corpus/*.tsv` has no row for this DOI). Equations 12, 13 and 14 below are quoted from the
full text; the earlier draft of this note said pages 25-55 of volume 13 and "in the corpus, converted",
and both were wrong.

The paper derives an optimal-foraging model that **accounts for overcrowding** and measures the model
against up to 200 physical robots. Three results matter here.

1. **Congestion is a travel-time penalty, not an exclusion.** Eq. (12) verbatim:

   $$T_{C,j}(\rho_j S) = T_{0,j}\left(\exp\left[\kappa_j \frac{\rho_j S}{N_{\text{crit},j}}\right] - 1\right)$$

   > "where $T_{0,j}$ is a constant which sets the time scale of the additional travel time, $\kappa_j$ is
   > a constant included to fine-tune the nonlinear effect of overcrowding on the path to $A_j$, and
   > $N_{\text{crit},j}$ is the critical number for which traffic congestion may have a significant
   > effect. This means that $T_{C,j}(\rho_j S)$ is **negligible if $\kappa_j \rho_j S \ll N_{\text{crit},j}$**."

   Eq. (13) fixes the critical number as "how many bodies fit": $N_{\text{crit},j} = \max(\lfloor d_j /
   Z_{KB} \rfloor, 1)$, where $Z_{KB} = 3.3$ cm is a Kilobot's diameter. The paper's fitted values
   (Appendix A, Table 2): $\kappa = 2.321$, $T_0 = 0.029$ s, $\beta = 0.965$.

2. **A crowded patch is still used.** Fig. 3's own caption: *"There is a linear growth for small-to-medium
   numbers of robots on a path, and a **nonlinear decay for medium-to-large numbers of robots** on a
   path."* Decay, not zero. And the optimum is an **interior split**: §5.2.2, with two nearby sources of
   equal quality, *"the best strategy is to avoid traffic congestions on the trail leading to the
   higher-quality items … reducing collisions and congestion on the higher-quality source path means
   allocating more workers to the lower-quality source."*

3. **The yield is the quantity the agent is maximising.** Eq. (14):
   $R = \sum_j q_j \beta_j \rho_j S / \tilde d_j^2$ with $\tilde d_j = d_j + v_0 T_{C,j}(\rho_j S)$ — a
   patch's attractiveness is divided by its **congestion-inflated distance**, i.e. a multiplier that
   starts at 1 and falls with occupancy and never reaches zero. That is exactly the shape a decision
   score needs.

### 2. The social-psychology side: a crowd makes each member's effort worth less

**Latané, Williams & Harkins 1979**, *Many hands make light the work: The causes and consequences of
social loafing*, *Journal of Personality and Social Psychology* 37(6):822-832, DOI
`10.1037/0022-3514.37.6.822`. The finding is that per-person effort in a collective task falls as the
number of contributors rises, and the review's own summary of the mechanism is a **power function of the
number of others** (the same law the crate already cites for witness dilution, Latané 1981,
DOI `10.1037/0003-066X.36.4.343`). This is the human-labour counterpart of the traffic term: the *N*-th
worker at a bench adds less than the (N−1)-th.

The crate's `Tuning::witness_dilution` already uses this law in `src/reputation.rs` with exponent 0.5. A
crowding penalty on a *place* is the same shape applied to a different variable, and it should cite the
same source rather than invent a second one.

## What the crate does now, and why it is wrong

`src/behavior.rs::targets_for`:

```rust
ActivityTarget::Place(tag) => places
    .iter()
    .filter(|place| place.used < place.capacity && ...)
```

A full place is removed before `score_option` ever sees it. Consequences, measured:

- **The activity disappears.** `work` is 10.3% of a life at 24 people and 3.6% at 48, because the
  workshop (6 seats) is permanently full. A person does not "work less"; it cannot work.
- **Two meanings of full.** `ActivityTarget::Home` bypasses the gate entirely, so a home with 6 declared
  seats takes 16 residents; the crate warns at startup and nothing else.
- **The penalty is all-or-nothing.** `used = capacity - 1` and `used = capacity` score identically to
  `used = 0` and `used = ∞`. There is no gradient, so the decision cannot prefer an emptier place. The
  paper's own optimum requires that gradient.

## The shape to implement

Score a `Place` target by a **crowding multiplier**, and stop filtering:

$$s_{\text{crowd}} = \exp\left[-\kappa \frac{\max(0,\ \text{used} - \text{capacity}) + 1}{\text{capacity}}\right]$$

or, more simply and closer to the paper's exponential-in-users form with the capacity as
$N_{\text{crit}}$:

$$s_{\text{crowd}} = \exp\left[-\kappa \cdot \max\left(0, \frac{\text{used} + 1}{\text{capacity}} - 1\right)\right]$$

which is **1.0 while there is room**, and falls smoothly past capacity. Two properties the design needs:

- **Never zero**: `exp` of a finite argument. A crowded place stays reachable, at a price — the paper's
  "nonlinear decay", not a cliff.
- **Gradient across places**: two workshops, one at 3/6 and one at 6/6, score differently, so the decision
  sends the next worker to the emptier one. That is the paper's interior optimum appearing as an
  emergent allocation.

Parameters the paper reports, and what transfers:

| paper | value | transferable? |
|---|---|---|
| $T_{0}$ | 0.029 s | **No** — a Kilobot's collision latency |
| $\kappa$ | 2.321 | **The shape only.** The paper fits it to Kilobot collisions; the crate has no collision-duration model to fit. Design default, stated as such. |
| $N_{\text{crit}}$ | $\lfloor d / Z_{KB}\rfloor$ | **Yes, in spirit** — this is "how many bodies fit", which is what `Place::capacity` already declares. Use `capacity` as $N_{\text{crit}}$. |

So the crate adopts the **form** (exponential in occupancy ÷ seats) and one **structural** reading
(capacity is the critical number), and states plainly that $\kappa$ is a design default because the
paper's value is calibrated to robot collisions the crate does not simulate.

## What believable output looks like

- `work`'s share of person-ticks **holds roughly flat** (10.3% at 24, target ≥8% at 48) instead of
  collapsing to 3.6%.
- Rival density **stays in the 7-16% band** at 24, 36 and 48 rather than going 7.8 / 23.1 / 20.1.
- No activity **vanishes**: the `doing` column in `series.csv` keeps `work` and `craft` non-zero at every
  population.
- A **capacity-0 place is still a prohibition** — it can hold nobody, and `exp` of a large negative number
  is effectively zero but not exactly. Keep an explicit `capacity == 0` exclusion so a host can still say
  "this place takes nobody".

## Sources

- Talamali, Bose, Haire, Xu, Marshall & Reina **2020**, *Swarm Intelligence* **14**(1):25-56, DOI
  `10.1007/s11721-019-00176-9` — Eq. (12) congestion term, Eq. (13) critical number, Eq. (14) the
  congestion-inflated distance, §5.2.2 the interior optimum, Fig. 3 caption the "linear growth …
  nonlinear decay" collection curve, Table 2 the fitted κ / T₀ / β. Read from the publisher PDF at
  `https://shura.shu.ac.uk/25216/8/Talamali2019_Article_SophisticatedCollectiveForagin.pdf`; **not in the
  home-still corpus**.
- Latané, Williams & Harkins 1979, DOI `10.1037/0022-3514.37.6.822` — per-person effort falls with the
  number of contributors; the mechanism is Latané 1981's power law, already cited in
  `docs/research/rumor-and-reputation.md`.
- Baumeister et al. 2001, DOI `10.1037/1089-2680.5.4.323` — not used here; noted only because the
  previous iteration's open question (`good-is-outweighed-by-bad.md`) is adjacent and must not be
  conflated with this one.
