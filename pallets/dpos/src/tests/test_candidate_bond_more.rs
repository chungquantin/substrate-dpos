use crate::{
	constants::{CANDIDATE_1, CANDIDATE_2},
	mock::*,
	tests::ros,
	types::CandidateDetail,
	CandidatePool, Error, Event,
};
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};

#[test]
fn should_failed_bond_more_zero_value() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_1.id), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(CANDIDATE_1.id),
			Some(CandidateDetail::new(hold_amount))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: CANDIDATE_1.id,
			initial_bond: 15,
		}));

		assert_noop!(
			Dpos::candidate_bond_more(ros(CANDIDATE_1.id), 0),
			Error::<Test>::InvalidZeroAmount
		);
	});
}

#[test]
fn should_failed_bond_more_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_1.id), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(CANDIDATE_1.id),
			Some(CandidateDetail::new(hold_amount))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: CANDIDATE_1.id,
			initial_bond: 15,
		}));

		assert_noop!(
			Dpos::candidate_bond_more(ros(CANDIDATE_2.id), 100),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_ok_bond_more_successful() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(CANDIDATE_1.id), hold_amount));
		assert_eq!(
			CandidatePool::<Test>::get(CANDIDATE_1.id),
			Some(CandidateDetail::new(hold_amount))
		);

		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: CANDIDATE_1.id,
			initial_bond: 15,
		}));

		assert_ok!(Dpos::candidate_bond_more(ros(CANDIDATE_1.id), 100),);

		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateMoreBondStaked {
			candidate_id: CANDIDATE_1.id,
			additional_bond: 100,
		}));

		assert_eq!(CandidatePool::<Test>::get(CANDIDATE_1.id).unwrap().total(), hold_amount + 100);
		assert_eq!(Balances::free_balance(CANDIDATE_1.id), CANDIDATE_1.balance - hold_amount - 100);
		assert_eq!(Balances::total_balance_on_hold(&CANDIDATE_1.id), hold_amount + 100);
	});
}
