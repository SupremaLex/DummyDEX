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
	use frame_support::{ensure, fail, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, Zero};
	use traits::MultiErc20;

	type BalanceOf<T> =
		<<T as Config>::Tokens as MultiErc20<<T as frame_system::Config>::AccountId>>::Balance;

	type TokenIdOf<T> =
		<<T as Config>::Tokens as MultiErc20<<T as frame_system::Config>::AccountId>>::TokenId;

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
	pub(super) type TotalLiquidity<T: Config> =
		StorageMap<_, Blake2_128Concat, TokenIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_liquidity)]
	pub(super) type Liquidity<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TokenIdOf<T>,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Initialized(T::AccountId, T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
		TokenBought(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Uninitilized,
		AlreadyInitialized,
		WrongInitialization,
		WrongTokenId,
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
			T::Tokens::transfer_from(&first_token_id,&sender,&pool_address, first_token_amount)?;
			T::Tokens::transfer_from(
				&second_token_id,
				&sender,
				&pool_address,
				second_token_amount,
			)?;
			TotalLiquidity::<T>::insert(first_token_id, first_token_amount);
			TotalLiquidity::<T>::insert(second_token_id, second_token_amount);
			Liquidity::<T>::insert(&sender, first_token_id, first_token_amount);
			Liquidity::<T>::insert(&sender, second_token_id, second_token_amount);
			FirstToken::<T>::put(first_token_id);
			SecondToken::<T>::put(second_token_id);
			PoolAddress::<T>::put(&pool_address);
			Self::deposit_event(Event::Initialized(sender, pool_address, first_token_id, first_token_amount, second_token_id, second_token_amount));
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
	}

	impl<T: Config> Pallet<T> {
		fn price(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Option<BalanceOf<T>> {
			input_amount
				.checked_mul(&output_reserve)
				.unwrap()
				.checked_div(
					&input_reserve.checked_add(&input_amount).unwrap()
				)
		}

		fn deposit() {
			unimplemented!();
		}
		fn withdraw() {
			unimplemented!();
		}

		fn initialized() -> Result<(), Error<T>> {
			ensure!(Self::is_initialized(), <Error<T>>::Uninitilized);
			Ok(())
		}

		fn uninitialized() -> Result<(), Error<T>> {
			ensure!(!Self::is_initialized(), <Error<T>>::AlreadyInitialized);
			Ok(())
		}

		fn is_initialized() -> bool {
			Self::get_pool_address().is_some()
		}
	}
}
