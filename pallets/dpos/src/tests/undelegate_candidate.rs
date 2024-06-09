use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok, traits::fungible::InspectHold};
use tests::ros;
use types::{CandidateDetail, CandidateRegitrationRequest, DelegationInfo};

#[test]
fn should_failed_no_candidate_delegation_found() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		assert_err!(
			Dpos::undelegate_candidate(ros(ACCOUNT_3.id), ACCOUNT_1.id, 100),
			Error::<Test>::DelegationDoesNotExist
		);
	});
}

#[test]
fn should_ok_delegate_candidate_successfully() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.min_candidate_bond(20).min_delegate_amount(101).build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
		);
		assert_eq!(
			*CandidateRegistrations::<Test>::get().first().unwrap(),
			CandidateRegitrationRequest { bond: 40, request_by: candidate.id }
		);

		TestExtBuilder::run_to_block(5);

		assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
		assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
		assert_eq!(
			DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
			Some(DelegationInfo { amount: 200, last_modified_at: 5 })
		);
		assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200);
		assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);

		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
			candidate_id: candidate.id,
			delegated_by: ACCOUNT_4.id,
			amount: 200,
		}));

		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 200, registered_at: 1 })
		);
	});
}

#[test]
fn should_failed_undelegate_over_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.min_candidate_bond(20).min_delegate_amount(101).build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
		);
		assert_eq!(
			*CandidateRegistrations::<Test>::get().first().unwrap(),
			CandidateRegitrationRequest { bond: 40, request_by: candidate.id }
		);

		TestExtBuilder::run_to_block(5);

		assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
		assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
		assert_eq!(
			DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
			Some(DelegationInfo { amount: 200, last_modified_at: 5 })
		);
		assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200);
		assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);

		assert_err!(
			Dpos::undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 300),
			Error::<Test>::InsufficientDelegatedAmount
		);

		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
			candidate_id: candidate.id,
			delegated_by: ACCOUNT_4.id,
			amount: 200,
		}));

		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 200, registered_at: 1 })
		);
	});
}

#[test]
fn should_ok_undelegate_all_amount() {
	let mut ext = TestExtBuilder::default();
	let candidate = ACCOUNT_3;
	ext.min_candidate_bond(20).min_delegate_amount(101).build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
		);
		assert_eq!(
			*CandidateRegistrations::<Test>::get().first().unwrap(),
			CandidateRegitrationRequest { bond: 40, request_by: candidate.id }
		);

		TestExtBuilder::run_to_block(5);

		assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
		assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
		assert_eq!(
			DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
			Some(DelegationInfo { amount: 200, last_modified_at: 5 })
		);
		assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200);
		assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
			candidate_id: candidate.id,
			delegated_by: ACCOUNT_4.id,
			amount: 200,
		}));

		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 200, registered_at: 1 })
		);

		assert_ok!(Dpos::undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
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
	ext.min_candidate_bond(20).min_delegate_amount(101).build().execute_with(|| {
		assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 0, registered_at: 1 })
		);
		assert_eq!(
			*CandidateRegistrations::<Test>::get().first().unwrap(),
			CandidateRegitrationRequest { bond: 40, request_by: candidate.id }
		);

		TestExtBuilder::run_to_block(5);

		assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
		assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
		assert_eq!(
			DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
			Some(DelegationInfo { amount: 200, last_modified_at: 5 })
		);
		assert_eq!(Balances::free_balance(ACCOUNT_4.id), ACCOUNT_4.balance - 200);
		assert_eq!(Balances::total_balance_on_hold(&ACCOUNT_4.id), 200);
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
			candidate_id: candidate.id,
			delegated_by: ACCOUNT_4.id,
			amount: 200,
		}));

		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 200, registered_at: 1 })
		);

		TestExtBuilder::run_to_block(10);

		assert_ok!(Dpos::undelegate_candidate(ros(ACCOUNT_4.id), candidate.id, 75));
		assert_eq!(
			DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
			Some(DelegationInfo { amount: 125, last_modified_at: 10 })
		);
		assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
		assert_eq!(CandidateDelegators::<Test>::get(candidate.id), vec![ACCOUNT_4.id]);
		assert_eq!(
			CandidateDetailMap::<Test>::get(candidate.id),
			Some(CandidateDetail { bond: 40, total_delegations: 125, registered_at: 1 })
		);
	});
}
