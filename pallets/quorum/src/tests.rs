#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, assert_err};
use sp_runtime::{traits::BadOrigin};

use mock::{
	System, Runtime, ExtBuilder, Origin,
	QuorumModule, Balances,
	TestEvent,
};

#[test]
fn test_ext_builder() {
	ExtBuilder::default().build().execute_with(|| {
		// check that alice has injected funds
		let alice = 1 as u128;
		let balance = Balances::total_balance(&alice);
		assert_eq!(balance, 10_000 as u64);

		// check alice balance from QuorumModule::Currency
		let balance = Balances::free_balance(&alice);
		assert_eq!(balance, 10_000 as u64);

		// check total issuance
		assert_eq!(Balances::total_issuance(), 40_000 as u64);
	});
}

#[test]
fn test_create_quorum() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = 1 as u128;
		let nobody = 42 as u128;

		// alice has enough funds for the fee
		assert_eq!(QuorumCount::get(), 0);
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, false));
		assert_eq!(QuorumCount::get(), 1);

		// nobody does not
		assert!(QuorumModule::create(Origin::signed(nobody), 0, false).is_err());
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
		let alice = 1 as u128;
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
fn test_request() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let request_id = 1 as u32;
		let alice = 1 as u128;
		let valid_period = 10 as u32;
		let fee = 100 as u64;
		assert_ok!(QuorumModule::create(Origin::signed(alice), fee.into(), false));
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, alice));

		// new request is made
		let old_balance = Balances::total_balance(&alice);
		assert_ok!(
			QuorumModule::request(
				Origin::signed(alice),
				quorum_id,
				[0; 32],
				fee.into(),
				valid_period,
				1,
			)
		);
		assert_ok!(QuorumModule::find_request(request_id));

		// check that fee was paid
		let new_balance = Balances::total_balance(&alice);
		assert_eq!(old_balance, new_balance + fee);
	});
}

#[test]
fn test_request_expires() {
	ExtBuilder::default().build().execute_with(|| {
		let quorum_id = 1 as u32;
		let request_id = 1 as u32;
		let alice = 1 as u128;
		let valid_period = 10 as u32;
		let fee = 100 as u64;
		assert_ok!(QuorumModule::create(Origin::signed(alice), 0, false));
		assert_ok!(QuorumModule::add_relayer(Origin::signed(alice), quorum_id, alice));

		// new request is made
		assert_ok!(
			QuorumModule::request(
				Origin::signed(alice),
				quorum_id,
				[0; 32],
				fee.into(),
				valid_period,
				1,
			)
		);
		assert_ok!(QuorumModule::find_request(request_id));

		// TODO: advance x blocks, check that request was invalidated
		// skip_blocks(valid_period + 1);
		// assert!(QuorumModule::find_request(request_id).is_err());
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


// fn skip_blocks(n: u32) {
// 	for _ in 0..n {
// 		QuorumModule::on_finalize(System::block_number());
// 		System::set_block_number(System::block_number() + 1);
// 	}
// }
