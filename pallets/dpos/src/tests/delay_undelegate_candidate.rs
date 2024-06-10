use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok};
use tests::ros;
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

		TestExtBuilder::run_to_block(1010);

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
			vec![DelayActionRequest {
				amount: Some(delegated_amount - 1),
				created_at: 1010,
				delay_for: Dpos::delay_undelegate_candidate_duration(),
				target: Some(candidate)
			}]
		);

		TestExtBuilder::run_to_block_from(1010, TEST_BLOCKS_PER_EPOCH * 2);

		assert_noop!(
			Dpos::execute_undelegate_candidate(ros(ACCOUNT_6.id), 0),
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
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
			);
			assert_eq!(CandidateDetailMap::<Test>::count(), 1);

			TestExtBuilder::run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 300),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				vec![DelayActionRequest {
					amount: Some(300),
					created_at: 5,
					delay_for: Dpos::delay_undelegate_candidate_duration(),
					target: Some(candidate.id)
				}]
			);

			TestExtBuilder::run_to_block_from(5, TEST_BLOCKS_PER_EPOCH * 2);

			assert_noop!(
				Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id), 0),
				Error::<Test>::InsufficientDelegatedAmount
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
		.build()
		.execute_with(|| {
			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
			);

			TestExtBuilder::run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				vec![DelayActionRequest {
					amount: Some(200),
					created_at: 5,
					delay_for: Dpos::delay_undelegate_candidate_duration(),
					target: Some(candidate.id)
				}]
			);

			TestExtBuilder::run_to_block_from(5, TEST_BLOCKS_PER_EPOCH * 2);

			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id), 0),);

			assert_eq!(DelegationInfos::<Test>::get(ACCOUNT_1.id, candidate.id), None);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_1.id), 0);
			assert_eq!(CandidateDelegators::<Test>::get(ACCOUNT_1.id), vec![]);
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
			);
		});
}

#[test]
fn should_ok_undelegate_partial_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.genesis_candidates(vec![])
		.min_candidate_bond(20)
		.min_delegate_amount(101)
		.build()
		.execute_with(|| {
			assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
			);
			assert_eq!(CandidateDetailMap::<Test>::count(), 1);

			TestExtBuilder::run_to_block(5);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));

			TestExtBuilder::run_to_block(10);

			assert_ok!(Dpos::delay_undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75),);

			assert_eq!(
				DelayActionRequests::<Test>::get(
					ACCOUNT_4.id,
					DelayActionType::CandidateUndelegated
				),
				vec![DelayActionRequest {
					amount: Some(75),
					created_at: 10,
					delay_for: Dpos::delay_undelegate_candidate_duration(),
					target: Some(candidate.id)
				}]
			);

			TestExtBuilder::run_to_block_from(15, TEST_BLOCKS_PER_EPOCH * 2);

			assert_ok!(Dpos::execute_undelegate_candidate(ros(ACCOUNT_4.id), 0));

			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
				Some(DelegationInfo { amount: 125, last_modified_at: 55 })
			);
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
			assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![ACCOUNT_4.id]);
			assert_eq!(
				CandidateDetailMap::<Test>::get(candidate.id),
				Some(CandidateDetail { bond: 40, total_delegations: 125, registered_at: 1 })
			);
		});
}

// TODO
#[test]
fn should_ok_multiple_undelegate_both_all_and_partial() {
	// Questions:
	// - Can we undelegate more while there is another delay action request in queue?
	// - Delay for duration should go to next epoch or counting the number of block numbers?
}
