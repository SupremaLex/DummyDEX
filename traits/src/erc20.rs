use codec::FullCodec;

use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
};

pub trait MultiErc20<AccountId> {
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

	fn init(
		who: &AccountId,
		token_id: &Self::TokenId,
		initial_supply: Self::Balance,
	) -> DispatchResult;

	fn total_supply(token_id: Self::TokenId) -> Result<Self::Balance, sp_runtime::DispatchError>;

	fn balance_of(
		token_id: Self::TokenId,
		account: &AccountId,
	) -> Result<Self::Balance, sp_runtime::DispatchError>;

	fn allowance(
		token_id: Self::TokenId,
		owner: AccountId,
		spender: AccountId,
	) -> Result<Self::Balance, sp_runtime::DispatchError>;

	fn transfer(
		token_id: &Self::TokenId,
		owner: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		Self::transfer_from_to(&token_id, &owner, &to, amount)?;
		Ok(())
	}

	fn transfer_from(
		token_id: &Self::TokenId,
		owner: &AccountId,
		spender: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		Self::transfer_from_to(&token_id, &owner, &spender, amount)?;
		Self::decrease_allowance(&token_id, &owner, &spender, amount)?;
		Ok(())
	}

	fn approve(
		token_id: &Self::TokenId,
		owner: &AccountId,
		spender: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		Self::increase_allowance(&token_id, &owner, &spender, amount)?;
		Ok(())
	}

	fn transfer_from_to(
		token_id: &Self::TokenId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn increase_allowance(
		token_id: &Self::TokenId,
		owner: &AccountId,
		spender: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn decrease_allowance(
		token_id: &Self::TokenId,
		owner: &AccountId,
		spender: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn token_initialized(token_id: &Self::TokenId) -> DispatchResult;

	fn token_uninitialized(token_id: &Self::TokenId) -> DispatchResult;

	fn is_initialized(token_id: &Self::TokenId) -> bool;
}
