#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Zero};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + Zero;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageValue<_, T::Balance>;

	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type Balances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_allowance)]
	pub(super) type Allowances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Approval(T::AccountId, T::AccountId, T::Balance),
		Transfer(T::AccountId, T::AccountId, T::Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientFunds,
		InsufficientAllowance,
		Overflow,
		AlreadyInitialized,
		SelfTransfer,
		ZeroTransfer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn init(origin: OriginFor<T>, initial_supply: T::Balance) -> DispatchResult {
			match <TotalSupply<T>>::get() {
				Some(_) => Err(<Error<T>>::AlreadyInitialized)?,
				None => {
					let sender = ensure_signed(origin)?;
					<Balances<T>>::insert(sender.clone(), initial_supply);
					<TotalSupply<T>>::put(initial_supply);
					Ok(())
				}
			}
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::transfer_from_to(&sender, &to, amount)?;
			Self::deposit_event(Event::Transfer(sender, to, amount));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer_from(
			origin: OriginFor<T>,
			owner: T::AccountId,
			spender: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			Self::decrease_allowance(&owner, &spender, amount)?;
			Self::transfer_from_to(&owner, &spender, amount)?;
			Self::deposit_event(Event::Transfer(owner, spender, amount));
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn approve(
			origin: OriginFor<T>,
			spender: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::increase_allowance(&sender, &spender, amount)?;
			Self::deposit_event(Event::Approval(sender, spender, amount));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transfer_from_to(
			from: &T::AccountId,
			to: &T::AccountId,
			amount: T::Balance,
		) -> Result<(), Error<T>> {
			ensure!(!amount.is_zero(), <Error<T>>::ZeroTransfer);
			ensure!(from != to, <Error<T>>::SelfTransfer);

			Balances::<T>::try_mutate(&from, |balance| -> Result<(), Error<T>> {
				let updated_sender_balance =
					balance.checked_sub(&amount).ok_or(Error::<T>::InsufficientFunds)?;
				*balance = updated_sender_balance;
				Ok(())
			})?;
			Balances::<T>::try_mutate(&to, |balance| -> Result<(), Error<T>> {
				let updated_to_balance =
					balance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*balance = updated_to_balance;
				Ok(())
			})?;
			Ok(())
		}

		fn increase_allowance(
			owner: &T::AccountId,
			spender: &T::AccountId,
			amount: T::Balance,
		) -> Result<(), Error<T>> {
			Allowances::<T>::try_mutate(owner, spender, |allowance| -> Result<(), Error<T>> {
				let updated_allowance =
					allowance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*allowance = updated_allowance;
				Ok(())
			})?;
			Ok(())
		}

		fn decrease_allowance(
			owner: &T::AccountId,
			spender: &T::AccountId,
			amount: T::Balance,
		) -> Result<(), Error<T>> {
			Allowances::<T>::try_mutate(owner, spender, |allowance| -> Result<(), Error<T>> {
				let updated_allowance =
					allowance.checked_sub(&amount).ok_or(Error::<T>::InsufficientAllowance)?;
				*allowance = updated_allowance;
				Ok(())
			})?;
			Ok(())
		}
	}
}
