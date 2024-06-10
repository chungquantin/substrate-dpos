use crate::{BalanceOf, Config};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub enum DelayActionType<T: Config> {
	CandidateRegistrationRemoved,
	CandidateUndelegated(BalanceOf<T>),
	EpochRewardPayoutSent(BalanceOf<T>),
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct DelayActionRequest<T: Config> {
	// The block number where the request is created
	pub created_at: BlockNumberFor<T>,
	pub delay_for: BlockNumberFor<T>,
	pub request_by: T::AccountId,
	pub action_type: DelayActionType<T>,
}

pub trait DelayExecutor<T: Config> {
	fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult;
}
