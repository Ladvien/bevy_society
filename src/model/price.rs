//! A price read off how full the village's stores are.
//!
//! The crate used to price a good by integrating its **net stock change**:
//! `Δprice = price_speed_per_day · (produced − consumed)`. Two defects were recorded against it, both
//! measured, and the second is why this module exists:
//!
//! 1. `consumed` is what [`Stock::take`](crate::Stock) actually *removed*, so `produced − consumed` is the
//!    net change in what villagers hold. A village that bakes more than it eats therefore saw its price
//!    **rise**, and a real shortage was invisible, because an eater with an empty shelf takes nothing and
//!    adds nothing.
//! 2. It counted **units, not heads**. The same six loaves are a glut in a village of three and a famine in
//!    one of thirty, and a unit count cannot tell them apart.
//!
//! The corpus's own game-economy model prices from the **storage ratio** instead. In its Model A
//! ("Production-Consumption"), each agent's price is a fixed function of how full its storage is
//! (*Gnome Trader*, corpus doc `eth-cgl-ar_games-Zun16a`, §3.1.1):
//!
//! $$p = w \cdot v(r) + k, \qquad r = \frac{S_{\text{cur}}}{S_{\text{max}}}, \qquad
//! v(r) = \begin{cases} -2(r - 1), & r < 0.5 \\ 1.5 - r, & \text{otherwise} \end{cases}$$
//!
//! and the paper states the reading plainly: *"The price modifier term v(r) **doubles the price if the
//! storage is empty and halves the price if the storage is full**."* A full store is half price, an empty
//! one is double, and half full is the nominal unit — which is exactly a scarcity signal expressed per
//! head, since `r` is a fraction of capacity rather than a count of units.
//!
//! **What does not transfer.** `w` is the paper's real-world market index (a 60-day moving average of the
//! SMI), so it needs an outside data source the crate does not have; `k` is a ±10% randomisation, which the
//! crate's determinism forbids. Both are dropped, and the nominal unit is `1.0`, which is the value
//! [`Prices`](crate::Prices) already documents for a good nobody has traded. `v` — the scarcity shape — is
//! the whole model, and it is unchanged.

/// How dear a good is relative to its nominal unit, given the fraction `fill` of village storage it occupies.
///
/// The corpus's `v(r)`: `2(1 − r)` below half full, `1.5 − r` above it. Continuous at `r = 0.5`, where both
/// branches give `1.0`; `2.0` at an empty store and `0.5` at a full one, which the paper spells out as
/// "doubles the price if the storage is empty and halves the price if the storage is full".
///
/// `fill` outside `[0, 1]` is clamped: a store cannot be more than full, and a negative total is a defect
/// upstream rather than a price signal.
pub fn scarcity_factor(fill: f32) -> f32 {
    let r = fill.clamp(0.0, 1.0);
    if r < 0.5 { -2.0 * (r - 1.0) } else { 1.5 - r }
}

