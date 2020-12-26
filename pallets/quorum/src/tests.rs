#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

use mock::{ExtBuilder, Origin, QuorumModule, System, TestEvent};

#[test]
fn test_create_quorum() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(QuorumModule::create(Origin::signed(1)));
    });
}

