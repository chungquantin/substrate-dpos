use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::Zero;

use crate::{BalanceOf, Config};

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct EpochInfo<T: Config> {
	current_indx: BlockNumberFor<T>,
	starting_from: BlockNumberFor<T>,
	duration: BlockNumberFor<T>,
	total_validators: u64,
	total_delegations: BalanceOf<T>,
}

impl<T> EpochInfo<T>
where
	T: Config,
{
	pub fn default(duration: BlockNumberFor<T>, max_active_validators: u64) -> Self {
		Self {
			current_indx: Zero::zero(),
			duration,
			starting_from: Zero::zero(),
			total_delegations: Zero::zero(),
			total_validators: max_active_validators,
		}
	}
}
