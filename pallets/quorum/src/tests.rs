#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, assert_err};
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

use mock::{Runtime, ExtBuilder, Origin, QuorumModule, System, TestEvent};

#[test]
fn test_create_quorum() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(QuorumCount::get(), 0);
		assert_ok!(QuorumModule::create(Origin::signed(1), 0, false));
		assert_eq!(QuorumCount::get(), 1);
	});
}

#[test]
fn test_add_remove_relayer() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let alice = 1 as u128;
		let bob = 2 as u128;
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, false));

		// add alice and verify
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, alice));
		assert_ok!(QuorumModule::find_quorum_relayer(quorum_id, alice));
		let (quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, alice).unwrap();
		assert!(quorum.relayers.len() == 1);
		assert!(index == 0);

		// cannot add alice twice
		assert_err!(
			QuorumModule::add_relayer(Origin::signed(alice), quorum_id, alice),
			Error::<Runtime>::AlreadyRelayer
		);

		// add bob
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, bob));
		assert_ok!(QuorumModule::find_quorum_relayer(quorum_id, bob));
		let (quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, bob).unwrap();
		assert!(quorum.relayers.len() == 2);
		assert!(index == 1);

		// alice gets kicked
		assert_ok!(QuorumModule::remove_relayer(Origin::signed(alice), quorum_id, alice));
		let quorum = QuorumModule::find_quorum(quorum_id).unwrap();
		assert!(quorum.relayers.len() == 1);
		assert!(!quorum.relayers.contains(&alice));

		// bob's index has changed
		let (_quorum, index) = QuorumModule::find_quorum_relayer(quorum_id, bob).unwrap();
		assert!(index == 0);

		// TODO
		// test rewards distribution on join
		// test rewards distribution and withdrawal on kick
	});
}

#[test]
fn test_leave_quorum() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let alice = 3 as u128;
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, false));

		// add alice and verify
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, alice));

		// alice leaves
		assert_ok!(QuorumModule::leave(Origin::signed(alice), quorum_id));
		let quorum = QuorumModule::find_quorum(quorum_id).unwrap();
		assert!(quorum.relayers.len() == 0);

		// TODO
		// test rewards distribution and withdrawal on leave
	});
}

#[test]
fn test_add_remove_user() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let alice = 1 as u128;
		let bob = 2 as u128;
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, true));

		// add alice and verify
		assert_ok!(QuorumModule::add_user(Origin::signed(alice), quorum_id, alice));
		assert!(QuorumModule::is_quorum_user(quorum_id, alice));
		assert_ok!(QuorumModule::add_user(Origin::signed(alice), quorum_id, alice));


		// alice gets kicked
		assert_ok!(QuorumModule::remove_user(Origin::signed(alice), quorum_id, alice));
		assert!(!QuorumModule::is_quorum_user(quorum_id, alice));
	});
}

#[test]
fn test_unauthorized() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let alice = 1 as u128;
		let bob = 2 as u128;
		// alice owns the quorum
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, true));
		// so bob can't add relayers
		assert_err!(
			QuorumModule::add_relayer(Origin::signed(bob), quorum_id, bob),
			Error::<Runtime>::Unauthorized
		);
		// only alice can add relayers
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, bob));
		// test removal
		assert!(QuorumModule::remove_relayer(Origin::signed(bob), quorum_id, bob).is_err());
		assert_ok!(QuorumModule::remove_relayer(Origin::signed(alice), quorum_id, bob));

		// only alice can add users
		assert_ok!(QuorumModule::add_user(Origin::signed(alice), quorum_id, bob));
		assert_ok!(QuorumModule::remove_user(Origin::signed(alice), quorum_id, bob));
		assert_err!(
			QuorumModule::add_user(Origin::signed(bob), quorum_id, bob),
			Error::<Runtime>::Unauthorized
		);
		assert_err!(
			QuorumModule::remove_user(Origin::signed(bob), quorum_id, bob),
			Error::<Runtime>::Unauthorized
		);
	});
}
