use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok, traits::fungible::InspectHold};
use sp_runtime::TokenError;

use tests::ros;
use types::{CandidateDetail, CandidateRegistrationRequest};

#[test]
fn should_failed_invalid_bond_amount() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		// Attemp to register as candidate without enough fund in the account
		assert_err!(
			Dpos::register_as_candidate(ros(ACCOUNT_1.id), 500),
			TokenError::FundsUnavailable
		);
		assert_err!(
			Dpos::register_as_candidate(ros(ACCOUNT_1.id), 5),
			Error::<Test>::BelowMinimumCandidateBond
		);
	});
}

#[test]
fn should_ok_register_single_sucessfully() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		let (succes_acc, bond) = ACCOUNT_2.to_tuple();
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(succes_acc), hold_amount));
		let current_block_number = System::block_number();
		assert_eq!(
			CandidateDetailMap::<Test>::get(succes_acc),
			Some(CandidateDetail {
				bond: hold_amount,
				registered_at: current_block_number,
				total_delegations: 0
			})
		);

		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![CandidateRegistrationRequest { bond: hold_amount, request_by: succes_acc }]
		);

		assert_eq!(Balances::free_balance(succes_acc), bond - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), hold_amount);
		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: 2,
			initial_bond: 15,
		}));
	});
}

#[test]
fn should_ok_register_multiple_candidates_sucessfully() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		let (candidate_1, balance_1) = ACCOUNT_2.to_tuple();
		let (candidate_2, balance_2) = ACCOUNT_3.to_tuple();
		let (candidate_3, balance_3) = ACCOUNT_4.to_tuple();
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(candidate_1), hold_amount));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate_1),
			Some(CandidateDetail {
				bond: hold_amount,
				registered_at: System::block_number(),
				total_delegations: 0
			})
		);

		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_1,
			initial_bond: hold_amount,
		}));

		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_1 }]
		);

		assert_ok!(Dpos::register_as_candidate(ros(candidate_2), hold_amount));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate_2),
			Some(CandidateDetail {
				bond: hold_amount,
				registered_at: System::block_number(),
				total_delegations: 0
			})
		);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_2,
			initial_bond: hold_amount,
		}));

		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![
				CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_1 },
				CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_2 }
			]
		);

		assert_ok!(Dpos::register_as_candidate(ros(candidate_3), hold_amount));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate_3),
			Some(CandidateDetail {
				bond: hold_amount,
				registered_at: System::block_number(),
				total_delegations: 0
			})
		);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: candidate_3,
			initial_bond: hold_amount,
		}));

		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![
				CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_1 },
				CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_2 },
				CandidateRegistrationRequest { bond: hold_amount, request_by: candidate_3 }
			]
		);

		assert_eq!(Balances::free_balance(candidate_1), balance_1 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_1), hold_amount);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_1), vec![]);

		assert_eq!(Balances::free_balance(candidate_2), balance_2 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_2), hold_amount);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_2), vec![]);

		assert_eq!(Balances::free_balance(candidate_3), balance_3 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_3), hold_amount);
		assert_eq!(CandidateDelegators::<Test>::get(&candidate_2), vec![]);
	});
}

#[test]
fn should_failed_duplicate_candidate() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(2), 15));
		assert_err!(Dpos::register_as_candidate(ros(2), 15), Error::<Test>::CandidateAlreadyExist)
	});
}
