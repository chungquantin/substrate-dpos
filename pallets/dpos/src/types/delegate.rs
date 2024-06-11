use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

use crate::{BalanceOf, Config};

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct DelegationInfo<T: Config> {
	pub amount: BalanceOf<T>,
}

impl<T: Config> DelegationInfo<T> {
	pub fn default(amount: BalanceOf<T>) -> Self {
		Self { amount }
	}

	pub fn update_delegated_amount(&mut self, amount: BalanceOf<T>) {
		self.amount = amount;
	}
}
