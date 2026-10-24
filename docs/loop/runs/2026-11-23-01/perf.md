# Perf

## The change

`PlaceView` carries one `Vec2`; `Decision` carries a `Vec<(Entity, Vec2)>` built once per decision from the
place list — four entries in the village. The signal does a **linear scan of four** to find the target.
Against a decision that scores every activity against every candidate, this is not measurable.

The `places` query gained `&Transform`; it already ran once per decision.

## Behaviour with the shipped pack: bit-identical

All **67 199** series rows of `village-evening` at seed 7 are identical before and after, because nothing in
the pack asks for `Distance` on a `Place`.

| | before | after |
|---|---|---|
| travelling | 58.9% | 58.9% |
| need mean | 0.545 | 0.545 |
| sleep | 15.0% | 15.0% |

No scenario slowed, and none changed.

## Standing benchmark

Not re-run: the change adds one `Vec2` per place and one scan of a four-element vector per decision, which
cannot move a benchmark whose cost is dominated by scoring. Recorded rather than measured, and honestly so.
