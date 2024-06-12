use frame_support::assert_ok;

use crate::{
	constants::AccountId,
	mock::*,
	tests::ros,
	types::{CandidateDetail, ValidatorStatus},
	BalanceOf, CandidateDelegators, CandidatePool, Event, HoldReason,
};
use frame_support::traits::fungible::InspectHold;

pub fn register_new_candidate(
	candidate: AccountId,
	balance: BalanceOf<Test>,
	hold_amount: BalanceOf<Test>,
) {
	assert_ok!(Dpos::register_as_candidate(ros(candidate), hold_amount));
	assert_eq!(
		CandidatePool::<Test>::get(candidate),
		Some(CandidateDetail {
			bond: hold_amount,
			total_delegations: 0,
			status: ValidatorStatus::Online
		})
	);
	assert_eq!(Balances::free_balance(candidate), balance - hold_amount);
	assert_eq!(Balances::total_balance_on_hold(&candidate), hold_amount);
	assert_eq!(
		Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate),
		hold_amount
	);

	assert_eq!(CandidateDelegators::<Test>::get(&candidate), vec![]);

	// Assert that the correct event was deposited
	System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
		candidate_id: candidate,
		initial_bond: hold_amount,
	}));
}
