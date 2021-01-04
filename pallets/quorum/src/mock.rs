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
	Perbill,
};
use balances as pallet_balances;

pub type AccountId = u128;
pub type BlockNumber = u64;

// test runtime
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;

mod quorum {
	pub use super::super::*;
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		pallet_balances<T>,
		quorum<T>,
	}
}

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type AvailableBlockRatio = AvailableBlockRatio;
	type MaximumBlockLength = MaximumBlockLength;
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
}
pub type System = frame_system::Module<Runtime>;

ord_parameter_types! {
	pub const One: AccountId = 1;
}

impl Trait for Runtime {
	type Event = TestEvent;
	type Currency = pallet_balances::Module<Runtime>;
	// type WeightInfo = ();
}
pub type QuorumModule = Module<Runtime>;


parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Trait for Runtime {
	type MaxLocks = ();
	type Balance = u64;
	type Event = TestEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
pub type PalletBalances = pallet_balances::Module<Runtime>;

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		// inject test balances
		pallet_balances::GenesisConfig::<Runtime>{
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
