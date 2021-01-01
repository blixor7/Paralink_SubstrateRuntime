#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	codec::{Decode, Encode},
	traits::{Currency, ReservableCurrency},
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	RuntimeDebug,
};

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;
mod tests;

/// A maximum number of relayers.
/// When membership reaches this number, no new relayers may join.
pub const MAX_RELAYERS: usize = 32;

pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct Quorum<AccountId, BalanceOf> {
	pub relayers: Vec<AccountId>,
	pub balances: Vec<BalanceOf>,
	pub creator: AccountId,
	pub pending_rewards: BalanceOf,
	pub fee: BalanceOf,
}


pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
pub type QuorumOf<T> = Quorum<<T as system::Trait>::AccountId, BalanceOf<T>>;
pub type QuorumIndex = u32;

decl_storage! {
	trait Store for Module<T: Trait> as RelayerQuorums {
		Quorums get(fn quorums): map hasher(blake2_128_concat) u32 => QuorumOf<T>;

		/// Number of existing quorums. Also used as a hashmap index.
		QuorumCount get(fn quorum_count): QuorumIndex;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		{
			QuorumCreated(QuorumIndex, AccountId),
			RelayerAdded(QuorumIndex, AccountId),
			RelayerRemoved(QuorumIndex, AccountId),
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyRelayer,
		NotRelayer,
		RelayerLimitReached,
		InvalidQuorum,
		Unauthorized,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		type Error = Error<T>;

		/// Create a new quorum
		#[weight = 10_000]
		pub fn create(origin) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO: add a minimum fee that is burned

			// Safely update the quorum index
			let index = QuorumCount::get()
				.checked_add(1)
				.ok_or("quorum index overflow")?;
			QuorumCount::put(index);

			// Create a new quorum
			<Quorums<T>>::insert(index, Quorum {
				relayers: vec![],
				balances: vec![],
				creator: who.clone(),
				pending_rewards: 0.into(),
				fee: 0.into(),
			});

			Self::deposit_event(RawEvent::QuorumCreated(index, who));
			Ok(())
		}

		/// Creator adds a relayer to the relayers set unless the max is reached
		#[weight = 10_000]
		pub fn add_relayer(origin, quorum_id: QuorumIndex, relayer: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut quorum = Self::find_quorum(quorum_id)?;

			// only quorum creator can add/remove new relayers (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			ensure!(quorum.relayers.len() < MAX_RELAYERS, Error::<T>::RelayerLimitReached);

			// Avoid duplicates. Because the list is always ordered, we can
			// leverage the binary search which makes this check O(log n).
			match quorum.relayers.binary_search(&relayer) {
				// If the search succeeds, the caller is already a relayer, so just return
				Ok(_) => Err(Error::<T>::AlreadyRelayer.into()),
				// If the search fails, the caller is not a relayer and we learned the index where
				// they should be inserted
				Err(index) => {
					// TODO: trigger pending rewards distribution
					quorum.relayers.insert(index, relayer.clone());
					quorum.balances.insert(index, 0.into());
					// Upsert the quorum
					<Quorums<T>>::insert(&quorum_id, quorum);
					Self::deposit_event(RawEvent::RelayerAdded(quorum_id, relayer));
					Ok(())
				}
			}
		}

		/// Creator removes a relayer.
		#[weight = 10_000]
		pub fn remove_relayer(origin, quorum_id: QuorumIndex, relayer: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let (mut quorum, index) = Self::find_quorum_relayer(quorum_id, relayer.clone())?;

			// only quorum creator can add/remove new relayers (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			// TODO: trigger pending rewards distribution
			let balance = quorum.balances.get(index).unwrap();
			if *balance > BalanceOf::<T>::from(0) {
				T::Currency::deposit_into_existing(&who, *balance)?;
			}
			quorum.relayers.remove(index);
			quorum.balances.remove(index);
			<Quorums<T>>::insert(&quorum_id, quorum);
			Self::deposit_event(RawEvent::RelayerRemoved(quorum_id, relayer));
			Ok(())
		}

		/// Relayer leaves
		#[weight = 10_000]
		pub fn leave(origin, quorum_id: QuorumIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let (mut quorum, index) = Self::find_quorum_relayer(quorum_id, who.clone())?;

			// TODO: trigger pending rewards distribution
			let balance = quorum.balances.get(index).unwrap();
			if *balance > BalanceOf::<T>::from(0) {
				T::Currency::deposit_into_existing(&who, *balance)?;
			}
			quorum.relayers.remove(index);
			quorum.balances.remove(index);
			<Quorums<T>>::insert(&quorum_id, quorum);
			Self::deposit_event(RawEvent::RelayerRemoved(quorum_id, who));
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	/// Find a quorum
	pub fn find_quorum(quorum_id: QuorumIndex) ->
		Result<Quorum<T::AccountId, BalanceOf<T>>, Error<T>>
	{
		ensure!(<Quorums<T>>::contains_key(&quorum_id), Error::<T>::InvalidQuorum);
		let quorum = <Quorums<T>>::get(&quorum_id);
		Ok(quorum)
	}

	/// Find and return a quorum and the location of its relayer
	pub fn find_quorum_relayer(quorum_id: QuorumIndex, relayer: T::AccountId) ->
		Result<(Quorum<T::AccountId, BalanceOf<T>>, usize), Error<T>>
	{
		let quorum = Self::find_quorum(quorum_id)?;

		match quorum.relayers.binary_search(&relayer) {
			Ok(index) => {
				Ok((quorum, index))
			},
			Err(_) => Err(Error::<T>::NotRelayer.into()),
		}
	}

	/// Distribute pending_rewards between quorum relayers
	fn _distribute_pending_rewards() {
		todo!();
	}
}
