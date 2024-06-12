use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use tests::{ros, test_helpers};
use types::{CandidateDetail, DelayActionRequest, DelayActionType, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		assert_noop!(
			Dpos::delay_undelegate_candidate(ros(ACCOUNT_3.id), ACCOUNT_1.id, 100),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_failed_no_candidate_delegation_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![DEFAULT_ACTIVE_SET[0]]).build().execute_with(|| {
		assert_noop!(
			Dpos::delay_undelegate_candidate(ros(ACCOUNT_3.id), DEFAULT_ACTIVE_SET[0].0, 100),
			Error::<Test>::DelegationDoesNotExist
		);
	});
}

#[test]
fn should_failed_undelegate_below_delegated_amount() {
	let mut ext = TestExtBuilder::default();
	ext.min_delegate_amount(100).build().execute_with(|| {
		MaxDelegateCount::set(100);

		ext.run_to_block(1010);

		let delegated_amount = 101;
		let (candidate, _) = DEFAULT_ACTIVE_SET[0];
		assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), candidate, delegated_amount));

		assert_ok!(Dpos::delay_undelegate_candidate(
			ros(ACCOUNT_6.id),
			candidate,
			delegated_amount - 1
		),);

		assert_eq!(
			DelayActionRequests::<Test>::get(ACCOUNT_6.id, DelayActionType::CandidateUndelegated),
			Some(DelayActionRequest {
				amount: Some(delegated_amount - 1),
				created_at: 1010,
				delay_for: <mock::Test as pallet::Config>::DelayUndelegateCandidate::get(),
				target: Some(candidate)
			})
		);

		ext.run_to_block_from(1010, TEST_BLOCKS_PER_EPOCH * 2);

		assert_noop!(
			Dpos::execute_undelegate_candidate(ros(ACCOUNT_6.id)),
			Error::<Test>::BelowMinimumDelegateAmount
		);
	});
}

#[test]
fn should_failed_undelegate_over_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.reward_distribution_disabled()
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 0,
					status: types::ValidatorStatus::Online
				})
			);
			assert_eq!(CandidatePool::<Test>::count(), 1);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 300),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				Some(DelayActionRequest {
					amount: Some(300),
					created_at: 5,
					delay_for: <mock::Test as pallet::Config>::DelayUndelegateCandidate::get(),
					target: Some(candidate.id)
				})
			);

			ext.run_to_block_from(5, TEST_BLOCKS_PER_EPOCH * 2);

			assert_noop!(
				Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)),
				Error::<Test>::InvalidMinimumDelegateAmount
			);
		});
}

#[test]
fn should_ok_undelegate_all_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.reward_distribution_disabled()
		.build()
		.execute_with(|| {
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

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				Some(DelayActionRequest {
					amount: Some(200),
					created_at: 5,
					delay_for: <mock::Test as pallet::Config>::DelayUndelegateCandidate::get(),
					target: Some(candidate.id)
				})
			);

			ext.run_to_block_from(5, TEST_BLOCKS_PER_EPOCH * 2);

			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)));

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
			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidatePool::<Test>::get(candidate.id),
				Some(CandidateDetail {
					bond: 40,
					total_delegations: 0,
					status: types::ValidatorStatus::Online
				})
			);
			assert_eq!(CandidatePool::<Test>::count(), 1);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			ext.run_to_block(10);

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				Some(DelayActionRequest {
					amount: Some(75),
					created_at: 10,
					delay_for: <mock::Test as pallet::Config>::DelayUndelegateCandidate::get(),
					target: Some(candidate.id)
				})
			);

			ext.run_to_block_from(15, TEST_BLOCKS_PER_EPOCH * 2);

			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)));

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
		.delay_undelegate_candidate(TEST_BLOCKS_PER_EPOCH)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			ext.run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_5.id), candidate.id, 300));

			// Undelegate ACCOUNT_4
			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75));
			let latest_block_height = ext.run_to_block_from(5, TEST_BLOCKS_PER_EPOCH * 2);
			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)));

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
			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_5.id), candidate.id, 199));
			let latest_block_height =
				ext.run_to_block_from(latest_block_height, TEST_BLOCKS_PER_EPOCH * 2);
			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_5.id)));

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
			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_5.id), candidate.id, 101));
			ext.run_to_block_from(latest_block_height, TEST_BLOCKS_PER_EPOCH * 2);
			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_5.id)));

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

/// Test should failed when trying to undelegate while there is another delay action request
#[test]
fn should_failed_undelegate_while_in_delay_duration() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);
			assert_eq!(CandidatePool::<Test>::count(), 1);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75),);

			// Duplicate undelegate request before the due date
			ext.run_to_block(15);

			assert_noop!(
				Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75),
				Error::<Test>::ActionIsStillInDelayDuration
			);

			// Duplicate undelegate request after the due date
			ext.run_to_block_from(15, TEST_BLOCKS_PER_EPOCH * 2);

			assert_noop!(
				Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75),
				Error::<Test>::ActionIsStillInDelayDuration
			);
		});
}

// Delay for duration should go to next epoch or counting the number of block numbers?
#[test]
fn should_ok_undelegate_before_the_due_date() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.reward_distribution_disabled()
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.delay_undelegate_candidate(TEST_BLOCKS_PER_EPOCH)
		.build()
		.execute_with(|| {
			test_helpers::register_new_candidate(candidate.id, candidate.balance, 40);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			// Undelegate ACCOUNT_4
			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75));

			ext.run_to_block(TEST_BLOCKS_PER_EPOCH - 1);
			assert_noop!(
				Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)),
				Error::<Test>::ActionIsStillInDelayDuration
			);

			ext.run_to_block(TEST_BLOCKS_PER_EPOCH + 1);
			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id)));
		});
}
