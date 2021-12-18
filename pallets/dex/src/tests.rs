use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Perbill;
use traits::MultiErc20;

const TOKEN_1_ID: u32 = 1;
const TOKEN_2_ID: u32 = 2;
const DECIMALS: u32 = 6;
const MIL: u128 = (10 as u128).pow(DECIMALS);

const POOL: u64 = 101;
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const MARRY: u64 = 5;
const JOHN: u64 = 6;

fn init_tokens(total_supply: u128) {
	assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_1_ID, total_supply, DECIMALS));
	assert_ok!(Erc20::init(Origin::signed(ALICE), TOKEN_2_ID, total_supply, DECIMALS));
}

fn approve(address: u64, amount: u128) {
	assert_ok!(Erc20::approve(Origin::signed(address), TOKEN_1_ID, POOL, amount * MIL));
	assert_ok!(Erc20::approve(Origin::signed(address), TOKEN_2_ID, POOL, amount * MIL));
}

fn transfer(from: u64, to: u64, amount: u128) {
	assert_ok!(Erc20::transfer(Origin::signed(from), TOKEN_1_ID, to, amount * MIL));
	assert_ok!(Erc20::transfer(Origin::signed(from), TOKEN_2_ID, to, amount * MIL));
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
		approve(ALICE, total_supply);
		init_dex(100, 1000);
		assert_eq!(Dex::get_pool_address(), Some(POOL));
		assert_eq!(Dex::get_token_ids(), Some((TOKEN_1_ID, TOKEN_2_ID)));
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
		approve(ALICE, 1000);
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
		approve(ALICE, 1000);
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
		approve(ALICE, 1000);
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
		approve(ALICE, 1000);
		init_dex(100, 1000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100)); // 100%
		assert_noop!(
			Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 1 * MIL),
			Error::<Test>::NoLiquiudity
		);
	});
}

#[test]
fn deposit_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 300);
		approve(BOB, 1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 300_000_000);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(400_000_000));
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 375_000_000);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(225_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(450_000_000));
	});
}

#[test]
fn deposit_single_token_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 300);
		approve(BOB, 1000);
		init_dex(100, 100);
		assert_ok!(Dex::deposit_single_token(Origin::signed(BOB), TOKEN_1_ID, 10 * MIL));
		// due to token swap(and fee) the liquidity of BOB is 9.526565 instead of 10, in case of deposit of 5 tokens
		assert_eq!(Dex::get_liquidity(BOB), 9_526_565);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(110_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(100_000_000));
	});
}

#[test]
fn withdraw_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1100);
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 100);
		approve(BOB, 1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL)); // 0.99 * 100 * 1000 / (100 + 99) = 497.487437
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 15)); // 15%
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(930_000_000)); // 800 + 200 * 0.15 = 830
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(753_768_84)); // 0 + 502.512563 * 0.15 = 575.376884
		assert_eq!(Dex::get_liquidity(ALICE), 935_000_000); // ~ 1100 * 0.85 = 935
	});
}

#[test]
fn withdraw_should_work_2() {
	new_test_ext().execute_with(|| {
		init_tokens(1100);
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 100);
		approve(BOB, 1000);
		init_dex(100, 1000);
		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(502_512_563));
		assert_eq!(Dex::get_liquidity(ALICE), 1100 * MIL);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 90));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1080_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(452_261_307));
		assert_eq!(Dex::get_liquidity(ALICE), 110_000_000);
	});
}

#[test]
fn withdraw_single_token_should_work_1() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE, 1000);
		init_dex(100, 100);
		assert_eq!(Dex::get_liquidity(ALICE), 200 * MIL);
		// withdraw 50% of token_1 and buy token_1 for 50% of token_2 share
		assert_ok!(Dex::withdraw_single_token(Origin::signed(ALICE), TOKEN_1_ID, 50)); // 50 + 0.99 * 50 * 50 / (100 + 0.99 * 50) = 50 + 16.555183
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(334_448_17));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(100_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(966_555_183));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(900_000_000));
		assert_eq!(Dex::get_liquidity(ALICE), 100_000_000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(0)); // 33.444817
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1000_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(1000_000_000));
	});
}

#[test]
fn withdraw_single_token_should_work_2() {
	new_test_ext().execute_with(|| {
		init_tokens(200);
		approve(ALICE, 200);
		transfer(ALICE, BOB, 100);
		approve(BOB, 200);
		init_dex(100, 100);
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(200_000_000));
		assert_eq!(Dex::get_pool_share(&ALICE), Perbill::from_percent(50));
		assert_eq!(Dex::get_pool_share(&BOB), Perbill::from_percent(50));
		// withdraw 50% of token_1 and buy token_1 for 50% of token_2 share
		assert_ok!(Dex::withdraw_single_token(Origin::signed(ALICE), TOKEN_1_ID, 75)); // 75 + 0.99 * 75 * 125 / (200 + 0.99 * 75) = 75 + 33.842297
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(91_157_703)); // 91.157703
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(200_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(108_842_297));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(0));
		assert_eq!(Dex::get_pool_share(&ALICE), Perbill::from_percent(20));
		assert_eq!(Dex::get_pool_share(&BOB), Perbill::from_percent(80));
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100));
		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_eq!(Dex::get_pool_share(&ALICE), Perbill::from_percent(100));
	});
}

