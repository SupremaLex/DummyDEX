use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

const TOKEN_0_ID: u32 = 1;
const TOKEN_1_ID: u32 = 2;
const DECIMALS: u32 = 0;

#[test]
fn init_should_work_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(2), TOKEN_1_ID, 999, DECIMALS));
		assert_eq!(Erc20::get_balance(1, TOKEN_0_ID), 1000);
		assert_eq!(Erc20::get_total_supply(TOKEN_0_ID), 1000);
		assert_eq!(Erc20::get_decimals(TOKEN_0_ID), 0);
		assert_eq!(Erc20::get_balance(2, TOKEN_1_ID), 999);
		assert_eq!(Erc20::get_total_supply(TOKEN_1_ID), 999);
		assert_eq!(Erc20::get_decimals(TOKEN_1_ID), 0);
	});
}

#[test]
fn init_should_work_2() {
	new_test_ext().execute_with(|| {
		const DECIMALS_6: u32 = 6;
		const MILLION: u128 = (10 as u128).pow(DECIMALS_6);
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS_6));
		assert_ok!(Erc20::init(Origin::signed(2), TOKEN_1_ID, 999, DECIMALS_6));
		assert_eq!(Erc20::get_balance(1, TOKEN_0_ID), 1000 * MILLION);
		assert_eq!(Erc20::get_total_supply(TOKEN_0_ID), 1000 * MILLION);
		assert_eq!(Erc20::get_decimals(TOKEN_0_ID), DECIMALS_6);
		assert_eq!(Erc20::get_balance(2, TOKEN_1_ID), 999 * MILLION);
		assert_eq!(Erc20::get_total_supply(TOKEN_1_ID), 999 * MILLION);
		assert_eq!(Erc20::get_decimals(TOKEN_1_ID), DECIMALS_6);
	});
}

#[test]
fn init_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_1_ID, 1000, DECIMALS));
		assert_noop!(
			Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS),
			Error::<Test>::AlreadyInitialized
		);
	});
}

#[test]
fn transfer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::transfer(Origin::signed(1), TOKEN_0_ID, 2, 100));
		assert_eq!(Erc20::get_balance(1, TOKEN_0_ID), 900);
		assert_eq!(Erc20::get_balance(2, TOKEN_0_ID), 100);
	});
}

#[test]
fn transfer_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_noop!(
			Erc20::transfer(Origin::signed(1), TOKEN_0_ID, 1, 100),
			Error::<Test>::SelfTransfer
		);
	});
}

#[test]
fn transfer_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_noop!(
			Erc20::transfer(Origin::signed(1), TOKEN_0_ID, 2, 1001),
			Error::<Test>::InsufficientFunds
		);
	});
}

#[test]
fn transfer_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_noop!(
			Erc20::transfer(Origin::signed(1), TOKEN_0_ID, 2, 0),
			Error::<Test>::ZeroTransfer
		);
	});
}

#[test]
fn approve_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(1), TOKEN_0_ID, 2, 400));
		assert_eq!(Erc20::get_allowance((1, 2, TOKEN_0_ID)), 400);
	});
}

#[test]
fn transfer_from_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(1), TOKEN_0_ID, 2, 400));
		assert_eq!(Erc20::get_allowance((1, 2, TOKEN_0_ID)), 400);
		assert_ok!(Erc20::transfer_from(Origin::signed(3), TOKEN_0_ID, 1, 2, 200));
		assert_eq!(Erc20::get_balance(1, TOKEN_0_ID), 800);
		assert_eq!(Erc20::get_allowance((1, 2, TOKEN_0_ID)), 200);
		assert_eq!(Erc20::get_balance(2, TOKEN_0_ID), 200);
	});
}

#[test]
fn transfer_from_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(1), TOKEN_0_ID, 2, 400));
		assert_noop!(
			Erc20::transfer_from(Origin::signed(3), TOKEN_0_ID, 1, 2, 500),
			Error::<Test>::InsufficientAllowance
		);
	});
}

#[test]
fn transfer_from_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(1), TOKEN_0_ID, 2, 400));
		assert_ok!(Erc20::transfer(Origin::signed(1), TOKEN_0_ID, 3, 1000));
		assert_noop!(
			Erc20::transfer_from(Origin::signed(3), TOKEN_0_ID, 1, 2, 100),
			Error::<Test>::InsufficientFunds
		);
	});
}

#[test]
fn transfer_from_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc20::init(Origin::signed(1), TOKEN_0_ID, 1000, DECIMALS));
		assert_ok!(Erc20::approve(Origin::signed(1), TOKEN_0_ID, 2, 400));
		assert_noop!(
			Erc20::transfer_from(Origin::signed(3), TOKEN_0_ID, 1, 2, 0),
			Error::<Test>::ZeroTransfer
		);
	});
}