/// The price a village's stores imply for a good, before it is clamped to the host's band.
///
/// `nominal · v(mean_held / full_store)` where `mean_held = total_held / people`. **Per head by
/// construction**: one loaf in a village of three and one in a village of thirty are different fractions of
/// the same level.
///
/// **`full_store` is the level at which the good is plentiful — half price — and it is not the storage
/// limit.** The paper's `S_max` is "the storage size" of a producer that spends a whole run filling it, so
/// `r` spans the unit interval there. A village's hard clamp is `Tuning::stock_ceiling` (six units a person)
/// and its people hold a quarter of a unit each: measured over 17 game-days at seed 7 the mean never
/// exceeds **0.71**, so `r` tops out at **0.12** and `v` never leaves **1.76 – 2.00**. Pricing off the clamp
/// therefore pins the price at the famine end of its range whatever the village does — a scarcity index that
/// reads the same in a glut and a drought, which is the defect this whole change is repairing, one level
/// down. `full_store` is a **design default** ([`Tuning::price_full_store`](crate::Tuning::price_full_store))
/// at one unit: one loaf is a meal, so a loaf a head is a village with a day in hand.
///
/// A `full_store` of zero and an empty village have no ratio to read, so they return the nominal unit: no
/// observation rather than a wrong one.
pub fn storage_price(total_held: f32, people: usize, full_store: f32, nominal: f32) -> f32 {
    let level = people as f32 * full_store.max(0.0);
    if level <= 0.0 {
        return nominal;
    }
    let mean_held = total_held.max(0.0) / people.max(1) as f32;
    nominal * scarcity_factor(mean_held / full_store.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The paper's three named points, verbatim: empty is double, full is half, half is nominal.
    ///
    /// This is the whole claim the model makes, and it is checkable against the sentence rather than
    /// against a fitted curve: *"v(r) doubles the price if the storage is empty and halves the price if the
    /// storage is full."*
    #[test]
    fn the_papers_three_named_points() {
        assert!((scarcity_factor(0.0) - 2.0).abs() < 1e-6, "empty is double");
        assert!((scarcity_factor(0.5) - 1.0).abs() < 1e-6, "half is nominal");
        assert!((scarcity_factor(1.0) - 0.5).abs() < 1e-6, "full is half");
    }

    /// The two branches meet at `r = 0.5`, and the function falls everywhere: a fuller store is never
    /// dearer. A discontinuity or a non-monotone step would be a transcription error of the piecewise form.
    #[test]
    fn it_is_continuous_and_falls_with_fill() {
        let mut previous = scarcity_factor(0.0);
        for step in 1..=1000 {
            let fill = step as f32 / 1000.0;
            let factor = scarcity_factor(fill);
            assert!(
                factor <= previous + 1e-6,
                "a fuller store must not be dearer: fill {fill} scored {factor} against {previous}"
            );
            previous = factor;
        }
        let left = scarcity_factor(0.5 - 1e-4);
        let right = scarcity_factor(0.5 + 1e-4);
        assert!(
            (left - right).abs() < 1e-3,
            "the branches must meet at half full: {left} against {right}"
        );
    }

    /// **The second recorded defect, as arithmetic: the same store is dearer in a bigger village.**
    ///
    /// Six loaves is six a head for one person and a fifth of one for thirty, and no unit-counting formula
    /// can say so. One person is past a full store and clamps to half price; thirty share `0.2` a head,
    /// which is below half full, so `v = 2(1 − 0.2) = 1.6`.
    #[test]
    fn the_same_store_is_dearer_per_head() {
        let one = storage_price(6.0, 1, 1.0, 1.0);
        let thirty = storage_price(6.0, 30, 1.0, 1.0);
        assert!((one - 0.5).abs() < 1e-6, "one person holding six: {one}");
        assert!(
            (thirty - 1.6).abs() < 1e-6,
            "six loaves for thirty people: {thirty}"
        );
        assert!(thirty > one, "the bigger village must price it dearer");
    }

    /// A village holding more than a day a head is clamped at the full-store end, not extrapolated: `v`
    /// would otherwise go negative and a price below zero is not a scarcity signal.
    #[test]
    fn over_a_full_store_is_clamped_and_an_empty_village_is_nominal() {
        assert!((storage_price(20.0, 1, 1.0, 1.0) - 0.5).abs() < 1e-6);
        assert_eq!(storage_price(3.0, 0, 1.0, 1.0), 1.0, "no people, no ratio");
        assert_eq!(storage_price(3.0, 4, 0.0, 1.0), 1.0, "no level, no ratio");
    }

    /// **The saturation this model had to be rescued from.** Pricing the ratio against the storage *clamp*
    /// (six units a head) put the village at `r <= 0.12` and pinned the price in 1.76 – 2.00 whatever it did;
    /// against a day's supply a head, the same stores span most of `v`'s useful range.
    #[test]
    fn a_day_a_head_is_a_range_and_a_clamp_is_not() {
        // The village's own measured stores: 0.08, 0.25 and 0.71 of a loaf a head.
        let with_clamp: Vec<f32> = [0.08, 0.25, 0.71]
            .iter()
            .map(|held| storage_price(held * 24.0, 24, 6.0, 1.0))
            .collect();
        let with_day: Vec<f32> = [0.08, 0.25, 0.71]
            .iter()
            .map(|held| storage_price(held * 24.0, 24, 1.0, 1.0))
            .collect();
        let clamp_span = with_clamp[0] - with_clamp[2];
        let day_span = with_day[0] - with_day[2];
        assert!(
            clamp_span < 0.25,
            "the clamp pins the price into {clamp_span} of range, which is the saturation: {with_clamp:?}"
        );
        assert!(
            day_span > 0.7,
            "a day a head must let the same stores move the price visibly: {with_day:?}"
        );
    }
}
