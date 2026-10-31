//! Response curves: how one consideration's normalised input becomes a utility factor. The
//! compensation and dual-utility selection built on them come from Mark & Dill (Game AI Pro 3,
//! ch. 13, "Choosing Effective Utility-Based Considerations"; corpus doc
//! `gameaipro3-ch13-choosing-effective-utility-ba…`).

use serde::{Deserialize, Serialize};

/// A response curve applied to one consideration's input, always yielding a value in `[0, 1]`.
///
/// `Hours` reads the hour of day, so its input is the normalised hour `h / 24`. `Days` reads the position in the
/// week, so its input is `(weekday + h / 24) / 7`.
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
    /// `1` inside the wrapping week window `[from, to)`, in days from the start of the week, otherwise `0`; see
    /// [`day_window`]. `Days(5.0, 7.0)` is the last two days of the week.
    Days(f32, f32),
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
            Curve::Days(from, to) => day_window(from, to, x * f32::from(DAYS_PER_WEEK)),
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
    wrapped_window(from, to, hour, 24.0)
}

/// Days in the week: a calendar convention, not a model parameter. A host with another calendar scripts it.
pub const DAYS_PER_WEEK: u8 = 7;

/// `1.0` when `day` (a position in the week, wrapped onto [`DAYS_PER_WEEK`]) lies in `[from, to)`, wrapping past the
/// end of the week when `from > to`; `0.0` otherwise. `from == to` is an empty window. `day` is the weekday plus the
/// fraction of the day elapsed, so `Days(4.75, 6.0)` opens at 18:00 on the fifth day.
pub fn day_window(from: f32, to: f32, day: f32) -> f32 {
    wrapped_window(from, to, day, f32::from(DAYS_PER_WEEK))
}

/// The wrapping window `[from, to)` on a clock of `period` units.
fn wrapped_window(from: f32, to: f32, at: f32, period: f32) -> f32 {
    let at = at.rem_euclid(period);
    let inside = if from < to {
        at >= from && at < to
    } else if from > to {
        at >= from || at < to
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

    #[test]
    fn day_window_wraps_past_the_end_of_the_week() {
        // Days(5, 7) is the rest days; Days(6, 1) is the last day and the first, wrapping.
        assert_eq!(day_window(5.0, 7.0, 5.0), 1.0);
        assert_eq!(day_window(5.0, 7.0, 6.99), 1.0);
        assert_eq!(day_window(5.0, 7.0, 4.99), 0.0);
        assert_eq!(day_window(6.0, 1.0, 6.5), 1.0);
        assert_eq!(day_window(6.0, 1.0, 0.5), 1.0);
        assert_eq!(day_window(6.0, 1.0, 1.0), 0.0);
        assert_eq!(day_window(2.0, 2.0, 2.0), 0.0, "from == to is empty");
        // A mid-day edge: Friday 18:00 to the end of Saturday.
        assert_eq!(day_window(4.75, 6.0, 4.74), 0.0);
        assert_eq!(day_window(4.75, 6.0, 4.75), 1.0);
        assert_eq!(day_window(4.75, 6.0, 5.99), 1.0);
        assert_eq!(day_window(4.75, 6.0, 6.0), 0.0);
        // The week wraps: day 7 is day 0 again.
        assert_eq!(day_window(0.0, 5.0, 7.5), 1.0);
    }

    #[test]
    fn days_curve_puts_the_midnight_tick_on_the_new_day() {
        // The input a decision hands the curve is `(weekday + hour / 24) / 7`, in f32. Each day's first instant must
        // read as that day and the instant before it as the day before, for all seven days: the division and the
        // multiplication must not land 1 ulp short of the edge.
        for day in 0..7u8 {
            let first = f32::from(day) / 7.0;
            assert_eq!(Curve::Days(f32::from(day), f32::from(day) + 1.0).eval(first), 1.0, "day {day}");
            let last_tick = (f32::from(day) + 23.99 / 24.0) / 7.0;
            assert_eq!(Curve::Days(f32::from(day), f32::from(day) + 1.0).eval(last_tick), 1.0, "end of day {day}");
            let next_day = (f32::from(day) + 1.0) / 7.0;
            assert_eq!(Curve::Days(f32::from(day), f32::from(day) + 1.0).eval(next_day), 0.0, "start of day {}", day + 1);
        }
    }
}
