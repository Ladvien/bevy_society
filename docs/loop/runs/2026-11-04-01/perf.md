# Perf

Same seed, same scenario, release build, `--ticks 200`. The fix moves one message write from arrival
to decision, so the only expected cost is nothing; measured to confirm.

| Persons | before (ms/tick) | after (ms/tick) | delta |
|---|---|---|---|
| 100 | 0.549 | 0.549 | 0 |
| 1 000 | 12.47 | 12.47 | 0 |
| 5 000 | **418.3** | **419.7** | **+0.33%** |

The 5 000 figure is two separate processes on a shared machine, so ±0.3% is noise. No scenario got
slower.

`docs/guide/concepts.md` still quotes 351.6 ms at 5 000 from two iterations ago; the honest current
number is **418 ms**, and this iteration is not the one that changed it (the baseline at `HEAD` is
418.3). Left the guide alone rather than restating a number I have not investigated the cause of.
