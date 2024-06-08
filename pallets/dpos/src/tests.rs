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
			// Go past genesis block so events get deposited
			System::set_block_number(1);
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
				Some(CandidateDetail { bond: hold_amount, registered_at: current_block_number })
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
