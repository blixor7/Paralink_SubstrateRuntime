#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	codec::{Decode, Encode},
	traits::{Currency, ReservableCurrency, ExistenceRequirement},
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	traits::{Zero},
	RuntimeDebug,
};

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// A maximum number of members.
/// When membership reaches this number, no new members may join.
pub const MAX_MEMBERS: usize = 32;

pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: ReservableCurrency<Self::AccountId>;
}


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct Quorum<AccountId, Balance> {
	pub members: Vec<AccountId>,
	pub balances: Vec<Balance>,
	pub creator: AccountId,
	pub pendingRewards: Balance,
}


pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
pub type QuorumOf<T> = Quorum<<T as system::Trait>::AccountId, <T as balances::Trait>::Balance>;
// pub type QuorumOf<T> = Quorum<<T as system::Trait>::AccountId, BalanceOf<T>>;
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
			MemberAdded(QuorumIndex, AccountId),
			MemberRemoved(QuorumIndex, AccountId),
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyMember,
		NotMember,
		MembershipLimitReached,
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
		fn create(origin) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO: add a minimum fee that is burned

			// Safely update the quorum index
			let index = QuorumCount::get()
				.checked_add(1)
				.ok_or("quorum index overflow")?;
			QuorumCount::put(index);

			// Create a new quorum
			<Quorums<T>>::insert(index, Quorum {
				members: vec![],
				balances: vec![],
				creator: who.clone(),
				pendingRewards: 0.into(),
			});

			Self::deposit_event(RawEvent::QuorumCreated(index, who));
			Ok(())
		}

		/// Creator adds a member to the membership set unless the max is reached
		#[weight = 10_000]
		pub fn add_member(origin, quorum_id: QuorumIndex, new_member: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Quorums<T>>::contains_key(&quorum_id), Error::<T>::InvalidQuorum);
			let mut quorum = <Quorums<T>>::get(&quorum_id);

			// only quorum creator can add new members (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			ensure!(quorum.members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

			// Avoid duplicates. Because the list is always ordered, we can
			// leverage the binary search which makes this check O(log n).
			match quorum.members.binary_search(&new_member) {
				// If the search succeeds, the caller is already a member, so just return
				Ok(_) => Err(Error::<T>::AlreadyMember.into()),
				// If the search fails, the caller is not a member and we learned the index where
				// they should be inserted
				Err(index) => {
					// TODO: trigger pending rewards distribution
					quorum.members.insert(index, new_member.clone());
					quorum.balances.insert(index, Zero::zero());
					// Upsert the quorum
					<Quorums<T>>::insert(&quorum_id, quorum);
					Self::deposit_event(RawEvent::MemberAdded(quorum_id, new_member));
					Ok(())
				}
			}
		}

		/// Creator removes a member.
		#[weight = 10_000]
		pub fn remove_member(origin, quorum_id: QuorumIndex, remove_member: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Quorums<T>>::contains_key(&quorum_id), Error::<T>::InvalidQuorum);
			let mut quorum = <Quorums<T>>::get(&quorum_id);

			// only quorum creator can add new members (for now)
			ensure!(who == quorum.creator, Error::<T>::Unauthorized);

			match quorum.members.binary_search(&remove_member) {
				// If the search succeeds, the caller is a member; remove it
				Ok(index) => {
					// TODO: trigger pending rewards distribution
					let balance = quorum.balances.get(index).unwrap();
					if *balance > T::Balance::from(0)  {
						// TODO: transfer the balance to the member
					}
					quorum.members.remove(index);
					quorum.balances.remove(index);
					<Quorums<T>>::insert(&quorum_id, quorum);
					Self::deposit_event(RawEvent::MemberRemoved(quorum_id, remove_member));
					Ok(())
				},
				Err(_) => Err(Error::<T>::NotMember.into()),
			}
		}

		/// Member leaves
		#[weight = 10_000]
		pub fn leave(origin, quorum_id: QuorumIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(<Quorums<T>>::contains_key(&quorum_id), Error::<T>::InvalidQuorum);
			let mut quorum = <Quorums<T>>::get(&quorum_id);

			// We have to find out if the member exists in the sorted vec, and, if so, where.
			match quorum.members.binary_search(&who) {
				// If the search succeeds, the caller is a member, so remove her
				Ok(index) => {
					// TODO: trigger pending rewards distribution
					let balance = quorum.balances.get(index).unwrap();
					if *balance > 0.into()  {
						T::Currency::deposit_into_existing(&who, balance.into())?;
					}
					quorum.members.remove(index);
					quorum.balances.remove(index);
					<Quorums<T>>::insert(&quorum_id, quorum);
					Self::deposit_event(RawEvent::MemberRemoved(quorum_id, who));
					Ok(())
				},
				Err(_) => Err(Error::<T>::NotMember.into()),
			}
		}

	}
}

// private helper functions
impl<T: Trait> Module<T> {

	/// Distribute pendingRewards between quorum members
	fn refresh_quorum_balances(quorum_id: QuorumIndex) {
		todo!();
	}
}
