# The 5:1 pack change turns two trajectory-pinned tests red

Run `2026-10-27-07` changed the starter pack's `snub` grudge curve to `Power(1.15)` (compliments : insults 2.9 → 5–7 : 1, friends outnumber rivals). Two tests pin numbers of the old trajectory, and a pass may not edit tests:

1. `tests/label_hysteresis.rs::the_band_stops_the_low_end_flapping_and_moves_no_relationship` (line ~303): demotions with a band / without = 18 / 273 = 6.6%, bar is ≤ 5%. The second bar in the same test (≥ 90% removed) still passes at 93%, and the earlier history in the test comment shows the same fixture moving 1 → 6 → 18 as the village's day changed.
2. `tests/rumor_book.rs::the_book_accumulates_rumors_nobody_can_tell` (line 97): "most of the book should be untellable": 294 of 892 can still be told (33%), so the majority claim fails.

Options: (a) a human evaluator pass relaxes both bars to the property they state (≥ 90% churn removed; a stated untellable share) and merges the PR; (b) drop the pack change and keep the 2.9:1 village; (c) pin both fixtures to a custom pack so they stop depending on the shipped one. Recommend (c) for label_hysteresis/rumor_book if a pinned pack is cheap, otherwise (a). The PR is open and **not merged**.
