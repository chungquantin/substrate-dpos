use crate::{mock::*, *};
use constants::*;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};

use tests::ros;
use types::{CandidateDetail, DelayActionRequest, DelayActionType, DelegationInfo};

#[test]
fn should_failed_no_candidate_found() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		assert_noop!(
			Dpos::delay_deregister_candidate(ros(ACCOUNT_1.id)),
			Error::<Test>::CandidateDoesNotExist
		);
	});
}

#[test]
fn should_ok_delay_deregister_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
		let (succes_acc, bond) = ACCOUNT_2.to_tuple();
		let hold_amount = 15;

		TestExtBuilder::run_to_block(10);
		// Register first
		assert_ok!(Dpos::register_as_candidate(ros(succes_acc), hold_amount));
		// Then schedule to deregister
		assert_ok!(Dpos::delay_deregister_candidate(ros(succes_acc)));

		// This does not deregister the candidate from the pool yet
		TestExtBuilder::run_to_block(HALF_EPOCH);

		assert_eq!(
			CandidateDetailMap::<Test>::get(succes_acc),
			Some(CandidateDetail { bond: hold_amount, registered_at: 10, total_delegations: 0 })
		);
		assert_eq!(
			DelayActionRequests::<Test>::get(succes_acc, DelayActionType::CandidateLeaved),
			vec![DelayActionRequest {
				amount: None,
				created_at: 10,
				delay_for: Dpos::delay_deregister_candidate_duration()
			}]
		);

		// We go the few other blocks and try to execute it again
		TestExtBuilder::run_to_block(TEST_BLOCKS_PER_EPOCH * 2);

		assert_ok!(Dpos::execute_deregister_candidate(ros(succes_acc)));

		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
			candidate_id: succes_acc,
		}));
		assert_eq!(CandidateDetailMap::<Test>::get(succes_acc), None);

		assert_eq!(Balances::free_balance(succes_acc), bond);
		assert_eq!(Balances::total_balance_on_hold(&succes_acc), 0);
	});
}

#[test]
fn should_ok_delay_deregister_all_candidates_sucessfully() {
	let mut ext = TestExtBuilder::default();
	ext.min_delegate_amount(100).build().execute_with(|| {
		MaxDelegateCount::set(100);

		TestExtBuilder::run_to_block(1010);

		let delegated_amount = 101;
		for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
			assert_ok!(Dpos::delay_deregister_candidate(ros(candidate)));

			assert_eq!(
				DelayActionRequests::<Test>::get(candidate, DelayActionType::CandidateLeaved),
				vec![DelayActionRequest {
					amount: None,
					created_at: 1010,
					delay_for: Dpos::delay_deregister_candidate_duration()
				}]
			);

			assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_6.id), candidate, delegated_amount));
			assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), (indx + 1) as u32);
			assert_eq!(
				DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate),
				Some(DelegationInfo { amount: delegated_amount, last_modified_at: 1010 })
			);
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - delegated_amount * (indx + 1) as u128
			);
			assert_eq!(
				Balances::total_balance_on_hold(&ACCOUNT_6.id),
				delegated_amount * (indx + 1) as u128
			);
		}

		TestExtBuilder::run_to_block(1010 + TEST_BLOCKS_PER_EPOCH * 2);

		for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
			assert_ok!(Dpos::execute_deregister_candidate(ros(candidate)));

			System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistrationRemoved {
				candidate_id: candidate,
			}));
			assert_eq!(CandidateDetailMap::<Test>::get(candidate), None);
			assert_eq!(
				CandidateDetailMap::<Test>::count(),
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
			assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_6.id), total_delegated_amount);
		}
	});
}

