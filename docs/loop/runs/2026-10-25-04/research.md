# Research: a tie is one-directional, and the decision is not

## The observation

`village-evening`, seed 7, 24 people, end of a 16.4 game-day run, all 276 unordered pairs:

| | |
|---|---|
| mutual friendships | **6** |
| one-way friendships | **12** |
| **share of directed friend ties that are unreciprocated** | **50%** |
| rivalries: mutual / one-way | 7 / 64 |
| mean `|affinity(A→B) − affinity(B→A)|` | **0.145** |

The crate models each person's tie separately, so asymmetry is *representable* — the research note says
so: "each person's tie is held separately, so asymmetric views are allowed but not explained." What is
missing is any mechanism that makes a *relationship* mutual, and the consequence is measurable: a
friendship is as likely to be one-sided as shared, and the two directions disagree by 0.145 on a scale
whose friend band is 0.05 wide.

## Why that matters to behaviour, not just to bookkeeping

The decision model reads **the actor's own tie** and the acceptance model reads **the target's**:

- `Signal::Affinity` is `decision.ties` — the actor's liking for the target (`src/behavior.rs`).
- The acceptance draw is `accept.base + accept.affinity · toward_actor.affinity + …`, where `toward_actor`
  is the **target's** tie toward the actor.

So `compliment`, whose only consideration is `Affinity Logistic(16, 0.25)`, is chosen when the actor likes
someone — and then the target, who is the one who decides, disagrees 0.145 of the time on average.
Measured refusal rate by the target's label toward the actor:

| interaction | refused by a **friend** | by an **acquaintance** | by a **rival** |
|---|---|---|---|
| `chat` | 22.0% | — | 100% |
| `gossip` | 25.0% | 48.4% | 66.4% |
| `compliment` | 23.1% | 45.8% | 100% |
| `insult` | 28.5% | 45.3% | 65.9% |

The draw discriminates powerfully — 23% against 100% — and the **overall 38.9% refusal rate is a
composition effect**: most approaches are aimed at acquaintances, at 45-48%. And an approach from a friend
is still refused a quarter of the time, which is the asymmetry showing through.

## What the literature says

- **Heider, F. (1946/1958), balance theory.** Read as summarised in **Schweighofer, S. & Schweitzer, F.**
  (JASSS), *Raising the Spectrum of Polarization* (DOI `10.18564/jasss.5323`, in the corpus):

  > "The central postulate of Balance Theory is that cognitive representations of attitude relations tend
  > toward balance. ... **In a dyadic relation, balance basically means reciprocity: Feeling positive about
  > a person who doesn't reciprocate this feeling is unbalanced.**"

  That is the exact defect, named. A one-sided friendship is not a neutral bookkeeping state; it is an
  unbalanced one that the literature says people act to resolve. The crate already cites Heider for the
  *triadic* case (`src/model/balance.rs`: "a friend of a friend is pulled toward friendship") and does
  nothing for the **dyad**, which is the simpler and more basic case.

- **Byrne, D. (1961).** Interpersonal attraction and attitude similarity (DOI `10.1037/h0044721`, not
  converted). Already cited in the crate for affinity-from-similarity. It is the mechanism that makes two
  people's opinions converge, and in a dyad that converges *symmetrically* — which is precisely why the
  measured 0.145 gap is surprising rather than expected.

- **Cartwright, D. & Harary, F. (1956).** Structural balance (DOI `10.1037/h0046049`, not converted), already
  cited. The formal treatment: a dyad is balanced when `sign(aff(A→B)) == sign(aff(B→A))`.

## What the papers do not supply

- **No rate** for how fast a dyad resolves toward reciprocity, and no threshold below which it is ignored.
  Heider's statement is qualitative. Any number here is a **design default**, chosen by measurement.
- No claim about *which* direction moves. Balance theory is symmetric — both are pulled — and the model
  must decide whether to move both, or only the less-committed one.

## What believable output looks like

- The share of **unreciprocated friend ties falls** substantially, and the mean directional gap narrows.
- The **refusal rate falls** as a consequence, because actors stop aiming expensive approaches at people who
  do not like them — and that is downstream evidence the change touched behaviour, not just numbers.
- Friendship and rivalry **counts do not explode**: a reciprocity step that pushes every tie to the mean
  would flatten the graph into indifference (every `affinity` the same) and destroy the labels.
- The change is **symmetric in expectation**: seed 42 and seed 11 show the same direction.
- Determinism holds; no draw, no iteration order.
