use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

use crate::{BalanceOf, Config};

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct DelegationInfo<T: Config> {
	pub amount: BalanceOf<T>,
	pub last_modified_at: BlockNumberFor<T>,
}

impl<T: Config> DelegationInfo<T> {
	pub fn default(amount: BalanceOf<T>) -> Self {
		Self { amount, last_modified_at: frame_system::Pallet::<T>::block_number() }
	}

	pub fn update_delegated_amount(&mut self, amount: BalanceOf<T>) {
		self.amount = amount;
		self.last_modified_at = frame_system::Pallet::<T>::block_number();
	}
}
