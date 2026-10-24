# Perf

## The change

None shipped.

## What it would have cost

One multiply per tie delta: `delta.affinity * (1 - tie.affinity.abs())` against `delta.affinity`. Unmeasurable
against a decision pass of millions of consideration evaluations — and irrelevant, because it was reverted on
behaviour rather than cost.
