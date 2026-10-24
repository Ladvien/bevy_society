# Research: why a person did that

## The gap

`Doing` says *what* a person is doing. Nothing said *what else it was weighing, or by how much*. A
host watching Bram snub Corin at 15:22 had no way to learn that `relax` and `wander` had scored higher,
and that the weighted draw picked the lowest scorer. The crate could show behaviour but not explain
it, and the loop's own brief puts "explainable" beside believable and deterministic.

Measured, from the village at seed 7, tick 4147 — the first trace this crate has ever produced:

```
 1.000  snub
 1.000  relax
 1.000  wander
 0.971  eat
 0.912  chat
 0.752  compliment   <- chosen
```

Six options weighed, the lowest chosen, and every number invisible before.

## What the source says

Game AI Pro 3 ch. 6, "Debugging AI with Instant In-Game Scrubbing" (corpus doc
`gameaipro3-ch06-debugging-ai-instant-game-scrubbing`). Three constraints, quoted, and the AI-specific
use:

| The chapter | What it demands here |
|---|---|
| "The recording of object data must have minimal to no impact on the speed of the game simulation" | the trace is opt-in and measured at **15.16 ms/tick with it, 15.28 without** at 1 000 people |
| "the memory requirements of the system must be **fixed**" | a ring of `capacity` decisions, oldest dropped; `with_capacity(2, 8)` holds exactly 2 however long the run |
| "the recording of object data must have **no impact on the outcome of simulating objects**" | asserted: the same seed and spawn produce a **byte-identical sequence of choices** with and without the trace |
| "recording an AI's entire behavior tree […] displayed which branches were currently executing as well as where within other possible decision branches, execution terminated prematurely" | each decision records every option that scored, with its score, and which one won |

The chapter's own recorder drops the **oldest** frame when its fixed buffer is full, which is the same
policy as `DecisionTrace`'s ring — a host that looks at "the last few minutes" is served by that, and
memory stays flat over a run that lasts hours.

## The formula

`DecisionTrace` stores, per decision: the tick, the person (and its name when one is known), and up to
`options_per_decision` scored options, best first.

The crate's selection is **Dill's dual-utility weighted draw**: options are cut to the best rank tier,
then to `tier_cutoff × best`, then one is drawn with probability proportional to `weight · score`. So
the chosen option is *often not the best one*, and saying how close a call it was must compare the
chosen score with the best score, not with the next one down:

$$\text{shortfall} = \frac{s_{\text{best}} - s_{\text{chosen}}}{s_{\text{best}}} \in [0, 1]$$

`0` means the draw took the best option; `0.25` means it took one a quarter worse. My first version
compared the chosen option with the *runner-up in the sorted list*, which produced a **negative**
"margin" the moment a weighted draw legitimately picked the fifth-best option — caught by running it
on the village, not by reasoning about it.

## What believable output looks like

A host prints one line and a reader understands the decision:

```
tick 4147: someone chose compliment (0.752), 25% below snub (1.000)
```

"someone" because that entity had despawned by the end of the run — which is exactly why the trace
copies the name when it can, and why `describe` accepts a name list from the caller.

## Design defaults, declared

No paper gives these two numbers; the module says so:

- `capacity: 64` decisions — a few minutes of a village's life.
- `options_per_decision: 8` — more than the starter pack can field.
