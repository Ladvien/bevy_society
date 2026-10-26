//! Congestion as a **price** on a place, not a prohibition.
//!
//! A place declares how many people it seats. The obvious implementation is a filter — drop a place from
//! the candidate list once `used >= capacity` — and it is wrong in a way a host can watch: the activity
//! *disappears* rather than becoming expensive. Measured on the project's `village-evening`, whose four
//! places seat 30, `work` fell from **10.3% to 3.6%** of person-ticks as the population went 24 → 48,
//! because the workshop (6 seats) was permanently full and nobody could choose to work at all.
//!
//! Talamali, Bose, Haire, Xu, Marshall & Reina 2020, *Sophisticated collective foraging with minimalist
//! agents: a swarm robotics test*, *Swarm Intelligence* **14**(1):25-56, DOI
//! `10.1007/s11721-019-00176-9`, give the shape. Congestion on a used path is a travel-time penalty,
//! Eq. (12):
//!
//! $$T_C = T_0 \left(\exp\left[\kappa \frac{\rho S}{N_{\text{crit}}}\right] - 1\right)$$
//!
//! **negligible** while the number of users is well under the critical number the path fits, and growing
//! **exponentially** past it. Three of the paper's readings transfer:
//!
//! 1. A crowded place is **still used** — the collection curve decays nonlinearly, it does not fall to
//!    zero. So the multiplier must never reach zero for finite occupancy.
//! 2. The optimum is an **interior split** (§5.2.2): when two sources are close, allocating more workers
//!    to the lower-quality one beats crowding the better one. That requires a *gradient* across places,
//!    which an all-or-nothing filter cannot express.
//! 3. `N_crit` is the number of bodies that fit — which is exactly what a `Place`'s declared capacity is.
//!
//! The crate adopts the exponential-in-occupancy-over-seats shape and `capacity` as `N_crit`. The paper's
//! fitted `κ = 2.321` is **not** transferable: it is fitted to Kilobot collisions, and this crate
//! simulates no collision duration. The rate is therefore a design default on
//! [`Tuning::place_crowding_penalty`](crate::Tuning::place_crowding_penalty), chosen by measurement.

/// How much a place's desirability is multiplied by, given who is already there and how many it seats.
///
/// `exp(-penalty · max(0, (used + 1) / capacity − 1))`, which is **exactly `1.0`** up to and including the
/// last free seat and falls smoothly past it.
///
/// `used + 1` because the person being decided is not counted in `used` yet — it is asking for a seat, so
/// it is the *(used+1)*-th occupant whose cost matters. `max(0, ..)` is what keeps a village with room
/// byte-identical to a crate without this term at all.
///
/// A `capacity` of zero is a **prohibition and not a price**: a place that seats nobody seats nobody, so it
/// returns an exact `0.0` rather than the exponential's near-zero. That is the one case a host really can
/// mean "nobody", and it must not decay into "almost nobody".
///
/// A zero or negative `penalty` makes occupancy free, which is the honest reading of "the host turned the
/// congestion model off": every place then scores exactly `1.0`.
pub fn crowding_factor(used: u32, capacity: u32, penalty: f32) -> f32 {
    if capacity == 0 {
        return 0.0;
    }
    if !(penalty > 0.0) {
        return 1.0;
    }
    let over = ((used + 1) as f32 / capacity as f32 - 1.0).max(0.0);
    (-penalty * over).exp()
}

#[cfg(test)]
mod tests {
    use super::crowding_factor;

    /// The guard for the whole change: **a place with room is worth exactly what it was worth before**.
    ///
    /// Every village with more seats than people — including the project's own 24-people / 30-seats
    /// scenario — must be byte-identical to the crate without this term, or the change is untestable
    /// against a baseline. `used + 1` is the seat being asked for, so the last free seat is `used + 1 ==
    /// capacity`.
    #[test]
    fn a_place_with_room_is_exactly_one() {
        for capacity in 1..=16 {
            for used in 0..capacity {
                let factor = crowding_factor(used, capacity, 1.0);
                assert_eq!(
                    factor, 1.0,
                    "{used}/{capacity} is a free seat, so the multiplier must be exactly 1.0, not {factor}"
                );
            }
        }
    }

    /// Past capacity the multiplier falls, and it keeps falling: the paper's "nonlinear decay", which is
    /// what lets a decision prefer the emptier of two crowded places rather than treating them alike.
    #[test]
    fn past_capacity_it_falls_and_never_reaches_zero() {
        let capacity = 6;
        let mut previous = 1.0;
        for used in capacity..=(capacity * 4) {
            let factor = crowding_factor(used, capacity, 1.0);
            assert!(
                factor < previous,
                "{used}/{capacity} scored {factor}, not below {previous}"
            );
            assert!(
                factor > 0.0,
                "{used}/{capacity} scored {factor}: a crowded place is a price, never a prohibition, \
                 so the activity must stay reachable"
            );
            previous = factor;
        }
        // The step past capacity is `1/capacity` in the exponential's argument, because the paper's
        // argument is the *fraction* of the critical number, not a count of bodies. At a six-seat
        // workshop the seventh person costs `exp(-1/6) = 0.847`, the tenth `exp(-4/6) = 0.513`.
        assert!((crowding_factor(capacity, capacity, 1.0) - (-1.0_f32 / 6.0).exp()).abs() < 1e-6);
        assert!((crowding_factor(capacity + 3, capacity, 1.0) - (-4.0_f32 / 6.0).exp()).abs() < 1e-6);
    }

    /// Two places of the same size, one full and one half empty, must score **differently** — that
    /// difference is the paper's interior optimum. An all-or-nothing gate scores them alike.
    #[test]
    fn two_places_of_one_size_differ_by_their_occupancy() {
        let empty = crowding_factor(2, 6, 1.0);
        let full = crowding_factor(6, 6, 1.0);
        let packed = crowding_factor(12, 6, 1.0);
        assert_eq!(empty, 1.0);
        assert!(
            full < empty,
            "a full workshop ({full}) must be worth less than a half-empty one ({empty}), or the \
             decision cannot send the next worker to the emptier place"
        );
        assert!(
            packed < 0.5,
            "and a workshop twice over must be worth far less ({packed}), or the price never bites"
        );
    }

    /// A place that declares it seats nobody seats nobody. This is the pre-existing contract for
    /// `tests/bound_places.rs::a_capacity_zero_bound_place_admits_nobody`, kept exact rather than relying on
    /// `exp` of a large negative number.
    #[test]
    fn a_capacity_of_zero_admits_nobody() {
        assert_eq!(crowding_factor(0, 0, 1.0), 0.0);
        assert_eq!(crowding_factor(9, 0, 1.0), 0.0);
        assert_eq!(crowding_factor(9, 0, 0.0), 0.0);
    }

    /// The penalty is the rate, and turning it off must restore "occupancy does not matter" rather than
    /// "occupancy is prohibitive": a host that writes `0.0` wants the old permissive behaviour.
    #[test]
    fn a_zero_penalty_makes_occupancy_free() {
        assert_eq!(crowding_factor(40, 2, 0.0), 1.0);
        assert_eq!(crowding_factor(0, 2, 0.0), 1.0);
        assert!(
            crowding_factor(40, 2, 2.0) < crowding_factor(40, 2, 1.0),
            "a larger penalty must cost more, or the field is not read"
        );
    }
}
