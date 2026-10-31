//! Engine-free model math: response curves, the personality-to-mood baseline, opinion dynamics,
//! recall and forgetting, rumor spreading, structural balance and the scarcity price on a good.
//! Nothing here names a Bevy type; every function is a pure function of its arguments, so each
//! formula is tested against its paper's worked values.

pub mod balance;
pub mod curves;
pub mod opinion;
pub mod pad;
pub mod price;
pub mod recall;
pub mod rumor_dk;

pub use curves::{Curve, DAYS_PER_WEEK};
