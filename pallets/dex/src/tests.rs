use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use traits::MultiErc20;

const TOKEN_1_ID: u32 = 1;
const TOKEN_2_ID: u32 = 2;
const DECIMALS: u32 = 6;
const MIL: u128 = (10 as u128).pow(DECIMALS);

const POOL: u64 = 101;
const ALICE: u64 = 1;
const BOB: u64 = 2;

#[test]
fn init_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply = 1000;
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, total_supply, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, total_supply, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_1_ID, POOL, total_supply * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, total_supply * MIL));
		assert_ok!(Dex::init(
			Origin::signed(ALICE),
			POOL,
			TOKEN_1_ID,
			100 * MIL,
			TOKEN_2_ID,
			1000 * MIL
		));
		assert_eq!(Dex::get_pool_address(), Some(POOL));
		assert_eq!(Dex::get_first_token(), Some(TOKEN_1_ID));
		assert_eq!(Dex::get_second_token(), Some(TOKEN_2_ID));
		assert_eq!(Dex::get_total_liquidity(TOKEN_1_ID), 100 * MIL);
		assert_eq!(Dex::get_total_liquidity(TOKEN_2_ID), total_supply * MIL);
		assert_eq!(Dex::get_liquidity(ALICE, TOKEN_1_ID), 100 * MIL);
		assert_eq!(Dex::get_liquidity(ALICE, TOKEN_2_ID), total_supply * MIL);
		assert_eq!(Dex::get_liquidity(BOB, TOKEN_1_ID), 0);
		assert_eq!(Dex::get_liquidity(BOB, TOKEN_2_ID), 0);
	});
}

#[test]
fn init_should_fail_0() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, 1000, DECIMALS));
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, TOKEN_1_ID, 100 * MIL, TOKEN_2_ID, 1000 * MIL),
			pallet_erc20::Error::<Test>::InsufficientAllowance
		);
	});
}

#[test]
fn init_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, 1000, DECIMALS));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 1000 * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_1_ID, POOL, 1000 * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, TOKEN_1_ID, 100 * MIL, TOKEN_2_ID, 1000 * MIL),
			pallet_erc20::Error::<Test>::InsufficientFunds
		);
	});
}

#[test]
fn init_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, TOKEN_1_ID, 0, TOKEN_2_ID, 0),
			Error::<Test>::WrongInitialization
		);
	});
}

#[test]
fn init_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::init(Origin::signed(ALICE), 0, TOKEN_1_ID, 100, TOKEN_2_ID, 1000),
			Error::<Test>::WrongInitialization
		);
	});
}

#[test]
fn init_should_fail_4() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_1_ID, POOL, 1000 * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, 3, 100 * MIL, 4, 1000 * MIL),
			pallet_erc20::Error::<Test>::Uninitilized
		);
	});
}

#[test]
fn buy_token_should_work_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_1_ID, POOL, 1000 * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_ok!(Dex::init(
			Origin::signed(ALICE),
			POOL,
			TOKEN_1_ID,
			100 * MIL,
			TOKEN_2_ID,
			1000 * MIL
		));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(800000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(500000000)); // 100 * 1000 / (100 + 100) = 500
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_2_ID, 235 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(863945578)); // 234 * 200 / (500 + 235) ~ 63.945578
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(265000000));
	});
}

#[test]
fn buy_token_should_work_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_1_ID, POOL, 1000 * MIL));
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_ok!(Dex::init(
			Origin::signed(ALICE),
			POOL,
			TOKEN_1_ID,
			100 * MIL,
			TOKEN_2_ID,
			1000 * MIL
		));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 1 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(899000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(9900990)); // 1 * 1000 / (100 + 1) ~ 9.90099
	});
}
