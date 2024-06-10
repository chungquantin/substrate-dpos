use crate::{BalanceOf, Config};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq, Clone)]
#[scale_info(skip_type_params(T))]
pub enum DelayActionType {
	CandidateRegistrationRemoved,
	CandidateUndelegated,
	EpochRewardPayoutSent,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct DelayActionRequest<T: Config> {
	// The block number where the request is created
	pub created_at: BlockNumberFor<T>,
	pub delay_for: BlockNumberFor<T>,
	pub amount: Option<BalanceOf<T>>,
}

pub trait DelayExecutor<T: Config> {
	fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult;
}
