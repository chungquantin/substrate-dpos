use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use tests::{ros, test_helpers};
use types::{CandidateDetail, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.reward_distribution_disabled()
		.genesis_candidates(vec![])
		.build()
		.execute_with(|| {
			assert_noop!(
				Dpos::force_undelegate_candidate(
					RuntimeOrigin::root(),
					ACCOUNT_3.id,
					ACCOUNT_1.id,
					100
				),
				Error::<Test>::CandidateDoesNotExist
			);
		});
}

#[test]
fn should_failed_no_candidate_delegation_found() {
	let mut ext = TestExtBuilder::default();
	ext.reward_distribution_disabled()
		.genesis_candidates(vec![DEFAULT_ACTIVE_SET[0]])
		.build()
		.execute_with(|| {
			assert_noop!(
				Dpos::force_undelegate_candidate(
					RuntimeOrigin::root(),
					ACCOUNT_3.id,
					DEFAULT_ACTIVE_SET[0].0,
					100
				),
				Error::<Test>::DelegationDoesNotExist
			);
		});
}

#[test]
fn should_failed_deregister_funds_no_available() {
	let mut ext = TestExtBuilder::default();
	ext.reward_distribution_disabled()
		.min_delegate_amount(100)
		.build()
		.execute_with(|| {
			MaxDelegateCount::set(100);

			ext.run_to_block(1010);

			let delegated_amount = 101;
			let (candidate, _) = DEFAULT_ACTIVE_SET[0];
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), candidate, delegated_amount));

			assert_noop!(
				Dpos::force_undelegate_candidate(
					RuntimeOrigin::root(),
					ACCOUNT_6.id,
					candidate,
					delegated_amount - 1
				),
				Error::<Test>::BelowMinimumDelegateAmount
			);
		});
}

#[test]
fn should_failed_undelegate_over_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
			assert_noop!(
				Dpos::force_undelegate_candidate(
					RuntimeOrigin::root(),
					ACCOUNT_4.id,
					candidate.id,
					300
				),
				Error::<Test>::InvalidMinimumDelegateAmount
			);
		});
}

#[test]
fn should_ok_undelegate_all_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
			assert_ok!(Dpos::force_undelegate_candidate(
				RuntimeOrigin::root(),
				ACCOUNT_4.id,
				candidate.id,
				200
			));
			assert_eq!(DelegationInfos::<Test>::get(ACCOUNT_1.id, candidate.id), None);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_1.id), 0);
			assert_eq!(CandidateDelegators::<Test>::get(ACCOUNT_1.id), vec![]);
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 0,
					status: types::ValidatorStatus::Online
				})
			);
		});
}

#[test]
fn should_ok_undelegate_partial_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			ext.run_to_block(10);

			assert_ok!(Dpos::force_undelegate_candidate(
				RuntimeOrigin::root(),
				ACCOUNT_4.id,
				candidate.id,
				75
			));
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: 125 })
			);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![ACCOUNT_4.id]);
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 125,
					status: types::ValidatorStatus::Online
				})
			);
		});
}

#[test]
fn should_ok_multiple_undelegate_both_all_and_partial() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_5.id), candidate.id, 300));

			ext.run_to_block(10);

			// Undelegate ACCOUNT_4
			assert_ok!(Dpos::force_undelegate_candidate(
				RuntimeOrigin::root(),
				ACCOUNT_4.id,
				candidate.id,
				75
			));
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: 200 - 75 })
			);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(
				CandidateDelegators::<Test>::get(candidate.id),
				vec![ACCOUNT_4.id, ACCOUNT_5.id]
			);
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 500 - 75,
					status: types::ValidatorStatus::Online
				})
			);
			assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200 + 75);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200 - 75);
			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateUndelegated {
				candidate_id: candidate.id,
				delegator: ACCOUNT_4.id,
				amount: 75,
				left_delegated_amount: 200 - 75,
			}));

			// Undelegate ACCOUNT_5
			assert_ok!(Dpos::force_undelegate_candidate(
				RuntimeOrigin::root(),
				ACCOUNT_5.id,
				candidate.id,
				199
			));
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_5.id, candidate.id),
				Some(DelegationInfo { amount: 300 - 199 })
			);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_5.id), 1);
			assert_eq!(
				CandidateDelegators::<Test>::get(candidate.id),
				vec![ACCOUNT_4.id, ACCOUNT_5.id]
			);
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 500 - 75 - 199,
					status: types::ValidatorStatus::Online
				})
			);
			assert_eq!(Balances::free_balance(ACCOUNT_5.id), ACCOUNT_5.balance - 300 + 199);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_5.id), 300 - 199);
			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateUndelegated {
				candidate_id: candidate.id,
				delegator: ACCOUNT_5.id,
				amount: 199,
				left_delegated_amount: 300 - 199,
			}));

			// Undelegate ALL from ACCOUNT_5
			// We expect this will remove the account 5 from the delegation pool of the candidate
			assert_ok!(Dpos::force_undelegate_candidate(
				RuntimeOrigin::root(),
				ACCOUNT_5.id,
				candidate.id,
				101
			));
			assert_eq!(DelegationInfos::<Test>::get(ACCOUNT_5.id, candidate.id), None);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_5.id), 0);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![ACCOUNT_4.id]);
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 500 - 75 - 199 - 101,
					status: types::ValidatorStatus::Online
				})
			);
			assert_eq!(Balances::free_balance(ACCOUNT_5.id), ACCOUNT_5.balance - 300 + 199 + 101);
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_5.id), 300 - 199 - 101);
			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateUndelegated {
				candidate_id: candidate.id,
				delegator: ACCOUNT_5.id,
				amount: 101,
				left_delegated_amount: 300 - 199 - 101,
			}));
		});
}
