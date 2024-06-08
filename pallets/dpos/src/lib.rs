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
	use frame_support::dispatch::DispatchResult;
	use frame_support::sp_runtime::traits::CheckedAdd;
	use frame_support::traits::fungible::MutateHold;
	use frame_support::Twox64Concat;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Zero,
		traits::{fungible, tokens::Precision, FindAuthor},
	};
	use frame_system::pallet_prelude::{OriginFor, *};
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
		#[pallet::storage]
		type MaxCandidates: Get<u32>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
		/// thanks to the `#[runtime]` macro.
		type RuntimeHoldReason: From<HoldReason>;
	}

	/// The minimum number of stake that the candidate need to provide to secure slot
	#[pallet::storage]
	#[pallet::getter(fn min_candidate_bond)]
	pub type MinCandidateBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// The maximum number of candidates that delegators can delegate to
	#[pallet::storage]
	#[pallet::getter(fn max_delegate_count)]
	pub type MaxDelegateCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// The minimum number of delegate amount that the delegator need to provide for one candidate
	#[pallet::storage]
	#[pallet::getter(fn min_delegate_amount)]
	pub type MinDelegateAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// The maximum number of total delegate amount that the delegator can delegate for one candidate
	#[pallet::storage]
	#[pallet::getter(fn max_total_delegate_amount)]
	pub type MaxTotalDelegateAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// We use a configurable constant `BlockNumber` to tell us when we should trigger the
	/// validator set change. The runtime developer should implement this to represent the time
	/// they want validators to change, but for your pallet, you just care about the block
	/// number.
	#[pallet::storage]
	#[pallet::getter(fn epoch_duration)]
	pub type EpochDuration<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type CandidateDetailMap<T: Config> = CountedStorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		CandidateDetail<BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery,
	>;

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	#[pallet::getter(fn candidate_regristration)]
	pub type CandidateRegistrations<T: Config> = StorageValue<
		_,
		BoundedVec<
			CandidateRegitrationRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			<T as Config>::MaxCandidates,
		>,
		ValueQuery,
	>;

	/// The maximum number of candidates that delegators can delegate to
	#[pallet::storage]
	#[pallet::getter(fn delegate_count_map)]
	pub type DelegateCountMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	#[pallet::getter(fn delegate_infos)]
	pub type DelegationInfos<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		T::AccountId,
		DelegationInfo<T>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub min_candidate_bond: BalanceOf<T>,
		pub max_delegate_count: u32,
		pub max_total_delegate_amount: BalanceOf<T>,
		pub min_delegate_amount: BalanceOf<T>,
		pub epoch_duration: BlockNumberFor<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			MinCandidateBond::<T>::put(self.min_candidate_bond);
			MaxDelegateCount::<T>::put(self.max_delegate_count);
			MaxTotalDelegateAmount::<T>::put(self.max_total_delegate_amount);
			MinDelegateAmount::<T>::put(self.min_delegate_amount);
			EpochDuration::<T>::put(self.epoch_duration);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CandidateRegistered {
			candidate_id: T::AccountId,
			initial_bond: BalanceOf<T>,
		},
		CandidateDelegated {
			candidate_id: T::AccountId,
			delegated_by: T::AccountId,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// This is a pretty lightweight check that we do EVERY block, but then tells us when an
			// Epoch has passed...
			if n % EpochDuration::<T>::get() == BlockNumberFor::<T>::zero() {
				// CHANGE VALIDATORS LOGIC
				// You cannot return an error here, so you have to be clever with your code...
			}

			// We return a default weight because we do not expect you to do weights for your
			// project... Except for extra credit...
			return Weight::default();
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyValidators,
		TooManyCandidateDelegations,
		CandidateAlreadyExist,
		CandidateDoesNotExist,
		OverMaximumTotalDelegateAmount,
		BelowMinimumDelegateAmount,
		BelowMinimumCandidateBond,
	}

	/// A reason for the pallet dpos placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// The Pallet has reserved it for registering the candidate to pool.
		#[codec(index = 0)]
		CandidateBondReserved,
		#[codec(index = 1)]
		DelegateAmountReserved,
	}

	impl<T: Config> DelayExecutor<T> for Pallet<T> {
		fn execute_deregister_candidate(_origin: OriginFor<T>) -> DispatchResult {
			unimplemented!();
		}

		fn execute_undelegate(_origin: OriginFor<T>) -> DispatchResult {
			unimplemented!();
		}

		fn execute_reward_payout(_origin: OriginFor<T>) -> DispatchResult {
			unimplemented!();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delegate_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(
				CandidateDetailMap::<T>::contains_key(&candidate),
				Error::<T>::CandidateDoesNotExist
			);

			let delegator = ensure_signed(origin)?;
			match DelegationInfos::<T>::try_get(&delegator, &candidate) {
				Ok(mut delegation_info) => {
					// Update the delegated amount of the existing delegation info
					let new_delegated_amount =
						delegation_info.amount.checked_add(&amount).expect("Overflow");
					Self::check_delegate_payload(&delegator, new_delegated_amount)?;
					delegation_info.amount = new_delegated_amount;
				},
				Err(_) => {
					// First time delegate to this candidate
					Self::check_delegate_payload(&delegator, amount)?;
					let now = frame_system::Pallet::<T>::block_number();
					let delegate_count = DelegateCountMap::<T>::get(&delegator);
					DelegateCountMap::<T>::set(&delegator, delegate_count + 1);
					DelegationInfos::<T>::insert(
						&delegator,
						&candidate,
						DelegationInfo { amount, last_modified_at: now },
					);
				},
			};

			T::NativeBalance::hold(&HoldReason::DelegateAmountReserved.into(), &delegator, amount)?;

			Self::deposit_event(Event::CandidateDelegated {
				candidate_id: candidate,
				delegated_by: delegator,
				amount,
			});
			Ok(())
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

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_candidate_exit(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Candidate leave the candidate pools, delegators token will be unlocked");
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_candidate_exit(_origin: OriginFor<T>) -> DispatchResult {
			todo!("Cancel the request to exit the candidate pool");
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
		fn check_delegate_payload(
			delegator: &T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let (max_delegate_amount, min_delegate_amount, max_delegate_count) = (
				MaxTotalDelegateAmount::<T>::get(),
				MinDelegateAmount::<T>::get(),
				MaxDelegateCount::<T>::get(),
			);
			ensure!(
				DelegateCountMap::<T>::get(&delegator) + 1 <= max_delegate_count,
				Error::<T>::TooManyCandidateDelegations
			);
			ensure!(amount < max_delegate_amount, Error::<T>::OverMaximumTotalDelegateAmount);
			ensure!(amount >= min_delegate_amount, Error::<T>::BelowMinimumDelegateAmount);
			Ok(())
		}

		pub fn hold_candidate_bond(validator: &T::AccountId, bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond >= MinCandidateBond::<T>::get(), Error::<T>::BelowMinimumCandidateBond);
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
				CandidateDetail { bond, registered_at: now, total_delegations: Zero::zero() },
			);
			Ok(())
		}

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
