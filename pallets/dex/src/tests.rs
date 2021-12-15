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
const CHARLIE: u64 = 3;

fn init_tokens(total_supply: u128) {
	assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, total_supply, DECIMALS));
	assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, total_supply, DECIMALS));
}

fn approve(address: u64, amount: u128) {
	assert_ok!(Erc20::approve(Origin::signed(address), TOKEN_1_ID, POOL, amount * MIL));
	assert_ok!(Erc20::approve(Origin::signed(address), TOKEN_2_ID, POOL, amount * MIL));
}

fn init_dex(amount_0: u128, amount_1: u128) {
	assert_ok!(Dex::init(
		Origin::signed(ALICE),
		POOL,
		TOKEN_1_ID,
		amount_0 * MIL,
		TOKEN_2_ID,
		amount_1 * MIL
	));
}

#[test]
fn init_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply = 1000;
		init_tokens(total_supply);
		approve(ALICE,total_supply);
		init_dex(100, 1000);
		assert_eq!(Dex::get_pool_address(), Some(POOL));
		assert_eq!(Dex::get_first_token(), Some(TOKEN_1_ID));
		assert_eq!(Dex::get_second_token(), Some(TOKEN_2_ID));
		assert_eq!(Dex::get_liquidity(ALICE), 1100 * MIL);
		assert_eq!(Dex::get_liquidity(BOB), 0);
	});
}

#[test]
fn init_should_fail_0() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, TOKEN_1_ID, 100 * MIL, TOKEN_2_ID, 1000 * MIL),
			pallet_erc20::Error::<Test>::InsufficientAllowance
		);
	});
}

#[test]
fn init_should_fail_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 1000 * MIL));
		approve(ALICE,1000);
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
		init_tokens(1000);
		approve(ALICE,1000);
		assert_noop!(
			Dex::init(Origin::signed(ALICE), POOL, 3, 100 * MIL, 4, 1000 * MIL),
			pallet_erc20::Error::<Test>::Uninitilized
		);
	});
}

#[test]
fn buy_token_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(800000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(497487437)); // 0.99 * 100 * 1000 / (100 + 99) = 497.487437
		assert_ok!(Erc20::approve(Origin::signed(ALICE), TOKEN_2_ID, POOL, 1000 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_2_ID, 235 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(863292123)); // 0.99 * 235 * 200 / (502.512563 + 232.65) = 63.292123
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(262487437));
	});
}

#[test]
fn buy_token_should_fail() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		init_dex(100, 1000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100)); // 100%
		assert_noop!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 1 * MIL), Error::<Test>::NoLiquiudity);
	});
}

#[test]
fn deposit_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 300 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_2_ID, BOB, 300 * MIL));
		approve(BOB,1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 300000000);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(400000000));
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 375000000);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(225000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(450000000));
	});
}

#[test]
fn withdraw_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 100 * MIL));
		approve(BOB,1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL)); // 0.99 * 100 * 1000 / (100 + 99) = 497.487437
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 15)); // 15%
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(830000000)); // 800 + 200 * 0.15 = 830
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(75376884)); // 0 + 502.512563 * 0.15 = 575.376884
		assert_eq!(Dex::get_liquidity(ALICE), 935000000); // ~ 1100 * 0.85 = 935
	});
}

#[test]
fn withdraw_should_work_2() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 100 * MIL));
		approve(BOB,1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(502512563));
		assert_eq!(Dex::get_liquidity(ALICE), 1100 * MIL);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1000000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(502512563));
		assert_eq!(Dex::get_liquidity(ALICE), 0);
	});
}

#[test]
fn withdraw_should_fail() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 100 * MIL)); // 100x1000 => 200x500
		assert_eq!(Dex::get_liquidity(ALICE), 1100 * MIL);
		assert_noop!(Dex::withdraw(Origin::signed(ALICE), 0), Error::<Test>::WrongShareValue);
		assert_noop!(Dex::withdraw(Origin::signed(ALICE), 101), Error::<Test>::WrongShareValue);
	});
}

#[test]
fn deposit_withdraw_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 100 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_2_ID, BOB, 200 * MIL));
		approve(BOB,1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(400000000));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(300000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(267558529));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(700000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(732441471));
		assert_eq!(Dex::get_liquidity(ALICE), 300000000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 50)); // 50%
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(775000000)); // 700 + 300 * 0.5 * 0.5 = 775
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(799331103)); // 733.(3) + 266.(6) * 0.5 * 0.5 = 800
		assert_eq!(Dex::get_liquidity(ALICE), 150000000); // ~ 300 * 0.5 = 550
	});
}

#[test]
fn deposit_withdraw_should_work_2() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE,1000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 300 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_2_ID, BOB, 300 * MIL));
		approve(BOB,1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_ok!(Dex::deposit(Origin::signed(ALICE), TOKEN_2_ID, 100 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 375000000);
		assert_eq!(Dex::get_liquidity(ALICE), 450000000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100)); // 100%
		assert_eq!(Dex::get_liquidity(ALICE), 0);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(125000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(250000000));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(700000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(700000000));
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100)); // 100%
		assert_eq!(Dex::get_liquidity(BOB), 0);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &BOB), Ok(300000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &BOB), Ok(300000000));
	});
}

#[test]
fn general_test() {
	new_test_ext().execute_with(|| {
		init_tokens(3000);
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, BOB, 500 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_2_ID, BOB, 500 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_1_ID, CHARLIE, 500 * MIL));
		assert_ok!(Erc20::transfer(Origin::signed(ALICE), TOKEN_2_ID, CHARLIE, 500 * MIL));
		approve(ALICE,1000);
		approve(BOB,500);
		approve(CHARLIE,500);
		init_dex(500, 1000); // 500x1000
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1500000000);

		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL)); //600x834.724541
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1500000000);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(600000000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(834724541));

		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1585939999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(635939999));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(884724541));

		assert_ok!(Dex::deposit(Origin::signed(CHARLIE), TOKEN_2_ID, 250 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 2015639998);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(815639998));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(1134724541));

		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 50));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1265639999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(512148304));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(712504598));

		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 500 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1265639999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(1012148304));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(362318062));

		assert_ok!(Dex::withdraw(Origin::signed(CHARLIE), 100));
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 750000002);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(599784482));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(214704456));

		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 2); // 0.000002
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(1));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1903276175));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(1987110935));
	});
}