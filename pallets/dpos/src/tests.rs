use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok};
use sp_runtime::TokenError;

// Short for runtime origin signed
fn ros(indx: u64) -> RuntimeOrigin {
	RuntimeOrigin::signed(indx)
}

#[cfg(test)]
mod register_as_candidate {
	use types::CandidateDetail;

	use super::*;

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
	fn should_ok_register_sucessfully() {
		use frame_support::traits::fungible::InspectHold;

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
	fn should_failed_duplicate_candidate() {
		let ext = TestExtBuilder::default();
		ext.build().execute_with(|| {
			assert_ok!(Dpos::register_as_candidate(ros(2), 15));
			assert_err!(
				Dpos::register_as_candidate(ros(2), 15),
				Error::<Test>::CandidateAlreadyExist
			)
		});
	}
}

#[cfg(test)]
mod delegate_candidate {
	use types::DelegationInfo;

	use super::*;

	#[test]
	fn should_failed_no_candidate_found() {
		let ext = TestExtBuilder::default();
		ext.build().execute_with(|| {
			assert_err!(
				Dpos::delegate_candidate(ros(ACCOUNT_3.id), ACCOUNT_1.id, 100),
				Error::<Test>::CandidateDoesNotExist
			);
		});
	}

	#[test]
	fn should_failed_over_range_delegate_amount() {
		let mut ext = TestExtBuilder::default();
		let candidate = ACCOUNT_3;
		ext.min_candidate_bond(20)
			.min_delegate_amount(101)
			.max_total_delegate_amount(300)
			.build()
			.execute_with(|| {
				assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));

				assert_err!(
					Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 100),
					Error::<Test>::BelowMinimumDelegateAmount
				);

				assert_err!(
					Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 350),
					Error::<Test>::OverMaximumTotalDelegateAmount
				);
			});
	}

	#[test]
	fn should_fail_delegate_too_many_candidates() {
		let mut ext = TestExtBuilder::default();
		ext.min_candidate_bond(5)
			.min_delegate_amount(101)
			.max_total_delegate_amount(300)
			.max_delegate_count(1)
			.build()
			.execute_with(|| {
				assert_ok!(Dpos::register_as_candidate(ros(ACCOUNT_2.id), 5));
				assert_ok!(Dpos::register_as_candidate(ros(ACCOUNT_3.id), 40));
				assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), ACCOUNT_2.id, 200));
				assert_err!(
					Dpos::delegate_candidate(ros(ACCOUNT_4.id), ACCOUNT_3.id, 100),
					Error::<Test>::TooManyCandidateDelegations
				);
			});
	}

	#[test]
	fn should_ok_delegate_candidate_successfully() {
		let mut ext = TestExtBuilder::default();
		let candidate = ACCOUNT_3;
		ext.min_candidate_bond(20)
			.min_delegate_amount(101)
			.max_total_delegate_amount(300)
			.build()
			.execute_with(|| {
				assert_ok!(Dpos::register_as_candidate(ros(candidate.id), 40));
				assert_ok!(Dpos::delegate_candidate(ros(ACCOUNT_4.id), candidate.id, 200));
				assert_eq!(DelegateCountMap::<Test>::get(ACCOUNT_4.id), 1);
				assert_eq!(
					DelegationInfos::<Test>::get(ACCOUNT_4.id, candidate.id),
					Some(DelegationInfo { amount: 200, last_modified_at: 1 })
				);

				System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
					candidate_id: candidate.id,
					delegated_by: ACCOUNT_4.id,
					amount: 200,
				}));
			});
	}
}
