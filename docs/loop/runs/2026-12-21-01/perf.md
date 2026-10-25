# Perf

## The change

One `Startup` system, run once, comparing every pair of `Place` entities. At `P` places that is `P²` distance
computations, once, at startup — with the project's own four places, sixteen.

## Cost

Not measurable in tick cost: it runs at `Startup`, not in `FixedUpdate`.

A village with 1 000 places would do 1 000 000 distance computations once. That is still a startup cost, not a
per-tick one, and a world with a thousand places has larger problems — but it is worth naming rather than
leaving for someone to discover.

## Scale curve, still unverified

Fifth iteration without a trustworthy number: the machine's load ranged 1.8 to 10.2, and consecutive runs of the
same binary at 1 000 people gave 36.8 / 58.1 / 46.2 ms. The guide's `0.6 / 13.5 / 245.1 ms` table remains
unconfirmed. It should not be quoted until somebody measures it on an idle machine.
