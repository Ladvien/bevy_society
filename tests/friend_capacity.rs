//! **A person's closeness is a budget** (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970): a bond not yet
//! formed gains affinity more slowly the more friends its owner already holds, and not at all at
//! `Tuning::friend_capacity`; a bond that is already a friend, and every loss, keep the full rate.

use bevy_society::Tuning;

#[test]
fn brake_scales_a_new_bond_by_the_room_left() {
    let tuning = Tuning::default();
    assert!(tuning.friend_capacity > 0.0, "the brake ships on");
    let cap = tuning.friend_capacity;

    assert!(
        (tuning.bond_scale(0, false) - 1.0).abs() < 1e-6,
        "an empty circle is not braked"
    );
    assert!(
        (tuning.bond_scale(cap as usize / 2, false) - 0.5).abs() < 1e-6,
        "half full, half the rate"
    );
    assert_eq!(
        tuning.bond_scale(cap as usize, false),
        0.0,
        "a full circle forms no new bond"
    );
    assert_eq!(
        tuning.bond_scale(cap as usize * 3, false),
        0.0,
        "and never goes negative"
    );
}

#[test]
fn an_existing_friend_and_a_disabled_brake_are_not_braked() {
    let tuning = Tuning::default();
    assert_eq!(
        tuning.bond_scale(1000, true),
        1.0,
        "a full circle keeps its friends"
    );

    let off = Tuning {
        friend_capacity: 0.0,
        ..Tuning::default()
    };
    assert_eq!(off.bond_scale(1000, false), 1.0, "0.0 turns the brake off");
}
