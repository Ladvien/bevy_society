# Needs and utility

## What bevy_society does

Each person has needs, scalars in `[0, 1]` that the definition file names and that scripts can move (`fx.need` clamps to `[0, 1]`). A person chooses an activity by scoring the activities on offer. Each activity has considerations: an input signal read from the person, the world or a tie, passed through a response curve to give a factor in `[0, 1]`. The factors combine into one score. The chosen activity is the highest-scoring one, sampled with the crate's deterministic RNG. **Status:** the response curves are on HEAD (`src/model/curves.rs`) and the decision system (`src/behavior.rs`: `decide`, `arrive`, the exchange) is on HEAD, with the combination rule and compensation step as described here. This note describes them as specified, and it says so wherever it applies.

## Equations as implemented

- Response curves (`src/model/curves.rs`, on HEAD), input `x` clamped to `[0, 1]`:
  - `Linear(m, b)`: `clamp(m·x + b, 0, 1)`.
  - `Power(k)`: `x^k`.
  - `Logistic(k, x0)`: `1 / (1 + e^(−k(x − x0)))`, which is 0.5 at `x = x0`.
  - `Step(at)`: 1 when `x ≥ at`, else 0.
  - `Hours(from, to)`: 1 inside the hour window `[from, to)`, wrapping past midnight when `from > to`. Input is `h / 24`.
- Compensation (specified for D7, not on HEAD): `s′ = s + (1 − s) · (1 − 1/n) · s`, where `s` is the combined score and `n` the number of considerations. Design choice: Game AI Pro 3 ch. 13 multiplies raw scores; compensation keeps many-consideration activities comparable (see the decision below).
- Social proof: the share of close ties whose current activity matches, or 0 with no close ties.
- `Field(input)`: the stigmergy grid's value at the actor's own cell, divided by that channel's peak, so it is in `[0, 1]`. `OwnFaction` and `OtherFactions` are relative to the actor's memberships. `Gradient(input, toward)` is a target, not a signal: it walks 4 cells along the slope, and yields no target on a flat field. Both read the **previous** tick's grid (`deposit_fields` runs after `Decide`), which is defensible for a person choosing from what it can see, and on the first tick reads nothing.
- `Distance`: for a `Person` target, `distance / perception_radius`, clamped to 1. For a **`Place`** it is the same quantity from that place's position — added 2026-11-23, because until then a `Place` reported `1.0` (the maximum) and any definition asking for `Distance` on one scored it at 0 under `Linear(-1, 1)`, silently disabling the activity. `Point` (a `Wander` destination, not in the place list) and `Itself` (the actor's own position) still report the maximum, and the source comment says why each is genuinely unknown. Measured: `eat` in the starter pack, `food` mean **0.402 → 0.034** with the broken input and **0.176** with it fixed. The **curve** is the open question, not the input: `Linear(-1.0, 1.0)` scores a tavern 6.5 m away inside an 8 m radius at **0.19**, too harsh for a daily destination (see `docs/loop/needs-decision/travel-cost-is-not-scored.md`).

- **What cycles, and why.** `food` oscillates daily at the starter pack's rates because `eat` supplies `1.2 x 0.75 h = 0.90` against an exactly equal `0.9/day` decay; `rest` does **not** and cannot, because `sleep` supplies `0.15 x 8 h = 1.20` against `0.70/day` — a net **+0.50 per sleep**, so it climbs until sleeping stops being chosen. A need whose supply exceeds its decay settles high and is correct to do so; the property worth checking is that every need is *met and re-met*, not that a particular one starves. See `docs/loop/runs/2026-12-06-01/`.

## Why these defaults

