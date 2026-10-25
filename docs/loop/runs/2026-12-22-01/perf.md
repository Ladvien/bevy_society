# Perf

## The change

None shipped.

## Measurement noise, named

The scale curve is **still** unverified — sixth iteration. Load average sat between 6.5 and 11.8 for the whole
session, and three consecutive runs of the same binary at 1 000 people gave:

```
50.868 ms/tick
51.083 ms/tick
54.670 ms/tick
```

A 7% spread on identical code. `docs/guide/concepts.md` claims **13.5 ms** at 1 000. That is a 3.8x discrepancy
that no amount of averaging will resolve on this machine, and the honest position is that the guide's table is
**unconfirmed** rather than wrong — the November measurement may have been taken under different conditions.

It should not be quoted, and nobody should "fix" it by editing the number downward without a quiet machine.

## Cost of what was tried

Every lever was a constant change or a small arithmetic addition per decision; none was measurably slower. Since
none shipped, this is moot beyond the record.
