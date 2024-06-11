use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	sp_runtime::traits::{CheckedAdd, CheckedSub},
	traits::DefensiveSaturating,
};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::Zero;

use crate::{BalanceOf, Config};

use super::DispatchResultWithValue;

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub enum ValidatorStatus {
	Online,
	Offline,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct CandidateDetail<T: Config> {
	pub bond: BalanceOf<T>,
	pub total_delegations: BalanceOf<T>,
	pub status: ValidatorStatus,
}

impl<T: Config> CandidateDetail<T> {
	pub fn new(bond: BalanceOf<T>) -> Self {
		CandidateDetail { total_delegations: Zero::zero(), bond, status: ValidatorStatus::Online }
	}

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

	pub fn update_bond(&mut self, bond: BalanceOf<T>) {
		self.bond = bond;
	}

	pub fn total(&self) -> BalanceOf<T> {
		self.total_delegations.defensive_saturating_add(self.bond)
	}

	pub fn toggle_status(&mut self) {
		self.status = match self.status {
			ValidatorStatus::Online => ValidatorStatus::Offline,
			ValidatorStatus::Offline => ValidatorStatus::Online,
		}
	}
}

#[allow(type_alias_bounds)]
pub type ActiveValidatorSet<T: Config> = sp_std::vec::Vec<(T::AccountId, BalanceOf<T>)>;

#[allow(type_alias_bounds)]
pub type CandidateSet<T: Config> = sp_std::vec::Vec<(T::AccountId, BalanceOf<T>)>;
