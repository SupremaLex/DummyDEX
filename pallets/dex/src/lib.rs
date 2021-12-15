#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, ensure, fail, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
		Perbill
	};
	use traits::MultiErc20;

	type BalanceOf<T> =
		<<T as Config>::Tokens as MultiErc20<<T as frame_system::Config>::AccountId>>::Balance;

	type TokenIdOf<T> =
		<<T as Config>::Tokens as MultiErc20<<T as frame_system::Config>::AccountId>>::TokenId;

	const FEE: Perbill = Perbill::from_percent(99); // 1% per trade

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Tokens: MultiErc20<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_pool_address)]
	pub(super) type PoolAddress<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn get_first_token)]
	pub(super) type FirstToken<T: Config> = StorageValue<_, TokenIdOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_second_token)]
	pub(super) type SecondToken<T: Config> = StorageValue<_, TokenIdOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_liquidity)]
	pub(super) type TotalLiquidity<T: Config> = StorageValue<_, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_liquidity)]
	pub(super) type Liquidity<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Initialized(
			T::AccountId,
			T::AccountId,
			TokenIdOf<T>,
			BalanceOf<T>,
			TokenIdOf<T>,
			BalanceOf<T>,
		),
		TokenBought(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
		Deposited(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
		Withdrawed(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Uninitilized,
		AlreadyInitialized,
		WrongInitialization,
		WrongTokenId,
		Overflow,
		WrongShareValue,
		NoLiquiudity,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		#[transactional]
		pub fn init(
			origin: OriginFor<T>,
			pool_address: T::AccountId,
			first_token_id: TokenIdOf<T>,
			first_token_amount: BalanceOf<T>,
			second_token_id: TokenIdOf<T>,
			second_token_amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::uninitialized()?;
			ensure!(
				!first_token_amount.is_zero()
					&& !second_token_amount.is_zero()
					&& pool_address != T::AccountId::default(),
				Error::<T>::WrongInitialization
			);
			T::Tokens::transfer_from(&first_token_id, &sender, &pool_address, first_token_amount)?;
			T::Tokens::transfer_from(
				&second_token_id,
				&sender,
				&pool_address,
				second_token_amount,
			)?;
			let total_liquidity = first_token_amount.checked_add(&second_token_amount).unwrap();
			TotalLiquidity::<T>::put(total_liquidity);
			Liquidity::<T>::insert(&sender, total_liquidity);
			FirstToken::<T>::put(first_token_id);
			SecondToken::<T>::put(second_token_id);
			PoolAddress::<T>::put(&pool_address);
			Self::deposit_event(Event::Initialized(
				sender,
				pool_address,
				first_token_id,
				first_token_amount,
				second_token_id,
				second_token_amount,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn buy_token(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			Self::has_liquidity()?;
			let token_1 = Self::get_first_token().unwrap();
			let token_2 = Self::get_second_token().unwrap();
			let token_to_buy = match token_id {
				t1 if t1 == token_1 => token_2,
				t2 if t2 == token_2 => token_1,
				_ => fail!(Error::<T>::WrongTokenId),
			};
			let address = Self::get_pool_address().unwrap();
			let input_reserve = T::Tokens::balance_of(token_id, &address)?;
			let output_reserve = T::Tokens::balance_of(token_to_buy, &address)?;
			let bought = Self::price(amount, input_reserve, output_reserve).unwrap();

			T::Tokens::transfer_from(&token_id, &sender, &address, amount)?;
			T::Tokens::transfer_from_to(&token_to_buy, &address, &sender, bought)?;
			Self::deposit_event(Event::TokenBought(sender, token_id, amount, token_to_buy, bought));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn deposit(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			let token_1 = Self::get_first_token().unwrap();
			let token_2 = Self::get_second_token().unwrap();
			let second_token = match token_id {
				t1 if t1 == token_1 => token_2,
				t2 if t2 == token_2 => token_1,
				_ => fail!(Error::<T>::WrongTokenId),
			};
			let address = Self::get_pool_address().unwrap();
			let first_reserve = T::Tokens::balance_of(token_id, &address)?;
			let second_reserve = T::Tokens::balance_of(second_token, &address)?;
			let second_token_amount = amount
				.checked_mul(&second_reserve)
				.unwrap()
				.checked_div(&first_reserve)
				.unwrap();

			Self::increase_liquidity(&sender, amount.checked_add(&second_token_amount).unwrap())?;
			T::Tokens::transfer_from(&token_id, &sender, &address, amount)?;
			T::Tokens::transfer_from(&second_token, &sender, &address, second_token_amount)?;
			Self::deposit_event(Event::Deposited(
				sender,
				token_id,
				amount,
				second_token,
				second_token_amount,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn withdraw(origin: OriginFor<T>, share_percent: u32) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			ensure!(share_percent > 0 && share_percent <= 100, Error::<T>::WrongShareValue);
			let share_percent = Perbill::from_percent(share_percent);

			let address = Self::get_pool_address().unwrap();
			let token_1 = Self::get_first_token().unwrap();
			let token_2 = Self::get_second_token().unwrap();
			let first_reserve = T::Tokens::balance_of(token_1, &address)?;
			let second_reserve = T::Tokens::balance_of(token_2, &address)?;
			let total_liquidity = Self::get_total_liquidity().unwrap();

			let share_percent = share_percent
				* Perbill::from_rational(Self::get_liquidity(&sender), total_liquidity);
			let first_token_amount = share_percent * first_reserve;
			let second_token_amount = share_percent * second_reserve;

			Self::decrease_liquidity(&sender, share_percent * total_liquidity)?;
			T::Tokens::transfer_from_to(&token_1, &address, &sender, first_token_amount)?;
			T::Tokens::transfer_from_to(&token_2, &address, &sender, second_token_amount)?;
			Self::deposit_event(Event::Withdrawed(
				sender,
				token_1,
				first_token_amount,
				token_2,
				second_token_amount,
			));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn price(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Option<BalanceOf<T>> {
			let input_amount_with_fee = FEE * input_amount;
			input_amount_with_fee.checked_mul(&output_reserve)
				.unwrap()
				.checked_div(&input_reserve.checked_add(&input_amount_with_fee).unwrap())
		}

		fn initialized() -> Result<(), Error<T>> {
			ensure!(Self::is_initialized(), <Error<T>>::Uninitilized);
			Ok(())
		}

		fn uninitialized() -> Result<(), Error<T>> {
			ensure!(!Self::is_initialized(), <Error<T>>::AlreadyInitialized);
			Ok(())
		}

		fn has_liquidity() -> Result<(), Error<T>> {
			ensure!(Self::get_total_liquidity().unwrap() != BalanceOf::<T>::default(), <Error<T>>::NoLiquiudity);
			Ok(())
		}

		fn is_initialized() -> bool {
			Self::get_pool_address().is_some()
		}

		fn increase_liquidity(owner: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			TotalLiquidity::<T>::try_mutate(|liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.unwrap().checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = Some(updated_liquidity);
				Ok(())
			})?;
			Liquidity::<T>::try_mutate(owner, |liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = updated_liquidity;
				Ok(())
			})?;
			Ok(())
		}

		fn decrease_liquidity(owner: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			TotalLiquidity::<T>::try_mutate(|liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.unwrap().checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = Some(updated_liquidity);
				Ok(())
			})?;
			Liquidity::<T>::try_mutate(owner, |liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = updated_liquidity;
				Ok(())
			})?;
			Ok(())
		}
	}
}
