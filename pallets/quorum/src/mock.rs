#![cfg(test)]

use super::*;
use frame_support::{
	weights::Weight,
	impl_outer_event, impl_outer_origin,
	ord_parameter_types, parameter_types,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::IdentityLookup,
};

use crate as pallet_quorum;
use balances as pallet_balances;

pub type AccountId = u128;
pub type BlockNumber = u64;

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Event<T>},
		QuorumModule: pallet_quorum::{Module, Call, Storage, Event<T>},
	}
);

mod quorum {
	pub use super::super::*;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

impl frame_system::Config for TestRuntime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

impl pallet_quorum::Config for TestRuntime {
	type Event = Event;
	type Currency = pallet_balances::Module<TestRuntime>;
	// type WeightInfo = ();
}


parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for TestRuntime {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();

		// inject test balances
		pallet_balances::GenesisConfig::<TestRuntime>{
			balances: vec![
				(0, 10_000), // root
				(1, 10_000), // alice
				(2, 10_000), // bob
				(3, 10_000), // charlie
			],
		}.assimilate_storage(&mut t).unwrap();

		t.into()
	}
}