// (FAILED) Delay deregister candidate before the scheduled date
// - Delegate tokens to all the candidates from one ACCOUNT
// - Create a delay action request to deregister candidates
// - Move to the block number < DELAY_DURATION
// - Expect error thrown
#[test]
fn should_failed_delay_deregister_candidates_behinds_due_date() {
	let mut ext = TestExtBuilder::default();
	ext.delay_deregister_candidate_duration(TEST_BLOCKS_PER_EPOCH)
		.min_delegate_amount(100)
		.build()
		.execute_with(|| {
			MaxDelegateCount::set(100);

			TestExtBuilder::run_to_block(1010);

			let delegated_amount = 101;
			for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
				assert_ok!(Dpos::delay_deregister_candidate(ros(candidate)));
				assert_eq!(
					DelayActionRequests::<Test>::get(candidate, DelayActionType::CandidateLeaved),
					vec![DelayActionRequest {
						amount: None,
						created_at: 1010,
						delay_for: Dpos::delay_deregister_candidate_duration()
					}]
				);

				assert_ok!(Dpos::delegate_candidate(
					ros(ACCOUNT_6.id),
					candidate,
					delegated_amount
				));
				assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), (indx + 1) as u32);
				assert_eq!(
					DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate),
					Some(DelegationInfo { amount: delegated_amount, last_modified_at: 1010 })
				);
				assert_eq!(
					Balances::free_balance(ACCOUNT_6.id),
					ACCOUNT_6.balance - delegated_amount * (indx + 1) as u128
				);
				assert_eq!(
					Balances::total_balance_on_hold(&ACCOUNT_6.id),
					delegated_amount * (indx + 1) as u128
				);
			}

			TestExtBuilder::run_to_block(1010 + HALF_EPOCH);

			for (_, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
				assert_noop!(
					Dpos::execute_deregister_candidate(ros(candidate)),
					Error::<Test>::ActionIsStillInDelayDuration
				);

				assert_eq!(CandidateDetailMap::<Test>::count(), DEFAULT_ACTIVE_SET.len() as u32);
				assert_eq!(
					DelayActionRequests::<Test>::get(candidate, DelayActionType::CandidateLeaved),
					vec![DelayActionRequest {
						amount: None,
						created_at: 1010,
						delay_for: Dpos::delay_deregister_candidate_duration()
					}]
				);
				assert_eq!(
					DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate),
					Some(DelegationInfo { amount: delegated_amount, last_modified_at: 1010 })
				);
				assert_eq!(
					DelegateCountMap::<Test>::get(ACCOUNT_6.id),
					DEFAULT_ACTIVE_SET.len() as u32
				);

				let total_delegated_amount = delegated_amount * (DEFAULT_ACTIVE_SET.len() as u128);
				assert_eq!(
					Balances::free_balance(ACCOUNT_6.id),
					ACCOUNT_6.balance - total_delegated_amount
				);
				assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_6.id), total_delegated_amount);
			}
		});
}

// (OK) Add test for cancel deregister_candidate delay action request
// - Must check that the state is not updated after the cancellation
// - Must check that the request no longer exist
#[test]
fn should_ok_cancel_deregister_candidate_requests() {
	let mut ext = TestExtBuilder::default();
	ext.delay_deregister_candidate_duration(TEST_BLOCKS_PER_EPOCH)
		.min_delegate_amount(100)
		.build()
		.execute_with(|| {
			MaxDelegateCount::set(100);

			TestExtBuilder::run_to_block(1010);

			let delegated_amount = 101;
			for (indx, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
				assert_ok!(Dpos::delay_deregister_candidate(ros(candidate)));
				assert_eq!(
					DelayActionRequests::<Test>::get(candidate, DelayActionType::CandidateLeaved),
					vec![DelayActionRequest {
						amount: None,
						created_at: 1010,
						delay_for: Dpos::delay_deregister_candidate_duration()
					}]
				);

				assert_ok!(Dpos::delegate_candidate(
					ros(ACCOUNT_6.id),
					candidate,
					delegated_amount
				));
				assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_6.id), (indx + 1) as u32);
				assert_eq!(
					DelegationInfos::<Test>::get(ACCOUNT_6.id, candidate),
					Some(DelegationInfo { amount: delegated_amount, last_modified_at: 1010 })
				);
				assert_eq!(
					Balances::free_balance(ACCOUNT_6.id),
					ACCOUNT_6.balance - delegated_amount * (indx + 1) as u128
				);
				assert_eq!(
					Balances::total_balance_on_hold(&ACCOUNT_6.id),
					delegated_amount * (indx + 1) as u128
				);
			}

			TestExtBuilder::run_to_block(1010 + HALF_EPOCH);

			for (_, (candidate, _)) in DEFAULT_ACTIVE_SET.clone().into_iter().enumerate() {
				assert_ok!(Dpos::cancel_deregister_candidate_request(ros(candidate)));
				assert_eq!(
					DelayActionRequests::<Test>::get(candidate, DelayActionType::CandidateLeaved),
					vec![]
				);
			}
		});
}

// (FAILED) Cancel not found delay action request
// - Provide wrong action type
// - Provide wrong request executor
#[test]
fn should_failed_cancel_not_found_delay_action_request() {
	let mut ext = TestExtBuilder::default();
	ext.genesis_candidates(vec![]).build().execute_with(|| {
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

		// Then schedule to deregister
		assert_ok!(Dpos::delay_deregister_candidate(ros(succes_acc)));

		// This does not deregister the candidate from the pool yet
		TestExtBuilder::run_to_block(HALF_EPOCH);

		assert_eq!(
			CandidateDetailMap::<Test>::get(succes_acc),
			Some(CandidateDetail { bond: hold_amount, registered_at: 10, total_delegations: 0 })
		);
		assert_eq!(
			DelayActionRequests::<Test>::get(succes_acc, DelayActionType::CandidateLeaved),
			vec![DelayActionRequest {
				amount: None,
				created_at: 10,
				delay_for: Dpos::delay_deregister_candidate_duration()
			}]
		);

		// We go the few other blocks and try to execute it again
		TestExtBuilder::run_to_block(TEST_BLOCKS_PER_EPOCH * 2);

		assert_noop!(
			Dpos::cancel_deregister_candidate_request(ros(ACCOUNT_3.id)),
			Error::<Test>::NoDelayActionRequestFound
		);
	});
}
