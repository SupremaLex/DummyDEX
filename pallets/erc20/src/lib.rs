#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use codec::FullCodec;
	use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Zero};
	use sp_std::{
		cmp::{Eq, PartialEq},
		fmt::Debug,
	};
	use traits::MultiErc20;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TokenId: FullCodec
			+ Eq
			+ PartialEq
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ scale_info::TypeInfo;
		type Balance: AtLeast32BitUnsigned
			+ FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ scale_info::TypeInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_total_supply)]
	pub(super) type TotalSupply<T: Config> =
		StorageMap<_, Blake2_128Concat, T::TokenId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::TokenId,
		T::Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_allowance)]
	pub(super) type Allowances<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::TokenId>,
		),
		T::Balance,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Approval(T::AccountId, T::AccountId, T::TokenId, T::Balance),
		Transfer(T::AccountId, T::AccountId, T::TokenId, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		Uninitilized,
		InsufficientFunds,
		InsufficientAllowance,
		Overflow,
		AlreadyInitialized,
		SelfTransfer,
		ZeroTransfer,
		WrongInitialization,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn init(
			origin: OriginFor<T>,
			token_id: T::TokenId,
			initial_supply: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Self as MultiErc20<_>>::init(&sender, &token_id, initial_supply)?;
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			token_id: T::TokenId,
			to: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Self as MultiErc20<_>>::transfer(&token_id, &sender, &to, amount)?;
			Self::deposit_event(Event::Transfer(sender, to, token_id, amount));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token_id: T::TokenId,
			owner: T::AccountId,
			spender: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			<Self as MultiErc20<_>>::transfer_from(&token_id, &owner, &spender, amount)?;
			Self::deposit_event(Event::Transfer(owner, spender, token_id, amount));
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn approve(
			origin: OriginFor<T>,
			token_id: T::TokenId,
			spender: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Self as MultiErc20<_>>::approve(&token_id, &sender, &spender, amount)?;
			Self::deposit_event(Event::Approval(sender, spender, token_id, amount));
			Ok(())
		}
	}

	impl<T: Config> MultiErc20<T::AccountId> for Pallet<T> {
		type TokenId = T::TokenId;
		type Balance = T::Balance;

		fn init(
			who: &T::AccountId,
			token_id: &Self::TokenId,
			initial_supply: Self::Balance,
		) -> DispatchResult {
			Self::token_uninitialized(token_id)?;
			ensure!(!initial_supply.is_zero(), <Error<T>>::WrongInitialization);
			<Balances<T>>::insert(who, token_id, initial_supply);
			<TotalSupply<T>>::insert(token_id, initial_supply);
			Ok(())
		}

		fn total_supply(
			token_id: Self::TokenId,
		) -> Result<Self::Balance, sp_runtime::DispatchError> {
			Self::token_initialized(&token_id)?;
			Ok(Self::get_total_supply(token_id))
		}

		fn balance_of(
			token_id: Self::TokenId,
			account: &T::AccountId,
		) -> Result<Self::Balance, sp_runtime::DispatchError> {
			Self::token_initialized(&token_id)?;
			Ok(Self::get_balance(account, token_id))
		}

		fn allowance(
			token_id: Self::TokenId,
			owner: T::AccountId,
			spender: T::AccountId,
		) -> Result<Self::Balance, sp_runtime::DispatchError> {
			Self::token_initialized(&token_id)?;
			Ok(Self::get_allowance((owner, spender, token_id)))
		}

		fn transfer_from_to(
			token_id: &Self::TokenId,
			from: &T::AccountId,
			to: &T::AccountId,
			amount: Self::Balance,
		) -> DispatchResult {
			Self::token_initialized(token_id)?;
			ensure!(!amount.is_zero(), <Error<T>>::ZeroTransfer);
			ensure!(from != to, <Error<T>>::SelfTransfer);

			Balances::<T>::try_mutate(&from, &token_id, |balance| -> Result<(), Error<T>> {
				let updated_sender_balance =
					balance.checked_sub(&amount).ok_or(Error::<T>::InsufficientFunds)?;
				*balance = updated_sender_balance;
				Ok(())
			})?;
			Balances::<T>::try_mutate(&to, &token_id, |balance| -> Result<(), Error<T>> {
				let updated_to_balance =
					balance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*balance = updated_to_balance;
				Ok(())
			})?;
			Ok(())
		}

		fn increase_allowance(
			token_id: &Self::TokenId,
			owner: &T::AccountId,
			spender: &T::AccountId,
			amount: Self::Balance,
		) -> DispatchResult {
			Self::token_initialized(token_id)?;
			Allowances::<T>::try_mutate(
				(owner, spender, token_id),
				|allowance| -> Result<(), Error<T>> {
					let updated_sender_allowance =
						allowance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
					*allowance = updated_sender_allowance;
					Ok(())
				},
			)?;
			Ok(())
		}

		fn decrease_allowance(
			token_id: &Self::TokenId,
			owner: &T::AccountId,
			spender: &T::AccountId,
			amount: Self::Balance,
		) -> DispatchResult {
			Self::token_initialized(token_id)?;
			Allowances::<T>::try_mutate(
				(owner, spender, token_id),
				|allowance| -> Result<(), Error<T>> {
					let updated_sender_allowance =
						allowance.checked_sub(&amount).ok_or(Error::<T>::InsufficientAllowance)?;
					*allowance = updated_sender_allowance;
					Ok(())
				},
			)?;
			Ok(())
		}

		fn token_initialized(token_id: &Self::TokenId) -> DispatchResult {
			ensure!(Self::is_initialized(token_id), Error::<T>::Uninitilized);
			Ok(())
		}

		fn token_uninitialized(token_id: &Self::TokenId) -> DispatchResult {
			ensure!(!Self::is_initialized(token_id), Error::<T>::AlreadyInitialized);
			Ok(())
		}

		fn is_initialized(token_id: &Self::TokenId) -> bool {
			!Self::get_total_supply(&token_id).is_zero()
		}
	}
}
