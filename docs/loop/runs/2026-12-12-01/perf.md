# Perf

## The change

`classify` reads six fields instead of six literals. It is called once per tie per society tick, from
`refresh_labels`, in a loop that already walks `PersonIndex.sorted`.

Six field reads against a `Tuning` that is already `Res`-borrowed in that system: no new lookup, no new query.

## Behaviour

Bit-identical — 67 199 of 67 199 series rows. No scenario slowed, since the work is the same work.
