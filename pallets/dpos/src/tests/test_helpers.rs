use frame_support::assert_ok;

use crate::{
	constants::{AccountId, Balance},
	mock::*,
	pallet,
	tests::ros,
	types::{
		CandidateDelegationSet, CandidateDetail, DelegationInfo, EpochSnapshot, ValidatorStatus,
	},
	BalanceOf, CandidateDelegators, CandidatePool, DelegateCountMap, DelegationInfos, Event,
	HoldReason,
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

	Dpos::do_try_state();
}

pub fn delegate_candidate(delegator: AccountId, candidate: AccountId, amount: Balance) {
	let before_balance = Balances::free_balance(delegator);
	let before_hold = Balances::total_balance_on_hold(&delegator);
	let before_delegation_info = DelegationInfos::<Test>::get(delegator, candidate);
	assert_ok!(Dpos::delegate_candidate(ros(delegator), candidate, amount));
	assert_eq!(DelegateCountMap::<Test>::get(delegator), 1);
	if let Some(value) = before_delegation_info {
		assert_eq!(
			DelegationInfos::<Test>::get(delegator, candidate),
			Some(DelegationInfo { amount: value.amount + amount })
		);
	}
	assert_eq!(Balances::free_balance(delegator), before_balance - amount);
	assert_eq!(Balances::total_balance_on_hold(&delegator), before_hold + amount);

	Dpos::do_try_state();
}

pub fn get_delegator_commission() -> u32 {
	<Test as pallet::Config>::DelegatorCommission::get()
}

pub fn get_author_commission() -> u32 {
	<Test as pallet::Config>::AuthorCommission::get()
}

pub fn get_genesis_epoch_snapshot(
	active_validator_set: CandidateDelegationSet<Test>,
) -> EpochSnapshot<Test> {
	let mut epoch_snapshot = EpochSnapshot::<Test>::default();
	for (active_validator_id, bond, _) in active_validator_set.to_vec().iter() {
		epoch_snapshot.add_validator(active_validator_id.clone(), bond.clone());
		for delegator in CandidateDelegators::<Test>::get(active_validator_id) {
			if let Some(delegation_info) =
				DelegationInfos::<Test>::get(&delegator, &active_validator_id)
			{
				epoch_snapshot.add_delegator(
					delegator,
					active_validator_id.clone(),
					delegation_info.amount,
				);
			}
		}
	}
	epoch_snapshot
}
