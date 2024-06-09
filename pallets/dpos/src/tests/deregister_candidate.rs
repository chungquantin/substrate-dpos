use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok, traits::fungible::InspectHold};

use tests::ros;
use types::{CandidateDetail, CandidateRegitrationRequest, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		assert_err!(
			Dpos::deregister_candidate(ros(ACCOUNT_1.id)),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_ok_deregister_sucessfully() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		let (succes_acc, bond) = ACCOUNT_2.to_tuple();
		let hold_amount = 15;

		TestExtBuilder::run_to_block(10);
		// Register first
		assert_ok!(Dpos::register_as_candidate(ros(succes_acc), hold_amount));
		assert_eq!(
			CandidateDetailMap::<Test>::get(succes_acc),
			Some(CandidateDetail { bond: hold_amount, registered_at: 10, total_delegations: 0 })
		);
		assert_eq!(Balances::free_balance(succes_acc), bond - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), hold_amount);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: 2,
			initial_bond: 15,
		}));
		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![CandidateRegitrationRequest { bond: hold_amount, request_by: succes_acc },]
		);

		// Then deregister
		assert_ok!(Dpos::deregister_candidate(ros(succes_acc)));
		assert_eq!(CandidateDetailMap::<Test>::get(succes_acc), None);
		assert_eq!(CandidateRegistrations::<Test>::get(), vec![]);

		assert_eq!(Balances::free_balance(succes_acc), bond);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), 0);
	});
}

#[test]
fn should_ok_deregister_multiple_candidates_sucessfully() {
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
			vec![CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_1 }]
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
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_1 },
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_2 }
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
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_1 },
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_2 },
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_3 }
			]
		);

		assert_eq!(Balances::free_balance(candidate_1), balance_1 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_1), hold_amount);

		assert_eq!(Balances::free_balance(candidate_2), balance_2 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_2), hold_amount);

		assert_eq!(Balances::free_balance(candidate_3), balance_3 - hold_amount);
		assert_eq!(Balances::total_balance_on_hold(&candidate_3), hold_amount);

		// Deregister candidate 1 from the candidate pool
		assert_ok!(Dpos::deregister_candidate(ros(candidate_1)));
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_1), None);
		assert_ne!(CandidateDetailMap::<Test>::get(candidate_2), None);
		assert_ne!(CandidateDetailMap::<Test>::get(candidate_3), None);
		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_2 },
				CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_3 }
			]
		);

		// Deregister candidate 3 from the candidate pool
		assert_ok!(Dpos::deregister_candidate(ros(candidate_3)));
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_1), None);
		assert_ne!(CandidateDetailMap::<Test>::get(candidate_2), None);
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_3), None);
		assert_eq!(
			CandidateRegistrations::<Test>::get(),
			vec![CandidateRegitrationRequest { bond: hold_amount, request_by: candidate_2 },]
		);

		// Deregister candidate 2 from the candidate pool
		assert_ok!(Dpos::deregister_candidate(ros(candidate_2)));
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_1), None);
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_2), None);
		assert_eq!(CandidateDetailMap::<Test>::get(candidate_3), None);
		assert_eq!(CandidateRegistrations::<Test>::get(), vec![]);
	});
}

#[test]
fn should_ok_deregister_with_delegations_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.min_candidate_bond(20)
		.min_delegate_amount(50)
		.max_delegate_count(3)
		.build()
		.execute_with(|| {
			let candidate = ACCOUNT_3;
			let (delegator_1, delegator_2, delegator_3) = (ACCOUNT_4, ACCOUNT_5, ACCOUNT_6);
			let (delegated_amount_1, delegated_amount_2, delegated_amount_3) = (100, 150, 150);

			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
			);

			TestExtBuilder::run_to_block(5);

			// Frst delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_1.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_1, last_modified_at: 5 })
			);
			assert_eq!(
				Balances::free_balance(delegator_1.id),
				delegator_1.balance - delegated_amount_1
			);
			assert_eq!(Balances::total_balance_on_hold(&delegator_1.id), delegated_amount_1);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: delegator_1.id,
				amount: delegated_amount_1,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 1);
			TestExtBuilder::run_to_block(10);

			// Second delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_2.id),
				candidate.id,
				delegated_amount_2
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_2.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_2.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_2, last_modified_at: 10 })
			);
			assert_eq!(
				Balances::free_balance(delegator_2.id),
				delegator_2.balance - delegated_amount_2
			);
			assert_eq!(Balances::total_balance_on_hold(&delegator_2.id), delegated_amount_2);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: delegator_2.id,
				amount: delegated_amount_2,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 2);
			TestExtBuilder::run_to_block(20);

			// Third delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_3.id),
				candidate.id,
				delegated_amount_3
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_3.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_3.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_3, last_modified_at: 20 })
			);
			assert_eq!(
				Balances::free_balance(delegator_3.id),
				delegator_3.balance - delegated_amount_3
			);
			assert_eq!(Balances::total_balance_on_hold(&delegator_3.id), delegated_amount_3);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: delegator_3.id,
				amount: delegated_amount_3,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 3);
			TestExtBuilder::run_to_block(100);

			// First delegator again
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_1.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_1 * 2, last_modified_at: 100 })
			);
			assert_eq!(
				Balances::free_balance(delegator_1.id),
				delegator_1.balance - delegated_amount_1 * 2
			);
			assert_eq!(Balances::total_balance_on_hold(&delegator_1.id), delegated_amount_1 * 2);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: delegator_1.id,
				amount: delegated_amount_1,
			}));

			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: delegated_amount_3 +
						delegated_amount_1 + delegated_amount_2 +
						delegated_amount_1,
					registered_at: 1
				})
			);
			assert_eq!(
				CandidateDelegators::<Test>::get(candidate.id),
				vec![delegator_1.id, delegator_2.id, delegator_3.id]
			);

			// Should clear all data of related delegations and the candidate
			assert_ok!(Dpos::deregister_candidate(ros(candidate.id)));
			assert_eq!(CandidateDetailMap::<Test>::get(candidate.id), None);
			assert_eq!(CandidateRegistrations::<Test>::get(), vec![]);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![]);
			assert_eq!(DelegationInfos::<Test>::get(delegator_1.id, candidate.id), None);
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 0);

			assert_eq!(Balances::free_balance(candidate.id), candidate.balance);
			assert_eq!(Balances::total_balance_on_hold(&candidate.id), 0);
		});
}
