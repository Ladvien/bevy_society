# Research: `Distance` reported the maximum for every place

## The gap

`Signal::Distance` is genuine for a `Person` target — the decision holds every neighbour's position and
returns `distance / perception_radius`. For a `Place` or `Point` it returned `None`, and `None` becomes
**1.0**:

```rust
there.map_or(1.0, |p| (decision.position.distance(p) / decision.perception_radius).min(1.0))
```

The comment said *"A place or point is somewhere in the world; without its position in the decision the
honest value is 'as far as it can see', not a confident zero."* That is honest about the missing data and
wrong about the consequence: `1.0` is the maximum, so a definition asking `Distance` on a place target
scored it as if the place were at the edge of perception — **always**.

## What that does to a definition

`Linear(-1, 1)` at input 1.0 is exactly **0.0**, and a zero consideration zeroes the whole option. So
adding the obvious line to a `Place` activity does not make it travel-aware; it **disables** it.

Measured on the starter pack's `eat`, which targets `Place("food")`:

| | `food` mean | at zero |
|---|---|---|
| without `Distance` | **0.402** | 0% |
| with `Distance Linear(-1, 1)` | **0.034** | **93%** |

A host writing the natural thing silently loses the activity, and nothing warns.

## The fix

`PlaceView` carries the place's plane position, `Decision` carries the list, and the signal finds the target:

```rust
DoingTarget::Place(place) => decision
    .place_positions
    .iter()
    .find(|(e, _)| *e == place)
    .map(|(_, p)| *p),
```

`Point` and `Itself` keep `None`: a `Point` is a `Wander` destination with no entry in the place list, and
`Itself` is the actor's own position, which is not what a distance consideration asks about.

## Measured

| | before the fix | after |
|---|---|---|
| both taverns in one run, 1 m vs 7 m, 6 000 decisions | **0 / 0** — the activity could never be chosen | **5 999 / 0** |
| `eat` with `Distance` in the village, `food` mean | 0.034, at zero 93% | **0.176**, at zero 0% |

Behaviour with the **current** pack is unchanged — all 67 199 series rows identical — because nothing in it
asks for `Distance` on a `Place`. This is a capability fix, not a behaviour change.

## What it does not fix

Adding `Distance` to `eat` still leaves `food` below its 0.402 baseline (**0.176**), because the pack's
curve is `Linear(-1.0, 1.0)`: a tavern 6.5 m away inside an 8 m perception radius scores **0.19**. The
signal is now correct; the *curve* is a tuning question, and a steep linear decay over the whole perception
radius is too harsh for a destination a person walks to once a day.

That is `docs/loop/needs-decision/travel-cost-is-not-scored.md`'s territory, and this iteration supplies the
missing capability rather than deciding the shape.

## Literature

No new model. `Signal::Distance` is an input of the utility decision (`docs/research/needs-and-utility.md`,
Game AI Pro 3 ch. 13 on response curves over normalised inputs). The crate had the signal in its vocabulary
and could not compute it for half the target kinds.
