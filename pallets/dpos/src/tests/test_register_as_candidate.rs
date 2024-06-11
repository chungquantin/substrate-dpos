use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use sp_runtime::TokenError;

use tests::ros;
use types::CandidateDetail;

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
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_1.id), 500));
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_2.id), 500));
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_3.id), 500));
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_4.id), 500));
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
		let (succes_acc, bond) = ACCOUNT_2.to_tuple();
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(succes_acc), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(succes_acc),
			Some(CandidateDetail {
				bond: hold_amount,
				total_delegations: 0,
				status: types::ValidatorStatus::Online
			})
		);

		assert_eq!(Balances::free_balance(succes_acc), bond - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), hold_amount);
		assert_eq!(
			Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &succes_acc),
			hold_amount
		);
		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: 2,
			initial_bond: 15,
		}));
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
		assert_ok!(Dpos::register_as_candidate(ros(candidate_1), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(candidate_1),
			Some(CandidateDetail {
				bond: hold_amount,
				total_delegations: 0,
				status: types::ValidatorStatus::Online
			})
		);

		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_1,
			initial_bond: hold_amount,
		}));
		assert_eq!(CandidatePool::<Test>::count(), 1);
		assert_ok!(Dpos::register_as_candidate(ros(candidate_2), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(candidate_2),
			Some(CandidateDetail {
				bond: hold_amount,
				total_delegations: 0,
				status: types::ValidatorStatus::Online
			})
		);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_2,
			initial_bond: hold_amount,
		}));

		assert_eq!(CandidatePool::<Test>::count(), 2);

		assert_ok!(Dpos::register_as_candidate(ros(candidate_3), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(candidate_3),
			Some(CandidateDetail {
				bond: hold_amount,
				total_delegations: 0,
				status: types::ValidatorStatus::Online
			})
		);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_3,
			initial_bond: hold_amount,
		}));

		assert_eq!(CandidatePool::<Test>::count(), 3);

		assert_eq!(Balances::free_balance(candidate_1), balance_1 - hold_amount);
		assert_eq!(
			Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate_1),
			hold_amount
		);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_1), vec![]);

		assert_eq!(Balances::free_balance(candidate_2), balance_2 - hold_amount);
		assert_eq!(
			Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate_2),
			hold_amount
		);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_2), vec![]);

		assert_eq!(Balances::free_balance(candidate_3), balance_3 - hold_amount);
		assert_eq!(
			Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate_3),
			hold_amount
		);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_2), vec![]);
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