- **Response curves.** Dill (2015) describes utility considerations as response curves over normalised inputs, and Game AI Pro 3 ch. 13 states that the considerations are "multiplied together to obtain the overall score of the DSE" (`gameaipro3-ch13-choosing-effective-utility-based-considerations`, converted; passage checked). The curve shapes here follow that description.
- **Utility-based choice.** "The advantage of utility-based AI is that it is better able to account for the subtle nuance of the situation" (`gameaipro2-ch03-dual-utility-reasoning`, converted; passage checked). "Utility theory is a concept that's been around long before games or even computers" (`gameaipro1-ch09-introduction-utility-theory`, converted; passage checked). Both support scoring over a continuous range rather than picking by rule.
- **Multiplicative combination.** Game AI Pro 3 ch. 13 multiplies considerations, which is what compensation corrects. The compensation formula itself is not in the converted text; it comes from the decision recorded below, not from a quoted source.

## Decision: Mark & Dill compensation is a design choice

Game AI Pro 3 ch. 13 multiplies raw consideration scores. Because a product of many factors falls as the count grows, an activity with many considerations scores lower than one with few, even when each factor is good. Compensation (`s′ = s + (1 − s)(1 − 1/n)s`) restores comparability by lifting a score toward 1 in proportion to the number of considerations. The crate keeps compensation on purpose. The code comment must read: "design choice: Game AI Pro 3 ch. 13 multiplies raw scores; compensation keeps many-consideration activities comparable." This is a departure from the source, recorded here and in `src/behavior.rs` when D7 lands.

## Decision: a need's source must meet its decay

Every need decays per game-day; every activity supplies `satisfies · duration_hours` per use. The starter
pack's budget:

| need | decay/day | best source | per use | uses/day |
|---|---|---|---|---|
| social | 1.00 | `chat` | 0.36 | 2.78 |
| rest | 0.70 | `sleep` | 1.20 | 0.58 |
| food | 0.90 | `eat` | 0.90 | 1.00 |
| fun | 0.80 | `relax` | 0.60 | 1.33 |

`fun` needs 1.33 uses a day and a 24-person village supplies 0.10, so it sits at zero for a third of a
run. Raising `relax` to 0.55 per game-hour (0.825 per use, 0.97 uses a day) halves `fun`'s zero fraction
— 33.8% → 18.8% — but **lowers** the mean of all four needs, 0.127 → 0.118, because the village is
over-subscribed and one supply only crowds out another. Reverted.

`BehaviorPlugin::finish` warns when a need's best source needs more than four uses a day, the point past
which no activity shape is plausible. The starter pack now also carries `play` — `Person`-targeted,
0.5 h, 0.85 per use — so `fun` needs 0.94 uses a day rather than 1.33, and in a 24-person village it
sits at zero for 1.3% of ticks rather than 33.8%. `SocietyDefs::need_decay` and `best_source` expose the same
arithmetic to a host. The open question — what should a village's evening be — is
`docs/loop/needs-decision/over-subscribed-evening.md`.

Vansteenkiste & Ryan 2013 (DOI 10.1037/a0032359) is why this is worth naming rather than leaving to a
plot: self-determination theory distinguishes **need satisfaction** from **need frustration**, and
frustration is not merely a low value but a different state with its own behavioural consequences.

## Decision: a place at capacity is a price, not a prohibition

An activity may target `Place("tag")` or a person's own `Bound(name)`. Both used to drop a place from the
candidate list **before scoring** (`used < capacity`), so a full place gave the activity *no candidate at
all*: not chosen less often, but unavailable. Measured on `village-evening`, whose four places seat 30:
`work` fell **10.3% → 3.6%** of person-ticks as the population went 24 → 48, because the workshop
(6 seats) was permanently full.

The score now carries a **crowding multiplier** (`src/model/crowding.rs`, engine-free):

$$s_{\text{crowd}} = \exp\left[-\kappa \cdot \max\left(0, \frac{u + 1}{c} - 1\right)\right]$$

