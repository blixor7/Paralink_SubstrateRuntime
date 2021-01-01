#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, assert_err};
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

use mock::{Runtime, ExtBuilder, Origin, QuorumModule, System, TestEvent};

#[test]
fn test_create_quorum() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(QuorumCount::get(), 0);
		assert_ok!(QuorumModule::create(Origin::signed(1)));
		assert_eq!(QuorumCount::get(), 1);
	});
}

#[test]
fn test_add_remove_relayer() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(QuorumModule::create(Origin::signed(1)));
		let quorum_id = 1 as u32;

		// add alice and verify
		let alice = 2 as u128;
		assert_ok!(QuorumModule::add_relayer(Origin::signed(1), quorum_id, alice));
		assert_ok!(QuorumModule::find_quorum_relayer(quorum_id, alice));
		let (quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, alice).unwrap();
		assert!(quorum.relayers.len() == 1);
		assert!(index == 0);

		// cannot add alice twice
		assert_err!(
			QuorumModule::add_relayer(Origin::signed(1), quorum_id, alice),
			Error::<Runtime>::AlreadyRelayer
		);

		// add bob
		let bob = 3 as u128;
		assert_ok!(QuorumModule::add_relayer(Origin::signed(1), quorum_id, bob));
		assert_ok!(QuorumModule::find_quorum_relayer(quorum_id, bob));
		let (quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, bob).unwrap();
		assert!(quorum.relayers.len() == 2);
		assert!(index == 1);

		// alice gets kicked
		assert_ok!(QuorumModule::remove_relayer(Origin::signed(1), quorum_id, alice));
		let quorum = QuorumModule::find_quorum(quorum_id).unwrap();
		assert!(quorum.relayers.len() == 1);
		assert!(!quorum.relayers.contains(&alice));

		// bob's index has changed
		let (quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, bob).unwrap();
		assert!(index == 0);

		// TODO
		// test rewards distribution on join
		// test rewards distribution and withdrawal on kick
	});
}

#[test]
fn test_leave() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(QuorumModule::create(Origin::signed(1)));
		let quorum_id = 1 as u32;

		// add alice and verify
		let alice = 3 as u128;
		assert_ok!(QuorumModule::add_relayer(Origin::signed(1), quorum_id, alice));

		// alice leaves
		assert_ok!(QuorumModule::leave(Origin::signed(alice), quorum_id));
		let quorum = QuorumModule::find_quorum(quorum_id).unwrap();
		assert!(quorum.relayers.len() == 0);

		// TODO
		// test rewards distribution and withdrawal on leave
	});
}


#[test]
fn test_unauthorized() {
	ExtBuilder::default().build().execute_with(|| {
		// alice owns the quorum
		assert_ok!(QuorumModule::create(Origin::signed(1)));
		// so bob can't add things to it
		assert_err!(
			QuorumModule::add_relayer(Origin::signed(2), 1 as u32, 2 as u128),
			Error::<Runtime>::Unauthorized
		);
		// only alice can
		assert_ok!(QuorumModule::add_relayer(Origin::signed(1), 1 as u32, 2 as u128));
		// test removal endpoint too
		assert!(QuorumModule::remove_relayer(Origin::signed(2), 1 as u32, 2 as u128).is_err());
		assert_ok!(QuorumModule::remove_relayer(Origin::signed(1), 1 as u32, 2 as u128));
	});
}
