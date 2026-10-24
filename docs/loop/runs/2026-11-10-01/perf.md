# Perf

No library code changed. The overlays are `Gizmos` systems in an example; the capture runs at 240
frames with 3 800 settle steps and takes about 16 s per clip.

| | |
|---|---|
| one clip (capture) | ~16 s |
| one clip (encode, on `big`) | ~9 s |
| six clips, end to end | ~3 min |

Capture cost is dominated by the settle steps and the per-frame GPU readback, not by the overlays:
a 10-frame capture costs the same 16 s as a 20-frame one, because the settle dominates.

Standing benchmark, unchanged:

| Persons | ms per society tick |
|---|---|
| 100 | 0.556 |
| 1 000 | 15.16 |
| 5 000 | 277.0 |
