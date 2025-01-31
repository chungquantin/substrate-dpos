use crate::{mock::*, *};
use constants::*;
use frame::deps::frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use tests::{ros, test_helpers};
use types::{CandidateDetail, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		assert_noop!(
			Dpos::delegate_candidate(ros(ACCOUNT_3.id), ACCOUNT_1.id, 100),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_failed_over_range_delegate_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			assert_noop!(
				Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 100),
				Error::<Test>::BelowMinimumDelegateAmount
			);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 300));
		});
}

#[test]
fn should_fail_delegate_too_many_candidates() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![])
		.min_candidate_bond(5)
		.min_delegate_amount(90)
		.max_delegate_count(5)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, 5);
			test_helpers::register_new_candidate(CANDIDATE_2.id, CANDIDATE_2.balance, 40);
			test_helpers::register_new_candidate(CANDIDATE_3.id, CANDIDATE_3.balance, 5);
			test_helpers::register_new_candidate(CANDIDATE_4.id, CANDIDATE_4.balance, 40);
			test_helpers::register_new_candidate(CANDIDATE_5.id, CANDIDATE_5.balance, 40);
			test_helpers::register_new_candidate(CANDIDATE_6.id, CANDIDATE_6.balance, 40);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_1.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_2.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_3.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_4.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_5.id, 200));
			assert_noop!(
				Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_6.id, 100),
				Error::<Test>::TooManyCandidateDelegations
			);

			Dpos::do_try_state();
		});
}

#[test]
fn should_ok_delegate_candidate_successfully() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: 200 })
			);
			assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: ACCOUNT_4.id,
				amount: 200,
				total_delegated_amount: 200,
			}));

			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 200,
					status: types::ValidatorStatus::Online
				})
			);

			Dpos::do_try_state();
		});
}

#[test]
fn should_ok_one_delegator_one_candidate_successfully() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			let (delegated_amount_1, delegated_amount_2) = (200, 100);
			// Delegate the first time
			assert_ok!(Dpos::delegate_candidate(
				ros(ACCOUNT_4.id),
				candidate.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: 200 })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_4.id),
				ACCOUNT_4.balance - delegated_amount_1
			);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: ACCOUNT_4.id,
				amount: 200,
				total_delegated_amount: 200,
			}));

			ext.run_to_block(10);

			// Delegate the second time
			let sum_delegated_amount = delegated_amount_1 + delegated_amount_2;
			assert_ok!(Dpos::delegate_candidate(
				ros(ACCOUNT_4.id),
				candidate.id,
				delegated_amount_2
			));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: sum_delegated_amount })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_4.id),
				ACCOUNT_4.balance - sum_delegated_amount
			);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), sum_delegated_amount);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate.id,
				delegated_by: ACCOUNT_4.id,
				amount: delegated_amount_2,
				total_delegated_amount: sum_delegated_amount,
			}));

			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: sum_delegated_amount,
					status: types::ValidatorStatus::Online
				})
			);

			Dpos::do_try_state();
		});
}

