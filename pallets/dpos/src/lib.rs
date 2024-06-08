#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::*;
	use crate::weights::WeightInfo;
	use frame_support::traits::fungible::MutateHold;
	use frame_support::traits::tokens::Precision;
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
			// You need to tell your trait bounds that the `Reason` is `RuntimeHoldReason`.
			+ fungible::hold::Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// The maximum number of authorities that the pallet can hold.
		#[pallet::constant]
		type MaxCandidates: Get<u32>;

		/// The minimum number of stake that the candidate need to provide to secure slot
		#[pallet::constant]
		type MinCandidateBond: Get<BalanceOf<Self>>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
		/// thanks to the `#[runtime]` macro.
		type RuntimeHoldReason: From<HoldReason>;
	}

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	pub type CandidateDetailMap<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		CandidateDetail<BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	pub type CandidateRegistrations<T: Config> = StorageValue<
		_,
		BoundedVec<
			CandidateRegitrationRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			<T as Config>::MaxCandidates,
		>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CandidateRegistered { candidate_id: T::AccountId, initial_bond: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyValidators,
		InsufficientBalance,
		CandidateAlreadyExist,
		CandidateDoesNotExist,
		BelowMinimumCandidateBond,
		InvalidNumberOfKeysMismatch,
	}

	/// A reason for the pallet dpos placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// The Pallet has reserved it for registering the candidate to pool.
		#[codec(index = 0)]
		CandidateBondReserved,
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
		pub fn register_as_candidate(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			let validator = ensure_signed(origin)?;

			// Update the registration list of candidates
			let mut candidate_registrations = CandidateRegistrations::<T>::get();
			candidate_registrations
				.try_push(CandidateRegitrationRequest { bond, request_by: validator.clone() })
				.map_err(|_| Error::<T>::TooManyValidators)?;

			Self::hold_candidate_bond(&validator, bond)?;

			Self::deposit_event(Event::CandidateRegistered {
				candidate_id: validator,
				initial_bond: bond,
			});
			Ok(())
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
				(new_set.len() as u32) < T::MaxCandidates::get(),
				Error::<T>::TooManyValidators
			);
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn hold_candidate_bond(validator: &T::AccountId, bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond >= T::MinCandidateBond::get(), Error::<T>::BelowMinimumCandidateBond);
			// Only hold the funds of a user which has no holds already.
			ensure!(
				!CandidateDetailMap::<T>::contains_key(&validator),
				Error::<T>::CandidateAlreadyExist
			);
			T::NativeBalance::hold(&HoldReason::CandidateBondReserved.into(), &validator, bond)?;
			// Store the amount held in our local storage.
			let now = frame_system::Pallet::<T>::block_number();
			CandidateDetailMap::<T>::insert(
				&validator,
				CandidateDetail { bond, registered_at: now },
			);
			Ok(())
		}

		/// This function will release the held balance of some user.
		pub fn release_candidate_bonds(candidate: T::AccountId) -> DispatchResult {
			let candidate_detail = CandidateDetailMap::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?;
			// NOTE: I am NOT using `T::HoldAmount::get()`... Why is that important?
			T::NativeBalance::release(
				&HoldReason::CandidateBondReserved.into(),
				&candidate,
				candidate_detail.bond,
				Precision::BestEffort,
			)?;
			Ok(())
		}

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
