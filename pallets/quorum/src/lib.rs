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

/// When quorum reaches this many relayers, new ones can't join
pub const MAX_RELAYERS: usize = 32;
/// What is the minimum number of blocks quorums have to service a request
pub const MIN_VALID_PERIOD: usize = 3;
/// For how many blocks is the longest pending request valid
pub const MAX_VALID_PERIOD: usize = 1000;
/// How much does it cost to create a new quorum
pub const QUORUM_CREATION_FEE: u128 = 0;

pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Encode, Decode, Clone, RuntimeDebug)]
pub enum Membership {
	/// Everyone Can make requests to the quorum
	Everyone,
	/// Only authorized users can make requests to the quorum
	Whitelist,
}

/// By default the quorums are open to all users
impl Default for Membership {
	fn default() -> Self {
		Membership::Everyone
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct Quorum<AccountId, BalanceOf> {
	/// Relayers
	pub relayers: Vec<AccountId>,
	/// Balances of relayers
	pub balances: Vec<BalanceOf>,
	/// Quorum creator (admin)
	pub creator: AccountId,
	/// Total pending rewards in fees to be distributed between relayers
	pub pending_rewards: BalanceOf,
	/// Minimum fee that the quorum accepts for jobs
	pub min_fee: BalanceOf,
	/// Who can make oracle job requests
	pub membership: Membership,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct Request<AccountId, BalanceOf> {
	/// User who made the request
	pub user: AccountId,
	/// Fee that has been paid
	pub fee: BalanceOf,
	/// Block number of request expiry
	pub valid_till: u64,
	// PQL address
	// relayer answers
	// min participation
	// validation rules
	// callback
	// callback status
}


pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
pub type QuorumOf<T> = Quorum<<T as system::Trait>::AccountId, BalanceOf<T>>;
pub type RequestOf<T> = Request<<T as system::Trait>::AccountId, BalanceOf<T>>;
pub type QuorumIndex = u32;
pub type RequestIndex = u32;

decl_storage! {
	trait Store for Module<T: Trait> as RelayerQuorums {
		/// Number of existing quorums. Also used as a hashmap index.
		QuorumCount get(fn quorum_count): QuorumIndex;

		/// Relayer quorums HashMap<quorum_id, quorum>
		Quorums get(fn quorums): map hasher(blake2_128_concat) QuorumIndex => QuorumOf<T>;

		/// Authorized users: DoubleHashMap<quorum_id, AccountId, ()>
		QuorumUsers get(fn quorum_users):
			double_map hasher(blake2_128_concat) QuorumIndex, hasher(blake2_128_concat) T::AccountId => ();

		/// Oracle Requests HashMap<request_id, request>
		Requests get(fn requests): map hasher(blake2_128_concat) RequestIndex => RequestOf<T>;

		/// Current max(request_id). Wraps around u64::max_value().
		MaxRequestId get(fn request_count): RequestIndex;

	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		// Balance = BalanceOf<T>,
		{
			QuorumCreated(QuorumIndex, AccountId),
			RelayerAdded(QuorumIndex, AccountId),
			RelayerRemoved(QuorumIndex, AccountId),
			UserAdded(QuorumIndex, AccountId),
			UserRemoved(QuorumIndex, AccountId),
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyRelayer,
		NotRelayer,
		RelayerLimitReached,
		InvalidQuorum,
		Unauthorized,
		ValueError,
		NotUser,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		type Error = Error<T>;

		/// Create a new quorum
		#[weight = 10_000]
		pub fn create(origin, min_fee: BalanceOf<T>, members_only: bool) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Safely update the quorum index
			let index = QuorumCount::get()
				.checked_add(1)
				.ok_or("quorum index overflow")?;
			QuorumCount::put(index);

			// TODO: add a minimum fee that is burned

			let membership = match members_only {
				true => Membership::Whitelist,
				false => Membership::Everyone,
			};

			// Create a new quorum
			<Quorums<T>>::insert(index, Quorum {
				relayers: vec![],
				balances: vec![],
				creator: who.clone(),
				pending_rewards: 0.into(),
				min_fee,
				membership,
			});

			Self::deposit_event(RawEvent::QuorumCreated(index, who));
			// TODO: update this to return quorum_id
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

		/// Creator adds a new user to the quorum
		#[weight = 10_000]
		pub fn add_user(origin, quorum_id: QuorumIndex, user: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let quorum = Self::find_quorum(quorum_id)?;
			ensure!(quorum.membership == Membership::Whitelist, Error::<T>::ValueError);

			// only quorum creator can add/remove new users (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			<QuorumUsers<T>>::insert(&quorum_id, &user, ());
			Self::deposit_event(RawEvent::UserAdded(quorum_id, user));
			Ok(())
		}

		/// Creator removes a user from the quorum
		#[weight = 10_000]
		pub fn remove_user(origin, quorum_id: QuorumIndex, user: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let quorum = Self::find_quorum(quorum_id)?;
			ensure!(quorum.membership == Membership::Whitelist, Error::<T>::ValueError);

			// only quorum creator can add/remove new users (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			<QuorumUsers<T>>::take(quorum_id, &user);
			Self::deposit_event(RawEvent::UserRemoved(quorum_id, user));
			Ok(())
		}

		/// Creator removes a user from the quorum
		#[weight = 10_000]
		pub fn request(origin, quorum_id: QuorumIndex) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let quorum = Self::find_quorum(quorum_id)?;

			// check if the user is allowed to submit a request to this quorum
			if quorum.membership == Membership::Whitelist {
				ensure!(<QuorumUsers<T>>::contains_key(quorum_id, &user), Error::<T>::NotUser);
			}

			// check valid period

			// pay the fee

			// store the request

			// emit the event
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

	/// Check if the user is a quorum user
	pub fn is_quorum_user(quorum_id: QuorumIndex, user: T::AccountId) -> bool {
		<QuorumUsers<T>>::contains_key(quorum_id, &user)
	}

	/// Distribute pending_rewards between quorum relayers
	fn _distribute_pending_rewards() {
		todo!();
	}
}
