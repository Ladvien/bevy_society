# Review

## Pass 1 — Behavior: does the change do what it claims?

**GREAT** for the scope shipped, and the scope is deliberately narrow.

| | claim | measured |
|---|---|---|
| the constant reaches the model | removing it improves mood | **-0.202 → -0.105** at identical accept/refuse counts |
| one path, not two | stacking is worse | pack 0.1 + crate 0.3 = **-0.207**; pack 0.1 replacing it = **-0.165** |
| the pack owns its refusal feeling | `chat`/`gossip` have `on_reject` | yes, `Distress 0.1` |

The middle row is the one that justifies the design: adding a definition-level emotion while still emitting
the crate's made mood **worse** than before the change. Two paths for one event.

## Pass 2 — Zoom: read the plot closely

**GREAT**, and the plot's job here is to convict the model, not celebrate the fix.
`evidence/mood.png`:

- **Left:** pleasure begins at exactly zero — the personality baseline — falls to about -0.15, and then
  oscillates there for 240 game-days. It never crosses the axis. Both traces (before and after) behave the
  same way, which is why the shipped change is honestly reported as -0.150 → -0.154 in the full village
  rather than claimed as a cure.
- **Middle:** 1 302 distress against 427 joy and 339 anger. A 3:1 ratio, not saturation.
- **Right:** at the end of the run **15 people hold Anger, 4 hold Distress, and nobody holds Joy.** That bar
  at zero is the whole finding: positive affect is not merely rarer, it is absent from the steady state.

No oscillation at a tick boundary, no clipping at ±1, no dead agents. The saturation *is* the subject and it
is on screen.

## Pass 3 — Fresh eyes: a different population

**GREAT.** Two populations were measured, and they disagree — which is the most useful thing here.

- **Eight people in a tight cluster:** refusals are about *half* the negativity (-0.202 → -0.105 without
  them), because with everyone in contact, interaction is most of what happens.
- **Twenty-four people in the real village:** refusals barely register (-0.150 → -0.154), because people
  spend their time walking and working, not talking.

A fix tuned on the first would have been mis-tuned for the second. Recorded in the decision file so the next
attempt starts from both numbers.

The narrative to a host:

> Being refused now costs exactly what your definitions say it costs, and if your definitions say nothing
> the crate applies one documented 0.3. Before, it always applied 0.3 on top of whatever you wrote, so a pack
> that made refusals mild was overruled.

## Verdict

Three GREAT. Shipped. The mood finding is logged as a decision, not silently retuned.
