use crate::Config;
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::OriginFor;

pub trait DelayExecutor<T: Config> {
	fn execute_deregister_candidate(_origin: OriginFor<T>) -> DispatchResult;
	fn execute_undelegate(_origin: OriginFor<T>) -> DispatchResult;
	fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult;
}
