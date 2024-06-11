use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};

use tests::ros;
use types::{CandidateDetail, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		assert_noop!(
			Dpos::force_deregister_candidate(RuntimeOrigin::root(), ACCOUNT_1.id),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_failed_no_delegation_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		assert_noop!(
			Dpos::force_deregister_candidate(RuntimeOrigin::root(), ACCOUNT_1.id),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_ok_deregister_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let (succes_acc, bond) = ACCOUNT_2.to_tuple();
		let hold_amount = 15;

		ext.run_to_block(10);
		// Register first
		assert_ok!(Dpos::register_as_candidate(ros(succes_acc), hold_amount));

		// Then deregister
		assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), succes_acc));
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: succes_acc,
		}));
		assert_eq!(CandidatePool::<Test>::get(succes_acc), None);
		assert_eq!(CandidatePool::<Test>::count(), 0);

		assert_eq!(Balances::free_balance(succes_acc), bond);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), 0);
	});
}

#[test]
fn should_ok_deregister_multiple_candidates_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let (candidate_1, _) = ACCOUNT_2.to_tuple();
		let (candidate_2, _) = ACCOUNT_3.to_tuple();
		let (candidate_3, _) = ACCOUNT_4.to_tuple();
		let hold_amount = 15;
		assert_ok!(Dpos::register_as_candidate(ros(candidate_1), hold_amount));
		assert_ok!(Dpos::register_as_candidate(ros(candidate_2), hold_amount));
		assert_ok!(Dpos::register_as_candidate(ros(candidate_3), hold_amount));

		// Deregister candidate 1 from the candidate pool
		assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), candidate_1));
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: candidate_1,
		}));
		assert_eq!(CandidatePool::<Test>::get(candidate_1), None);
		assert_ne!(CandidatePool::<Test>::get(candidate_2), None);
		assert_ne!(CandidatePool::<Test>::get(candidate_3), None);
		assert_eq!(CandidatePool::<Test>::count(), 2);

		// Deregister candidate 3 from the candidate pool
		assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), candidate_3));
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: candidate_3,
		}));
		assert_eq!(CandidatePool::<Test>::get(candidate_1), None);
		assert_ne!(CandidatePool::<Test>::get(candidate_2), None);
		assert_eq!(CandidatePool::<Test>::get(candidate_3), None);
		assert_eq!(CandidatePool::<Test>::count(), 1);

		// Deregister candidate 2 from the candidate pool
		assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), candidate_2));
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: candidate_2,
		}));
		assert_eq!(CandidatePool::<Test>::get(candidate_1), None);
		assert_eq!(CandidatePool::<Test>::get(candidate_2), None);
		assert_eq!(CandidatePool::<Test>::get(candidate_3), None);
		assert_eq!(CandidatePool::<Test>::count(), 0);
	});
}

#[test]
fn should_ok_deregister_with_delegations_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.reward_distribution_disabled()
		.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(50)
		.build()
		.execute_with(|| {
			let candidate = ACCOUNT_3;
			let (delegator_1, delegator_2, delegator_3) = (ACCOUNT_4, ACCOUNT_5, ACCOUNT_6);
			let (delegated_amount_1, delegated_amount_2, delegated_amount_3) = (100, 150, 150);

			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 0,
					status: types::ValidatorStatus::Online
				})
			);

			ext.run_to_block(5);

			// Frst delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));

			ext.run_to_block(10);

			// Second delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_2.id),
				candidate.id,
				delegated_amount_2
			));

			ext.run_to_block(20);

			// Third delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_3.id),
				candidate.id,
				delegated_amount_3
			));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 3);

			ext.run_to_block(100);

			// First delegator again
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));

			// Should clear all data of related delegations and the candidate
			assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), candidate.id));
			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
				candidate_id: candidate.id,
			}));
			assert_eq!(CandidatePool::<Test>::get(candidate.id), None);
			assert_eq!(CandidatePool::<Test>::count(), 0);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![]);
			assert_eq!(DelegationInfos::<Test>::get(delegator_1.id, candidate.id), None);
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 0);

			assert_eq!(Balances::free_balance(candidate.id), candidate.balance);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate.id),
				0
			);
		});
}

#[test]
fn should_ok_deregister_all_candidates_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.min_delegate_amount(100).build().execute_with(|| {
		MaxDelegateCount::set(100);

		ext.run_to_block(1010);

		let delegated_amount = 101;
		for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), candidate, delegated_amount));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), (indx + 1) as u32);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate),
				Some(DelegationInfo { amount: delegated_amount })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - delegated_amount * (indx + 1) as u128
			);
			assert_eq!(
				Balances::balance_on_hold(
					&HoldReason::DelegateAmountReserved.into(),
					&ACCOUNT_6.id
				),
				delegated_amount * (indx + 1) as u128
			);
		}

		for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
			assert_ok!(Dpos::force_deregister_candidate(RuntimeOrigin::root(), candidate));

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
				candidate_id: candidate,
			}));
			assert_eq!(CandidatePool::<Test>::get(candidate), None);
			assert_eq!(
				CandidatePool::<Test>::count(),
				(DEFAULT_ACTIVE_SET.len() - (indx + 1)) as u32
			);
			assert_eq!(CandidateDelegators::<Test>::get(candidate), vec![]);
			assert_eq!(DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate), None);
			assert_eq!(
				DelegateCountMap::<Test>::get(ACCOUNT_6.id),
				(DEFAULT_ACTIVE_SET.len() - (indx + 1)) as u32
			);

			let total_delegated_amount =
				delegated_amount * ((DEFAULT_ACTIVE_SET.len() - (indx + 1)) as u128);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - total_delegated_amount
			);
			assert_eq!(
				Balances::balance_on_hold(
					&HoldReason::DelegateAmountReserved.into(),
					&ACCOUNT_6.id
				),
				total_delegated_amount
			);
		}
	});
}
