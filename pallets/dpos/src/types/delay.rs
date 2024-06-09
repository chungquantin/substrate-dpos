use crate::Config;
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::OriginFor;

pub trait DelayExecutor<T: Config> {
	fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult;
}