#[test]
fn should_ok_one_delegator_multiple_candidates_successfully() {
	use frame::deps::frame_support::traits::fungible::InspectHold;
	let mut ext = TestExtBuilder::default();
	ext.reward_distribution_disabled()
		.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(50)
		.build()
		.execute_with(|| {
			let (candidate_1, candidate_2, candidate_3) = (ACCOUNT_3, ACCOUNT_4, ACCOUNT_5);
			let (delegated_amount_1, delegated_amount_2, delegated_amount_3) = (200, 100, 150);

			test_helpers::register_new_candidate(candidate_1.id, candidate_1.balance, 40);

			ext.run_to_block(5);

			// Delegate the first time
			assert_ok!(Dpos::delegate_candidate(
				ros(ACCOUNT_6.id),
				candidate_1.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate_1.id),
				Some(DelegationInfo { amount: delegated_amount_1 })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - delegated_amount_1
			);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_6.id), 200);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate_1.id,
				delegated_by: ACCOUNT_6.id,
				amount: delegated_amount_1,
				total_delegated_amount: delegated_amount_1,
			}));

			ext.run_to_block(10);

			// Delegate candidate 2
			test_helpers::register_new_candidate(candidate_2.id, candidate_2.balance, 70);
			assert_ok!(Dpos::delegate_candidate(
				ros(ACCOUNT_6.id),
				candidate_2.id,
				delegated_amount_2
			));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), 2);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate_2.id),
				Some(DelegationInfo { amount: delegated_amount_2 })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - delegated_amount_1 - delegated_amount_2
			);
			assert_eq!(
				Balances::total_balance_on_hold(&ACCOUNT_6.id),
				delegated_amount_1 + delegated_amount_2
			);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate_2.id,
				delegated_by: ACCOUNT_6.id,
				amount: delegated_amount_2,
				total_delegated_amount: delegated_amount_2,
			}));

			assert_eq!(
				CandidatePool::<Test>::get(candidate_2.id),
				Some(CandidateDetail {
					bond: 70,
					total_delegations: delegated_amount_2,
					status: types::ValidatorStatus::Online
				})
			);

			ext.run_to_block(100);

			// Delegate candidate 3
			test_helpers::register_new_candidate(candidate_3.id, candidate_3.balance, 70);
			assert_ok!(Dpos::delegate_candidate(
				ros(ACCOUNT_6.id),
				candidate_3.id,
				delegated_amount_3
			));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), 3);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate_3.id),
				Some(DelegationInfo { amount: delegated_amount_3 })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - delegated_amount_1 - delegated_amount_2 - delegated_amount_3
			);
			assert_eq!(
				Balances::total_balance_on_hold(&ACCOUNT_6.id),
				delegated_amount_1 + delegated_amount_2 + delegated_amount_3
			);

			assert_eq!(CandidateDelegators::<Test>::get(candidate_1.id).len(), 1);
			assert_eq!(CandidateDelegators::<Test>::get(candidate_2.id).len(), 1);
			assert_eq!(CandidateDelegators::<Test>::get(candidate_3.id).len(), 1);

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
				candidate_id: candidate_3.id,
				delegated_by: ACCOUNT_6.id,
				amount: delegated_amount_3,
				total_delegated_amount: delegated_amount_3,
			}));

			assert_eq!(
				CandidatePool::<Test>::get(candidate_3.id),
				Some(CandidateDetail {
					bond: 70,
					total_delegations: delegated_amount_3,
					status: types::ValidatorStatus::Online
				})
			);
		});
}

#[test]
fn should_ok_multiple_delegators_one_candidate_successfully() {
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

			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			// Frst delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 1);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![delegator_1.id]);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_1.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_1 })
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
				total_delegated_amount: delegated_amount_1,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 1);
			ext.run_to_block(10);

			// Second delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_2.id),
				candidate.id,
				delegated_amount_2
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_2.id), 1);
			assert_eq!(
				CandidateDelegators::<Test>::get(candidate.id),
				vec![delegator_1.id, delegator_2.id]
			);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_2.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_2 })
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
				total_delegated_amount: delegated_amount_1 + delegated_amount_2,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 2);
			ext.run_to_block(20);

			// Third delegator
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_3.id),
				candidate.id,
				delegated_amount_3
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_3.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_3.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_3 })
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
				total_delegated_amount: delegated_amount_1 +
					delegated_amount_2 + delegated_amount_3,
			}));

			assert_eq!(CandidateDelegators::<Test>::get(candidate.id).len(), 3);
			assert_eq!(
				CandidateDelegators::<Test>::get(candidate.id),
				vec![delegator_1.id, delegator_2.id, delegator_3.id]
			);
			ext.run_to_block(100);

			// First delegator again
			assert_ok!(Dpos::delegate_candidate(
				ros(delegator_1.id),
				candidate.id,
				delegated_amount_1
			));
			assert_eq!(DelegateCountMap::<Test>::get(delegator_1.id), 1);
			assert_eq!(
				DelegationInfos::<Test>::get(delegator_1.id, candidate.id),
				Some(DelegationInfo { amount: delegated_amount_1 * 2 })
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
				total_delegated_amount: delegated_amount_1 +
					delegated_amount_2 + delegated_amount_3 +
					delegated_amount_1,
			}));

			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: delegated_amount_3 +
						delegated_amount_1 + delegated_amount_2 +
						delegated_amount_1,
					status: types::ValidatorStatus::Online
				})
			);

			Dpos::do_try_state();
		});
}

// Make sure that the pallet can support direct delegation
#[test]
fn should_ok_direct_delegation() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![])
		.min_candidate_bond(5)
		.min_delegate_amount(90)
		.max_delegate_count(5)
		// This attribtue defines the direct delegation mode
		.max_delegate_count(1)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, 5);
			test_helpers::register_new_candidate(CANDIDATE_2.id, CANDIDATE_2.balance, 40);

			// With direct delegation, we can't delegate more than one candidate
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_1.id, 200));
			assert_noop!(
				Dpos::delegate_candidate(ros(ACCOUNT_6.id), CANDIDATE_2.id, 100),
				Error::<Test>::TooManyCandidateDelegations
			);

			Dpos::do_try_state();
		});
}
