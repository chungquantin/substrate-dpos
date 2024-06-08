use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok};
use sp_runtime::TokenError;

fn acc_signed(indx: u64) -> RuntimeOrigin {
	RuntimeOrigin::signed(indx)
}

#[test]
fn should_failed_register_as_candidate() {
	let ext = TestExtBuilder::default();
	ext.build().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Attemp to register as candidate without enough fund in the account
		assert_err!(Dpos::register_as_candidate(acc_signed(1), 500), TokenError::FundsUnavailable);
		assert_err!(
			Dpos::register_as_candidate(acc_signed(1), 5),
			Error::<Test>::BelowMinimumCandidateBond
		);
		assert_ok!(Dpos::register_as_candidate(acc_signed(2), 15));
		// Assert that the correct event was deposited
		System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
			candidate_id: 2,
			initial_bond: 15,
		}));
	});
}
