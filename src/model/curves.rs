//! Response curves: how one consideration's normalised input becomes a utility factor. The
//! compensation and dual-utility selection built on them come from Mark & Dill (Game AI Pro 3,
//! ch. 13, "Choosing Effective Utility-Based Considerations"; corpus doc
//! `gameaipro3-ch13-choosing-effective-utility-ba…`).

use serde::{Deserialize, Serialize};

/// A response curve applied to one consideration's input, always yielding a value in `[0, 1]`.
///
/// `Hours` reads the hour of day, so its input is the normalised hour `h / 24`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Curve {
    /// `clamp(m·x + b, 0, 1)`.
    Linear(f32, f32),
    /// `x^k` for `x` clamped to `[0, 1]`.
    Power(f32),
    /// `1 / (1 + e^(−k(x − x0)))`, see [`logistic`].
    Logistic(f32, f32),
    /// `1` when `x ≥ at`, otherwise `0`.
    Step(f32),
    /// `1` inside the wrapping hour window `[from, to)`, otherwise `0`; see [`hour_window`].
    Hours(f32, f32),
}

impl Curve {
    /// Applies the curve to a normalised input `x`.
    pub fn eval(self, x: f32) -> f32 {
        match self {
            Curve::Linear(m, b) => (m * x + b).clamp(0.0, 1.0),
            Curve::Power(k) => x.clamp(0.0, 1.0).powf(k),
            Curve::Logistic(k, x0) => logistic(k, x0, x),
            Curve::Step(at) => {
                if x >= at {
                    1.0
                } else {
                    0.0
                }
            }
            Curve::Hours(from, to) => hour_window(from, to, x * 24.0),
        }
    }
}

/// The logistic `1 / (1 + e^(−k(x − x0)))`, which is `0.5` at `x = x0` and steepens with `k`.
pub fn logistic(k: f32, x0: f32, x: f32) -> f32 {
    1.0 / (1.0 + (-k * (x - x0)).exp())
}

/// `1.0` when `hour` (wrapped onto a 24-hour clock) lies in `[from, to)`, wrapping past midnight
/// when `from > to`; `0.0` otherwise. `from == to` is an empty window.
pub fn hour_window(from: f32, to: f32, hour: f32) -> f32 {
    let h = hour.rem_euclid(24.0);
    let inside = if from < to {
        h >= from && h < to
    } else if from > to {
        h >= from || h < to
    } else {
        false
    };
    if inside { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_clamps_to_unit_interval() {
        assert!((Curve::Linear(-1.0, 1.0).eval(0.25) - 0.75).abs() < 1e-6);
        assert_eq!(Curve::Linear(2.0, 0.0).eval(0.9), 1.0);
        assert_eq!(Curve::Linear(-1.0, 0.0).eval(0.5), 0.0);
    }

    #[test]
    fn logistic_is_half_at_its_midpoint() {
        assert!((logistic(8.0, 0.7, 0.7) - 0.5).abs() < 1e-6);
        assert!(logistic(8.0, 0.7, 1.0) > 0.9);
        assert!(logistic(8.0, 0.7, 0.0) < 0.01);
    }

    #[test]
    fn hour_window_wraps_past_midnight() {
        // Hours(22, 6) is the sleep window: 23:00 and 03:00 are in, noon is out.
        assert_eq!(hour_window(22.0, 6.0, 23.0), 1.0);
        assert_eq!(hour_window(22.0, 6.0, 3.0), 1.0);
        assert_eq!(hour_window(22.0, 6.0, 12.0), 0.0);
        assert_eq!(hour_window(8.0, 17.0, 17.0), 0.0);
        assert_eq!(hour_window(8.0, 8.0, 8.0), 0.0);
    }

    #[test]
    fn hours_curve_reads_a_normalised_hour() {
        assert_eq!(Curve::Hours(22.0, 6.0).eval(23.0 / 24.0), 1.0);
        assert_eq!(Curve::Hours(8.0, 17.0).eval(12.0 / 24.0), 1.0);
    }

    #[test]
    fn step_and_power() {
        assert_eq!(Curve::Step(0.5).eval(0.5), 1.0);
        assert_eq!(Curve::Step(0.5).eval(0.49), 0.0);
        assert!((Curve::Power(2.0).eval(0.5) - 0.25).abs() < 1e-6);
    }
}
