//! Engine-free model math: response curves, the personality-to-mood baseline, opinion dynamics,
//! recall and forgetting, rumor spreading and structural balance. Nothing here names a Bevy type;
//! every function is a pure function of its arguments, so each formula is tested against its
//! paper's worked values.

pub mod balance;
pub mod curves;
pub mod opinion;
pub mod pad;
pub mod recall;
pub mod rumor_dk;

pub use curves::Curve;
