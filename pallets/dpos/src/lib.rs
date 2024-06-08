#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::weights::WeightInfo;
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible, FindAuthor},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	pub trait ReportNewValidatorSet<AccountId> {
		fn report_new_validator_set(_new_set: Vec<AccountId>) {}
	}

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_authorship::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// The maximum number of authorities that the pallet can hold.
		#[pallet::constant]
		type MaxValidators: Get<u32>;

		/// The minimum number of stake that the candidate need to provide
		#[pallet::constant]
		type MinCandidateStake: Get<u32>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type CandidateDetailMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BlockNumberFor<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// We usually use passive tense for events.
		SomethingStored { something: u32, who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyValidators,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delegate_candidate(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Delegate tokens to the candidate - User tokens will be locked");
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_undelegate(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Unstaking from the delegates or unstaking bond (scheduled)")
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn register_as_candidate(_origin: OriginFor<T>, _bond: BalanceOf<T>) -> DispatchResult {
			todo!("Register the validator as a candidate")
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_deregister_candidate(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Deregister the validator from the candidate set (scheduled)")
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_candidate_exit(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Candidate leave the candidate pools, delegators token will be unlocked");
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_candidate_exit(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Cancel the request to exit the candidate pool");
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult {
			todo!(
				"Distribute the reward back to the delegators and validator who produced the block"
			);
		}

		/// An example of directly updating the authorities into [`Config::ReportNewValidatorSet`].
		#[pallet::call_index(99)]
		#[pallet::weight(<T as Config>::WeightInfo::force_report_new_validators())]
		pub fn force_report_new_validators(
			origin: OriginFor<T>,
			new_set: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				(new_set.len() as u32) < T::MaxValidators::get(),
				Error::<T>::TooManyValidators
			);
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// A function to get you an account id for the current block author.
		pub fn find_author() -> Option<T::AccountId> {
			// If you want to see a realistic example of the `FindAuthor` interface, see
			// `pallet-authorship`.
			<T as pallet_authorship::Config>::FindAuthor::find_author::<'_, Vec<_>>(
				Default::default(),
			)
		}
	}
}
