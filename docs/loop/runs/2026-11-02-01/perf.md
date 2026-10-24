# Perf

Not measured: the harness is test-only code (`tests/common/capture.rs`), and the library change is one
boolean per activity in `finish`, once at startup.

The numbers from two iterations ago stand: 14.1 ms per society tick at 1 000 persons, 0.55 ms at 100,
351.6 ms at 5 000 (`docs/guide/concepts.md`).
