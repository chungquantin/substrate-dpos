use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_runtime::traits::{CheckedAdd, CheckedSub};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

use crate::{BalanceOf, Config};

use super::DispatchResultWithValue;

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct CandidateDetail<T: Config> {
	pub bond: BalanceOf<T>,
	pub total_delegations: BalanceOf<T>,
	pub registered_at: BlockNumberFor<T>,
}

impl<T: Config> CandidateDetail<T> {
	pub fn add_delegated_amount(
		&mut self,
		amount: BalanceOf<T>,
	) -> DispatchResultWithValue<BalanceOf<T>> {
		self.total_delegations = self.total_delegations.checked_add(&amount).expect("Overflow");
		Ok(self.total_delegations)
	}

	pub fn sub_delegated_amount(
		&mut self,
		amount: BalanceOf<T>,
	) -> DispatchResultWithValue<BalanceOf<T>> {
		self.total_delegations = self.total_delegations.checked_sub(&amount).expect("Overflow");
		Ok(self.total_delegations)
	}
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct CandidateRegitrationRequest<AccountId, Balance> {
	pub request_by: AccountId,
	pub bond: Balance,
}
