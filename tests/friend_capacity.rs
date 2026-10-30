//! **A person's closeness is a budget** (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970): a bond not yet
//! formed gains affinity more slowly the more friends its owner already holds, and not at all at
//! `Tuning::friend_capacity`; a bond that is already a friend, and every loss, keep the full rate.

use bevy_society::Tuning;

#[test]
fn brake_scales_a_new_bond_by_the_room_left() {
    let tuning = Tuning {
        friend_capacity: 10.0,
        ..Tuning::default()
    };
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
    let tuning = Tuning {
        friend_capacity: 10.0,
        ..Tuning::default()
    };
    assert_eq!(
        tuning.bond_scale(1000, true),
        1.0,
        "a full circle keeps its friends"
    );

    let off = Tuning::default();
    assert_eq!(off.friend_capacity, 0.0, "the brake is opt-in");
    assert_eq!(off.bond_scale(1000, false), 1.0, "0.0 turns the brake off");
}

#[test]
fn closeness_is_read_from_the_state_and_not_from_the_label() {
    use bevy::prelude::Entity;
    use bevy_society::{TieLabel, Ties};

    let tuning = Tuning {
        friend_capacity: 10.0,
        ..Tuning::default()
    };
    let mut ties = Ties::default();
    let close = Entity::from_raw_u32(1).unwrap();
    let far = Entity::from_raw_u32(2).unwrap();
    let labelled_only = Entity::from_raw_u32(3).unwrap();
    {
        let t = ties.entry(close);
        t.affinity = tuning.friend_affinity;
        t.trust = tuning.friend_trust;
    }
    ties.entry(far).affinity = tuning.friend_affinity - 0.01;
    // A friendship the label still holds through a dip (`friend_band`) is not close and takes no seat.
    ties.entry(labelled_only).label = TieLabel::Friend;

    assert_eq!(ties.friend_count(&tuning), 1);
    assert_eq!(
        ties.friend_count(&Tuning::default()),
        0,
        "the brake off counts nothing"
    );
}
