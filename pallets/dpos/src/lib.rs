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
	use crate::{types::*, weights::WeightInfo};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::{CheckedAdd, Zero},
		traits::{fungible, fungible::MutateHold, tokens::Precision, FindAuthor},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use sp_runtime::BoundedVec;
	use sp_std::{prelude::*, vec::Vec};

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

		/// The maximum number of delegators that the candidate can have
		#[pallet::storage]
		type MaxCandidateDelegators: Get<u32>;

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

	#[pallet::type_value]
	pub fn DefaultMaxDelegateCount<T: Config>() -> u32 {
		1
	}

	/// The maximum number of candidates that delegators can delegate to
	#[pallet::storage]
	#[pallet::getter(fn max_delegate_count)]
	pub type MaxDelegateCount<T: Config> =
		StorageValue<_, u32, ValueQuery, DefaultMaxDelegateCount<T>>;

	/// The minimum number of delegate amount that the delegator need to provide for one candidate
	#[pallet::storage]
	#[pallet::getter(fn min_delegate_amount)]
	pub type MinDelegateAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_active_candidates)]
	/// The total candidates are assigned to the active set
	pub(crate) type TotalActiveCandidates<T: Config> = StorageValue<_, u32, ValueQuery>;

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
	pub type CandidateDetailMap<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, CandidateDetail<T>, OptionQuery>;

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

	/// The number of candidates that delegators delegated to
	#[pallet::storage]
	#[pallet::getter(fn delegate_count)]
	pub type DelegateCountMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

	/// DelegationInfos[(delegator_id, validator_id, delegated_amount)]
	#[pallet::storage]
	#[pallet::getter(fn delegation_infos)]
	pub type DelegationInfos<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		T::AccountId,
		DelegationInfo<T>,
		OptionQuery,
	>;

	/// Maximum number of delegators that candidate can have
	#[pallet::storage]
	#[pallet::getter(fn candidate_delegators)]
	pub type CandidateDelegators<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::AccountId, <T as Config>::MaxCandidateDelegators>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub min_candidate_bond: BalanceOf<T>,
		pub max_delegate_count: u32,
		pub min_delegate_amount: BalanceOf<T>,
		pub epoch_duration: BlockNumberFor<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			MinCandidateBond::<T>::put(self.min_candidate_bond);
			MaxDelegateCount::<T>::put(self.max_delegate_count);
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
		TooManyDelegatorsInPool,
		CandidateAlreadyExist,
		CandidateDoesNotExist,
		DelegationDoesNotExist,
		BelowMinimumDelegateAmount,
		BelowMinimumCandidateBond,
		InsufficientDelegatedAmount,
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
			let delegator = ensure_signed(origin)?;
			let mut candidate_detail = CandidateDetailMap::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?;
			match DelegationInfos::<T>::try_get(&delegator, &candidate) {
				Ok(mut delegation_info) => {
					// Update the delegated amount of the existing delegation info
					let new_delegated_amount =
						delegation_info.amount.checked_add(&amount).expect("Overflow");
					Self::check_delegated_amount(new_delegated_amount)?;

					delegation_info.update_delegated_amount(new_delegated_amount);
					DelegationInfos::<T>::set(&delegator, &candidate, Some(delegation_info));
				},
				Err(_) => {
					// First time delegate to this candidate
					Self::check_delegated_amount(amount)?;
					let delegate_count = DelegateCountMap::<T>::get(&delegator);
					let new_delegate_count = delegate_count.saturating_add(1);
					ensure!(
						new_delegate_count <= MaxDelegateCount::<T>::get(),
						Error::<T>::TooManyCandidateDelegations
					);
					DelegateCountMap::<T>::set(&delegator, new_delegate_count);
					// Add delegator to the candidate delegators vector
					let mut candidate_delegators = CandidateDelegators::<T>::get(&candidate);
					candidate_delegators
						.try_push(delegator.clone())
						.map_err(|_| Error::<T>::TooManyDelegatorsInPool)?;
					CandidateDelegators::<T>::set(&candidate, candidate_delegators);

					DelegationInfos::<T>::insert(
						&delegator,
						&candidate,
						DelegationInfo::default(amount),
					);
				},
			};

			T::NativeBalance::hold(&HoldReason::DelegateAmountReserved.into(), &delegator, amount)?;
			candidate_detail.add_delegated_amount(amount)?;
			CandidateDetailMap::<T>::set(&candidate, Some(candidate_detail));

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

			ensure!(bond >= MinCandidateBond::<T>::get(), Error::<T>::BelowMinimumCandidateBond);
			// Only hold the funds of a user which has no holds already.
			ensure!(
				!CandidateDetailMap::<T>::contains_key(&validator),
				Error::<T>::CandidateAlreadyExist
			);

			T::NativeBalance::hold(&HoldReason::CandidateBondReserved.into(), &validator, bond)?;

			Self::register_as_candidate_inner(&validator, bond)?;

			Self::deposit_event(Event::CandidateRegistered {
				candidate_id: validator,
				initial_bond: bond,
			});
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let candidate = ensure_signed(origin)?;
			// Only hold the funds of a user which has no holds already.
			ensure!(
				CandidateDetailMap::<T>::contains_key(&candidate),
				Error::<T>::CandidateDoesNotExist
			);

			let candidate_delegators = CandidateDelegators::<T>::get(&candidate);
			for delegator in candidate_delegators.into_inner() {
				Self::release_delegated_amount(&delegator, &candidate)?;
				Self::remove_candidate_delegation(&delegator, &candidate);
			}
			CandidateDelegators::<T>::set(&candidate, BoundedVec::default());
			Self::release_candidate_bonds(&candidate)?;
			Self::deregister_candidate_inner(&candidate);
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn undelegate_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let delegator = ensure_signed(origin)?;
			let mut delegation_info = DelegationInfos::<T>::try_get(&delegator, &candidate)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)?;

			let new_delegated_amount = match delegation_info.amount.checked_add(&amount) {
				Some(value) => value,
				None => return Err(Error::<T>::InsufficientDelegatedAmount.into()),
			};

			if new_delegated_amount.is_zero() {
				Self::remove_candidate_delegation(&delegator, &candidate);

				// Remove delegator from the candidate delegators vector
				let mut candidate_delegators = CandidateDelegators::<T>::get(&candidate);
				if let Ok(indx) = candidate_delegators.binary_search(&delegator) {
					candidate_delegators.remove(indx);
				}
				CandidateDelegators::<T>::set(&candidate, candidate_delegators);
			} else {
				Self::check_delegated_amount(new_delegated_amount)?;

				delegation_info.update_delegated_amount(new_delegated_amount);
				DelegationInfos::<T>::set(&delegator, &candidate, Some(delegation_info));
			}
			Ok(())
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
		fn current_block_number() -> BlockNumberFor<T> {
			frame_system::Pallet::<T>::block_number()
		}

		fn remove_candidate_delegation(delegator: &T::AccountId, candidate: &T::AccountId) {
			DelegationInfos::<T>::remove(&delegator, &candidate);

			let delegate_count = DelegateCountMap::<T>::get(&delegator);
			DelegateCountMap::<T>::set(&delegator, delegate_count.saturating_sub(1));
		}

		fn deregister_candidate_inner(candidate: &T::AccountId) {
			// Remove candidate registration
			let mut candidate_registrations = CandidateRegistrations::<T>::get();
			candidate_registrations.retain(|registration| registration.request_by != *candidate);
			CandidateRegistrations::<T>::set(candidate_registrations);
			// CandidateDelegators:<T>::remove(&candidate);
			CandidateDetailMap::<T>::remove(&candidate);
		}

		fn register_as_candidate_inner(
			validator: &T::AccountId,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			// Update the registration list of candidates
			let mut candidate_registrations = CandidateRegistrations::<T>::get();
			candidate_registrations
				.try_push(CandidateRegitrationRequest { bond, request_by: validator.clone() })
				.map_err(|_| Error::<T>::TooManyValidators)?;
			CandidateRegistrations::<T>::set(candidate_registrations);

			// Store the amount held in our local storage.
			CandidateDetailMap::<T>::insert(
				&validator,
				CandidateDetail {
					bond,
					registered_at: Self::current_block_number(),
					total_delegations: Zero::zero(),
				},
			);
			Ok(())
		}

		fn check_delegated_amount(amount: BalanceOf<T>) -> DispatchResult {
			let min_delegate_amount = MinDelegateAmount::<T>::get();
			ensure!(amount >= min_delegate_amount, Error::<T>::BelowMinimumDelegateAmount);
			Ok(())
		}

		/// Releasing the hold balance amount of candidate
		pub fn release_candidate_bonds(candidate: &T::AccountId) -> DispatchResult {
			let candidate_detail = CandidateDetailMap::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?;
			T::NativeBalance::release(
				&HoldReason::CandidateBondReserved.into(),
				&candidate,
				candidate_detail.bond,
				Precision::BestEffort,
			)?;
			Ok(())
		}

		/// Releasing the hold balance amount of delegator
		pub fn release_delegated_amount(
			delegator: &T::AccountId,
			candidate: &T::AccountId,
		) -> DispatchResult {
			let delegation_info = DelegationInfos::<T>::try_get(&delegator, &candidate)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)?;
			T::NativeBalance::release(
				&HoldReason::DelegateAmountReserved.into(),
				&delegator,
				delegation_info.amount,
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
