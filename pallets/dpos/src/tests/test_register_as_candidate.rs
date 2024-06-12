use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::TokenError;

use tests::{ros, test_helpers};

#[test]
fn should_failed_invalid_bond_amount() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		// Attemp to register as candidate without enough fund in the account
		assert_noop!(
			Dpos::register_as_candidate(ros(ACCOUNT_1.id), 500),
			TokenError::FundsUnavailable
		);
		assert_noop!(
			Dpos::register_as_candidate(ros(ACCOUNT_1.id), 5),
			Error::<Test>::BelowMinimumCandidateBond
		);
	});
}

#[test]
fn should_failed_too_many_candidates() {
	let mut ext = TestExtBuilder::default();
	ext.max_candidates(4).genesis_candidates(vec![]).build().execute_with(|| {
		// Attemp to register as candidate without enough fund in the account
		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, 500);
		test_helpers::register_new_candidate(CANDIDATE_2.id, CANDIDATE_2.balance, 500);
		test_helpers::register_new_candidate(CANDIDATE_3.id, CANDIDATE_3.balance, 500);
		test_helpers::register_new_candidate(CANDIDATE_4.id, CANDIDATE_4.balance, 500);
		assert_noop!(
			Dpos::register_as_candidate(ros(CANDIDATE_5.id), 500),
			Error::<Test>::TooManyValidators
		);
	});
}

#[test]
fn should_ok_register_single_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let (success_acc, balance) = CANDIDATE_1.to_tuple();
		let hold_amount = 15;
		test_helpers::register_new_candidate(success_acc, balance, hold_amount);
	});
}

#[test]
fn should_ok_get_invalid_candidate() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![])
		.min_candidate_bond(5)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(ACCOUNT_2.id, ACCOUNT_2.balance, 5);
			test_helpers::register_new_candidate(ACCOUNT_3.id, ACCOUNT_3.balance, 40);
			assert_eq!(CandidatePool::<Test>::get(ACCOUNT_4.id), None);
		});
}

#[test]
fn should_ok_register_multiple_candidates_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let (candidate_1, balance_1) = ACCOUNT_2.to_tuple();
		let (candidate_2, balance_2) = ACCOUNT_3.to_tuple();
		let (candidate_3, balance_3) = ACCOUNT_4.to_tuple();
		let hold_amount = 15;

		test_helpers::register_new_candidate(candidate_1, balance_1, hold_amount);
		assert_eq!(CandidatePool::<Test>::count(), 1);
		test_helpers::register_new_candidate(candidate_2, balance_2, hold_amount);
		assert_eq!(CandidatePool::<Test>::count(), 2);
		test_helpers::register_new_candidate(candidate_3, balance_3, hold_amount);
		assert_eq!(CandidatePool::<Test>::count(), 3);
	});
}

#[test]
fn should_failed_duplicate_candidate() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(2), 15));
		assert_noop!(Dpos::register_as_candidate(ros(2), 15), Error::<Test>::CandidateAlreadyExist);
	});
}