#[test]
fn withdraw_should_fail() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE, 1000);
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
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 200);
		approve(BOB, 1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(200_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(400_000_000));
		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 100 * MIL));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(300_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(267_558_529));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(600_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(732_441_471));
		assert_eq!(Dex::get_liquidity(ALICE), 300_000_000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 50)); // 50%
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(675_000_000)); // 600 + 300 * 0.5 * 0.5 = 775
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(799_331_103)); // 733.(3) + 266.(6) * 0.5 * 0.5 = 800
		assert_eq!(Dex::get_liquidity(ALICE), 150_000_000); // ~ 300 * 0.5 = 550
	});
}

#[test]
fn deposit_withdraw_should_work_2() {
	new_test_ext().execute_with(|| {
		init_tokens(1000);
		approve(ALICE, 1000);
		transfer(ALICE, BOB, 300);
		approve(BOB, 1000);
		init_dex(100, 200); // 100x200
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL));
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_ok!(Dex::deposit(Origin::signed(ALICE), TOKEN_2_ID, 100 * MIL));
		assert_eq!(Dex::get_liquidity(BOB), 375_000_000);
		assert_eq!(Dex::get_liquidity(ALICE), 450_000_000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100)); // 100%
		assert_eq!(Dex::get_liquidity(ALICE), 0);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(125_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(250_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(700_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(700_000_000));
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100)); // 100%
		assert_eq!(Dex::get_liquidity(BOB), 0);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &BOB), Ok(300_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &BOB), Ok(300_000_000));
	});
}

#[test]
fn general_test_1() {
	new_test_ext().execute_with(|| {
		init_tokens(3000);
		transfer(ALICE, BOB, 500);
		transfer(ALICE, CHARLIE, 500);
		approve(ALICE, 1000);
		approve(BOB, 500);
		approve(CHARLIE, 500);
		init_dex(500, 1000); // 500x1000
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1500_000_000);

		assert_ok!(Dex::buy_token(Origin::signed(BOB), TOKEN_1_ID, 100 * MIL)); //600x834.724541
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1500_0000_00);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(600_000_000));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(834_724_541));

		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_2_ID, 50 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1585_939_999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(635_939_999));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(884_724_541));

		assert_ok!(Dex::deposit(Origin::signed(CHARLIE), TOKEN_2_ID, 250 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 2015_639_998);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(815_639_998));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(1134_724_541));

		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 50));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1265_639_999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(512_148_304));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(712_504_598));

		assert_ok!(Dex::buy_token(Origin::signed(ALICE), TOKEN_1_ID, 500 * MIL));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 1265_639_999);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(1012_148_304));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(362_318_062));

		assert_ok!(Dex::withdraw(Origin::signed(CHARLIE), 100));
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 750_000_002);
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(599_784_482));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(214_704_456));

		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100));
		assert_eq!(Dex::get_total_liquidity().unwrap(), 2); // 0.000002
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(1));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(0));
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1903_276_175));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(1987_110_935));
	});
}

#[test]
fn incentivize_model_test() {
	new_test_ext().execute_with(|| {
		init_tokens(5000);
		transfer(ALICE, BOB, 500);
		transfer(ALICE, CHARLIE, 1000);
		transfer(ALICE, MARRY, 1000);
		transfer(ALICE, JOHN, 1000);
		approve(ALICE, 1000);
		approve(BOB, 1000);
		approve(CHARLIE, 1000);
		approve(MARRY, 1000);
		approve(JOHN, 1000);
		init_dex(1000, 1000);
		assert_ok!(Dex::deposit(Origin::signed(BOB), TOKEN_1_ID, 500 * MIL));

		assert_ok!(Dex::buy_token(Origin::signed(CHARLIE), TOKEN_1_ID, 250 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(MARRY), TOKEN_2_ID, 400 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(JOHN), TOKEN_1_ID, 100 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(CHARLIE), TOKEN_2_ID, 100 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(JOHN), TOKEN_2_ID, 500 * MIL));
		assert_ok!(Dex::buy_token(Origin::signed(MARRY), TOKEN_1_ID, 500 * MIL));

		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &POOL), Ok(1543_933_772));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &POOL), Ok(1472_913_599));

		assert_eq!(Dex::get_total_reward(), Ok(16_847_371)); // 16.847371
		assert_eq!(Dex::get_liquidity(ALICE), 2000_000_000);
		assert_ok!(Dex::withdraw(Origin::signed(ALICE), 100));
		// ALICE gained 11.231578 tokens as reward
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &ALICE), Ok(1529_289_180));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &ALICE), Ok(1481_942_398));
		assert_eq!(Dex::get_liquidity(BOB), 1000_000_000);
		assert_ok!(Dex::withdraw(Origin::signed(BOB), 100));
		// BOB gained 5.615793 tokens as reward
		assert_eq!(Erc20::balance_of(TOKEN_1_ID, &BOB), Ok(514_644_591));
		assert_eq!(Erc20::balance_of(TOKEN_2_ID, &BOB), Ok(490_971_200));
	});
}
