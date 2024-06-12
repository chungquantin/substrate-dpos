use crate::{
	constants::{CANDIDATE_1, CANDIDATE_2},
	mock::*,
	tests::{ros, test_helpers},
	CandidatePool, Error, Event,
};
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};

#[test]
fn should_failed_bond_less_zero_value() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;
		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_noop!(
			Dpos::candidate_bond_less(ros(CANDIDATE_1.id), 0),
			Error::<Test>::InvalidZeroAmount
		);
	});
}

#[test]
fn should_failed_bond_less_all_remove_registration() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;
		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_ok!(Dpos::candidate_bond_less(ros(CANDIDATE_1.id), hold_amount));
		assert_eq!(CandidatePool::<Test>::get(CANDIDATE_1.id), None);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: CANDIDATE_1.id,
		}));
	});
}

#[test]
fn should_failed_bond_less_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;

		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_noop!(
			Dpos::candidate_bond_less(ros(CANDIDATE_2.id), 100),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_failed_bond_less_more_than_hold_amount() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;

		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_noop!(
			Dpos::candidate_bond_less(ros(CANDIDATE_1.id), hold_amount + 200),
			Error::<Test>::InvalidMinimumCandidateBond
		);
	});
}

#[test]
fn should_failed_bond_less_below_threshold() {
	let mut ext = TestExtBuilder::default();
	ext.min_candidate_bond(101).genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 200;
		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_noop!(
			Dpos::candidate_bond_less(ros(CANDIDATE_1.id), 100),
			Error::<Test>::BelowMinimumCandidateBond
		);
	});
}

#[test]
fn should_ok_bond_less_successful() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 300;
		test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, hold_amount);

		assert_ok!(Dpos::candidate_bond_less(ros(CANDIDATE_1.id), 100));

		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateLessBondStaked {
			candidate_id: CANDIDATE_1.id,
			deducted_bond: 100,
		}));

		assert_eq!(CandidatePool::<Test>::get(CANDIDATE_1.id).unwrap().total(), hold_amount - 100);
		assert_eq!(Balances::free_balance(CANDIDATE_1.id), CANDIDATE_1.balance - hold_amount + 100);
		assert_eq!(Balances::total_balance_on_hold(&CANDIDATE_1.id), hold_amount - 100);
	});
}