with $u$ people already targeting the place, $c$ its declared `capacity`, and $\kappa$
`Tuning::place_crowding_penalty` (default 1.0). It is **exactly 1.0** up to and including the last free
seat, so a village with room is unchanged, and it falls past capacity without reaching zero. $u + 1$
because the person deciding is asking for a seat it does not yet hold. `capacity: 0` is an exact `0.0` —
the one case that really is a prohibition.

**The shape is Talamali et al. (2020)**, *Swarm Intelligence* 14(1):25-56, DOI
`10.1007/s11721-019-00176-9`. Eq. (12) models congestion on a foraged path as
$T_C = T_{0}(\exp[\kappa \rho S / N_{\text{crit}}] - 1)$, "negligible if $\kappa \rho S \ll N_{\text{crit}}$"
and growing exponentially past it; Eq. (13) makes $N_{\text{crit}}$ "how many bodies fit"; and the paper
measures (Fig. 3 caption) "a linear growth for small-to-medium numbers of robots on a path, and a
**nonlinear decay for medium-to-large numbers**", with an **interior split** as the optimum when sources
are close (§5.2.2). The crate takes the form and the structural reading — `capacity` *is* $N_{\text{crit}}$
— and not the fitted $\kappa = 2.321$, which is calibrated to Kilobot collisions this crate does not
simulate; the rate is therefore a **design default**, chosen by measurement, and its doc says so.

`ActivityTarget::Home` is deliberately **not** priced. A home is where its residents sleep, it was never
gated, and a home that cannot seat them is a layout problem `warn_unreachable_places` already names at
startup. Pricing it would suppress `sleep` in exactly the over-full homes the warning is about, which is a
different change (`docs/loop/needs-decision/a-village-with-no-room.md`).

## What the model leaves out

- Dual-utility selection and an activity-level weight beyond the compensation step.
- Learning or adapting considerations over time; the curves are fixed per definition.
- Physiological need dynamics: needs move only through definitions, scripts and events, not through a metabolic model.
- Planning across several activities; each choice is local to the current tick.
- The Game AI Pro "infinite axis" utility system (Mark 2013) beyond the curve primitives above.

## Open contradictions and departures

- **Departure, documented:** compensation is kept although the source multiplies raw scores. Design choice (above).
- **Departure, documented:** recency in any retrieval-linked score is measured from creation (see `memory.md`).
- **Not yet on HEAD:** the combination rule and compensation are specified but unimplemented until D7 lands. This note will be checked against the code after that merge.

## Sources

- Dill, K. (2015). Choosing Effective Utility-Based Considerations. Game AI Pro 3, ch. 13. https://doi.org/10.1201/b18373-5 · home-still `gameaipro3-ch13-choosing-effective-utility-based-considerations` (converted)
- Game AI Pro 2, ch. 3, Dual Utility Reasoning · `gameaipro2-ch03-dual-utility-reasoning` (converted)
- Game AI Pro, ch. 9, Introduction to Utility Theory · `gameaipro1-ch09-introduction-utility-theory` (converted)
- Mark, D. (2013). Infinite Axis Utility System. Cited in Game AI Pro 3 ch. 13. Not converted.
- Talamali, M. S., Bose, T., Haire, M., Xu, X., Marshall, J. A. R. & Reina, A. (2020). *Sophisticated
  collective foraging with minimalist agents: a swarm robotics test.* *Swarm Intelligence* 14(1):25-56.
  https://doi.org/10.1007/s11721-019-00176-9 — Eq. (12) congestion term, Eq. (13) critical number, Eq. (14)
  congestion-inflated distance, §5.2.2 the interior split, Fig. 3 caption the growth-then-decay collection
  curve, Table 2 the fitted κ = 2.321 / T₀ = 0.029 s. **Not in the corpus** (checked against
  `docs/research/corpus/*.tsv`); read from the SHURA open copy at
  https://shura.shu.ac.uk/25216/8/Talamali2019_Article_SophisticatedCollectiveForagin.pdf, equations and
  quoted phrases verified in the full text.
