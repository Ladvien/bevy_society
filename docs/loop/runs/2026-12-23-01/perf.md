# Perf

## The change

One `Startup` system addition: build a `BTreeMap` of residents per home and compare each to the seats its arrival
disc holds. At `P` places and `N` people that is `O(N + P)`, once.

## Cost

Not measurable in tick cost; it runs at `Startup`.

## Scale curve, still unverified

Seventh iteration. The machine's load sat between 6.5 and 11.8, and three consecutive runs of the same binary at
1 000 people gave 50.868 / 51.083 / 54.670 ms. `docs/guide/concepts.md` claims 13.5 ms.

That 3.8x gap cannot be resolved by averaging on a loaded machine. The guide's table is **unconfirmed**, not
proven wrong — the November measurement may simply have had the machine to itself. It should not be quoted until
someone measures it quietly.
