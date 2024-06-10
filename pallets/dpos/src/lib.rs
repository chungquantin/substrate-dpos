#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod constants;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::{types::*, weights::WeightInfo};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::{CheckedAdd, CheckedSub, Zero},
		traits::{
			fungible::{self, MutateHold},
			tokens::Precision,
			DefensiveSaturating, FindAuthor,
		},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use sp_runtime::{traits::One, BoundedVec, Saturating};
	use sp_std::{cmp::Reverse, collections::btree_set::BTreeSet, prelude::*, vec::Vec};

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

		/// The maximum number of delegators that the candidate can have
		#[pallet::constant]
		type MaxCandidateDelegators: Get<u32>;

		/// The maximum number of candidates in the active validator set
		#[pallet::constant]
		type MaxActiveValidators: Get<u32>;

		/// The maximum number of candidates in the active validator set
		#[pallet::constant]
		type MinActiveValidators: Get<u32>;

		/// The maximum number of candidates that delegators can delegate to
		#[pallet::constant]
		type MaxDelegateCount: Get<u32>;

		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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

	/// Number of blocks required for the deregister_candidate_method to work
	#[pallet::storage]
	#[pallet::getter(fn delay_deregister_candidate_duration)]
	pub type DelayDeregisterCandidateDuration<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Number of blocks required for the undelegate_candidate to work
	#[pallet::storage]
	#[pallet::getter(fn delay_undelegate_candidate)]
	pub type DelayUndelegateCandidate<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	// TODO: Migrate to claiming process
	/// Number of blocks required for the reward_payout_sent to work
	#[pallet::storage]
	#[pallet::getter(fn delay_reward_payout_sent)]
	pub type DelayRewardPayoutSent<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type CandidateDetailMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CandidateDetail<T>, OptionQuery>;

	/// Storing all the registration record of the candidates
	#[pallet::storage]
	#[pallet::getter(fn candidate_regristration)]
	pub type CandidateRegistrations<T: Config> = StorageValue<
		_,
		BoundedVec<
			CandidateRegistrationRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			<T as Config>::MaxCandidates,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn epoch_info)]
	pub type CurrentEpochInfo<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Selected validators for the current epoch
	#[pallet::storage]
	#[pallet::getter(fn active_validators)]
	pub type CurrentActiveValidators<T: Config> = StorageValue<
		_,
		BoundedVec<(T::AccountId, BalanceOf<T>), <T as Config>::MaxActiveValidators>,
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

	#[warn(type_alias_bounds)]
	type BoundedDelayActionRequestList<T> = BoundedVec<
		DelayActionRequest<T>,
		AddGet<<T as Config>::MaxCandidates, <T as Config>::MaxDelegateCount>,
	>;

	/// Store the requests for delay actions (Format: delay_xxxxx())
	#[pallet::storage]
	pub type DelayActionRequests<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		DelayActionType,
		BoundedDelayActionRequestList<T>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub genesis_candidates: CandidatePool<T>,
		pub min_candidate_bond: BalanceOf<T>,
		pub min_delegate_amount: BalanceOf<T>,
		pub epoch_duration: BlockNumberFor<T>,
		pub delay_deregister_candidate_duration: BlockNumberFor<T>,
		pub delay_undelegate_candidate: BlockNumberFor<T>,
		pub delay_reward_payout_sent: BlockNumberFor<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				T::MaxActiveValidators::get() >= One::one(),
				"Need at least one active validator for the network to function"
			);

			let mut visited: BTreeSet<T::AccountId> = BTreeSet::default();
			for (candidate, bond) in self.genesis_candidates.iter() {
				assert!(*bond >= self.min_candidate_bond, "Invalid bond for genesis candidate");
				assert!(visited.insert(candidate.clone()), "Candidate registration duplicates");

				Pallet::<T>::register_as_candidate_inner(&candidate, *bond)
					.expect("Register candidate error");
			}

			MinCandidateBond::<T>::put(self.min_candidate_bond);
			MinDelegateAmount::<T>::put(self.min_delegate_amount);
			EpochDuration::<T>::put(self.epoch_duration);

			DelayDeregisterCandidateDuration::<T>::put(self.delay_deregister_candidate_duration);
			DelayUndelegateCandidate::<T>::put(self.delay_deregister_candidate_duration);
			DelayRewardPayoutSent::<T>::put(self.delay_deregister_candidate_duration);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CandidateRegistered {
			candidate_id: T::AccountId,
			initial_bond: BalanceOf<T>,
		},
		CandidateRegistrationRemoved {
			candidate_id: T::AccountId,
		},
		CandidateDelegated {
			candidate_id: T::AccountId,
			delegated_by: T::AccountId,
			amount: BalanceOf<T>,
			total_delegated_amount: BalanceOf<T>,
		},
		CandidateUndelegated {
			candidate_id: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
			left_delegated_amount: BalanceOf<T>,
		},
		CandidateElected {
			candidate_id: T::AccountId,
			epoch_index: BlockNumberFor<T>,
			total_staked: BalanceOf<T>,
		},
		NextEpochMoved {
			last_epoch: BlockNumberFor<T>,
			next_epoch: BlockNumberFor<T>,
			at_block: BlockNumberFor<T>,
			total_delegations: BalanceOf<T>,
			total_validators: u64,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// This is a pretty lightweight check that we do EVERY block, but then tells us when an
			// Epoch has passed...
			let epoch_indx = n % EpochDuration::<T>::get();
			if n % EpochDuration::<T>::get() == BlockNumberFor::<T>::zero() {
				// CHANGE VALIDATORS LOGIC
				let active_validator_set = Self::select_active_validator_set();
				// You cannot return an error here, so you have to be clever with your code...
				for (active_validator_id, total_staked) in active_validator_set.iter() {
					Self::deposit_event(Event::<T>::CandidateElected {
						candidate_id: active_validator_id.clone(),
						epoch_index: epoch_indx,
						total_staked: *total_staked,
					});
				}

				CurrentActiveValidators::<T>::put(
					BoundedVec::try_from(active_validator_set)
						.expect("Exceed limit number of the validators in the active set"),
				);
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
		NoDelayActionRequestFound,
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
			match DelegationInfos::<T>::try_get(&delegator, &candidate) {
				Ok(mut delegation_info) => {
					// Update the delegated amount of the existing delegation info
					// Must accummulate the existing delegated amount with the new added amount
					let new_delegated_amount =
						delegation_info.amount.checked_add(&amount).expect("Overflow");
					Self::check_delegated_amount(new_delegated_amount)?;

					// Add the delegated amount in delegation info and update the storage value
					delegation_info.update_delegated_amount(new_delegated_amount);
					DelegationInfos::<T>::set(&delegator, &candidate, Some(delegation_info));
				},
				Err(_) => {
					// First time delegate to this candidate
					// Initializing a new record on the delegation info
					Self::check_delegated_amount(amount)?;

					// Only allow delegators to delegate a specific number of candidates
					// In case of direct delegation, the MaxDelegationCoutn will be set to 1
					let delegate_count = DelegateCountMap::<T>::get(&delegator);
					let new_delegate_count = delegate_count.saturating_add(1);
					ensure!(
						new_delegate_count <= T::MaxDelegateCount::get(),
						Error::<T>::TooManyCandidateDelegations
					);
					DelegateCountMap::<T>::set(&delegator, new_delegate_count);

					// Add delegator to the candidate delegators vector
					let mut candidate_delegators = CandidateDelegators::<T>::get(&candidate);
					candidate_delegators
						.try_push(delegator.clone())
						.map_err(|_| Error::<T>::TooManyDelegatorsInPool)?;
					CandidateDelegators::<T>::set(&candidate, candidate_delegators);

					// Initializing a new delegation info between (candidate, delegator)
					let new_delegation_info = DelegationInfo::default(amount);
					DelegationInfos::<T>::insert(&delegator, &candidate, new_delegation_info);
				},
			};

			T::NativeBalance::hold(&HoldReason::DelegateAmountReserved.into(), &delegator, amount)?;

			let total_delegated_amount = Self::increase_candidate_delegations(&candidate, &amount)?;

			Self::deposit_event(Event::CandidateDelegated {
				candidate_id: candidate,
				delegated_by: delegator,
				amount,
				total_delegated_amount,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn register_as_candidate(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			let validator = ensure_signed(origin)?;

			ensure!(bond >= MinCandidateBond::<T>::get(), Error::<T>::BelowMinimumCandidateBond);
			// Only hold the funds of a user which has no holds already.
			ensure!(
				!CandidateDetailMap::<T>::contains_key(&validator),
				Error::<T>::CandidateAlreadyExist
			);

			Self::register_as_candidate_inner(&validator, bond)?;

			Self::deposit_event(Event::CandidateRegistered {
				candidate_id: validator,
				initial_bond: bond,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn force_deregister_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			ensure!(
				CandidateDetailMap::<T>::contains_key(&candidate),
				Error::<T>::CandidateDoesNotExist
			);

			Self::deregister_candidate_inner(candidate)?;
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn undelegate_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let delegator = ensure_signed(origin)?;
			let mut delegation_info = DelegationInfos::<T>::try_get(&delegator, &candidate)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)?;

			let new_delegated_amount = match delegation_info.amount.checked_sub(&amount) {
				Some(value) => value,
				None => return Err(Error::<T>::InsufficientDelegatedAmount.into()),
			};

			if new_delegated_amount.is_zero() {
				// If the delegated amount is removed completely, we want to remove
				// related information to the delegation betwene (delegator, candidate)
				Self::remove_candidate_delegation_data(&delegator, &candidate)?;
			} else {
				// Remove the delegated amoutn partially but makes sure it is still above
				// the minimum delegated amount
				Self::check_delegated_amount(new_delegated_amount)?;

				delegation_info.update_delegated_amount(new_delegated_amount);
				DelegationInfos::<T>::set(&delegator, &candidate, Some(delegation_info));
			}
			// Releasing the hold amount for the delegation betwene (delegator, candidate)
			Self::release_delegated_amount(&delegator, &amount)?;

			// Reduce the candidate total_delegation by the undelegated amount
			Self::decrease_candidate_delegations(&candidate, &amount)?;

			Self::deposit_event(Event::CandidateUndelegated {
				candidate_id: candidate,
				delegator,
				amount,
				left_delegated_amount: new_delegated_amount,
			});
			Ok(())
		}

		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let candidate = ensure_signed(origin)?;

			ensure!(
				CandidateDetailMap::<T>::contains_key(&candidate),
				Error::<T>::CandidateDoesNotExist
			);

			Self::create_delay_action_request(candidate, None, DelayActionType::CandidateLeaved)?;
			Ok(())
		}

		#[pallet::call_index(23)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::execute_delay_action_inner(executor, DelayActionType::CandidateLeaved, 0)?;
			Ok(())
		}

		#[pallet::call_index(24)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_undelegate_candidate(origin: OriginFor<T>, indx: u32) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::execute_delay_action_inner(
				executor,
				DelayActionType::CandidateUndelegated,
				indx,
			)?;
			Ok(())
		}

		/// An example of directly updating the authorities into [`Config::ReportNewValidatorSet`].
		#[pallet::call_index(99)]
		#[pallet::weight(<T as Config>::WeightInfo::force_report_new_validators())]
		pub fn force_report_new_validators(
			origin: OriginFor<T>,
			new_set: Vec<T::AccountId>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			Self::report_new_validators(new_set)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn select_active_validator_set() -> ActiveValidatorSet<T> {
			let total_in_active_set = T::MaxActiveValidators::get() as usize;
			let candidate_registrations = CandidateRegistrations::<T>::get().into_inner();

			if candidate_registrations.len() < total_in_active_set {
				// If the number of candidates does not reached the threshold, return all
				return Self::get_candidate_delegations(candidate_registrations);
			}
			// Collect candidates with their total stake (bond + total delegations)
			let mut sorted_candidates: Vec<(T::AccountId, BalanceOf<T>)> =
				Self::get_candidate_delegations(candidate_registrations);

			// Sort candidates by their total stake in descending order
			sorted_candidates.sort_by_key(|&(_, total_stake)| Reverse(total_stake));

			// Select the top candidates based on the maximum active validators allowed
			sorted_candidates.into_iter().take(total_in_active_set).collect()
		}

		pub fn get_candidate_delegations(
			candidate_registrations: Vec<
				CandidateRegistrationRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			>,
		) -> ActiveValidatorSet<T> {
			candidate_registrations
				.into_iter()
				.filter_map(|CandidateRegistrationRequest { request_by, bond: _ }| {
					CandidateDetailMap::<T>::get(&request_by).map(
						|CandidateDetail { bond, total_delegations, registered_at: _ }| {
							(request_by, total_delegations.defensive_saturating_add(bond))
						},
					)
				})
				.collect::<ActiveValidatorSet<T>>()
		}

		pub fn report_new_validators(new_set: Vec<T::AccountId>) -> DispatchResult {
			ensure!(
				(new_set.len() as u32) < T::MaxCandidates::get(),
				Error::<T>::TooManyValidators
			);
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
			Ok(())
		}

		fn decrease_candidate_delegations(
			candidate: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResultWithValue<BalanceOf<T>> {
			let mut candidate_detail = CandidateDetailMap::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?;
			let total_delegated_amount = candidate_detail.sub_delegated_amount(*amount)?;
			CandidateDetailMap::<T>::set(&candidate, Some(candidate_detail));

			Ok(total_delegated_amount)
		}

		fn increase_candidate_delegations(
			candidate: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResultWithValue<BalanceOf<T>> {
			let mut candidate_detail = CandidateDetailMap::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?;
			let total_delegated_amount = candidate_detail.add_delegated_amount(*amount)?;
			CandidateDetailMap::<T>::set(&candidate, Some(candidate_detail));

			Ok(total_delegated_amount)
		}

		fn remove_candidate_delegation_data(
			delegator: &T::AccountId,
			candidate: &T::AccountId,
		) -> DispatchResult {
			DelegationInfos::<T>::remove(&delegator, &candidate);

			let delegate_count = DelegateCountMap::<T>::get(&delegator);
			DelegateCountMap::<T>::set(&delegator, delegate_count.saturating_sub(1));

			// Remove delegator from the candidate delegators vector
			let mut candidate_delegators = CandidateDelegators::<T>::get(&candidate);
			candidate_delegators
				.binary_search(&delegator)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)
				.map(|indx| candidate_delegators.remove(indx))?;
			CandidateDelegators::<T>::set(&candidate, candidate_delegators);

			Ok(())
		}

		fn remove_candidate_registration_data(candidate: &T::AccountId) {
			// Remove candidate registration
			let mut candidate_registrations = CandidateRegistrations::<T>::get();
			candidate_registrations.retain(|registration| registration.request_by != *candidate);
			CandidateRegistrations::<T>::set(candidate_registrations);
			CandidateDetailMap::<T>::remove(&candidate);
		}

		fn get_delay_action_duration(action_type: &DelayActionType) -> BlockNumberFor<T> {
			match action_type {
				DelayActionType::CandidateLeaved => DelayDeregisterCandidateDuration::<T>::get(),
				DelayActionType::CandidateUndelegated => DelayUndelegateCandidate::<T>::get(),
				DelayActionType::EpochRewardPayoutSent => DelayRewardPayoutSent::<T>::get(),
			}
		}

		fn execute_delay_action_inner(
			request_by: T::AccountId,
			action_type: DelayActionType,
			indx: u32,
		) -> DispatchResult {
			let now = frame_system::Pallet::<T>::block_number();
			let mut delay_requests = DelayActionRequests::<T>::get(&request_by, &action_type);
			ensure!(!delay_requests.is_empty(), Error::<T>::NoDelayActionRequestFound);

			match delay_requests.get(indx as usize) {
				Some(request) => {
					// Delay action is due, start executing the action
					if now.saturating_add(request.created_at) >= request.delay_for {
						match action_type {
							DelayActionType::CandidateLeaved => {
								Self::deregister_candidate_inner(request_by.clone())?;
							},
							DelayActionType::CandidateUndelegated => {
								unimplemented!();
							},
							DelayActionType::EpochRewardPayoutSent => {
								unimplemented!();
							},
						}
					}
				},
				None => return Err(Error::<T>::NoDelayActionRequestFound.into()),
			}

			delay_requests.remove(indx as usize);
			DelayActionRequests::<T>::set(&request_by, &action_type, delay_requests);

			Ok(())
		}

		fn create_delay_action_request(
			request_by: T::AccountId,
			consumed_amount: Option<BalanceOf<T>>,
			action_type: DelayActionType,
		) -> DispatchResult {
			let mut delay_requests = DelayActionRequests::<T>::get(&request_by, &action_type);
			delay_requests
				.try_push(DelayActionRequest {
					created_at: frame_system::Pallet::<T>::block_number(),
					delay_for: Self::get_delay_action_duration(&action_type),
					amount: consumed_amount,
				})
				.map_err(|_| Error::<T>::TooManyValidators)?;
			DelayActionRequests::<T>::set(&request_by, action_type, delay_requests);

			Ok(())
		}

		fn deregister_candidate_inner(candidate: T::AccountId) -> DispatchResult {
			let candidate_delegators = CandidateDelegators::<T>::get(&candidate);

			// Processing all the delegators of the candidate
			for delegator in candidate_delegators.into_inner() {
				let delegation_info = DelegationInfos::<T>::try_get(&delegator, &candidate)
					.map_err(|_| Error::<T>::DelegationDoesNotExist)?;

				// Trying to release all the hold amount of the delegators
				Self::release_delegated_amount(&delegator, &delegation_info.amount)?;

				// Removing any information related to the delegation between (candidate, delegator)
				Self::remove_candidate_delegation_data(&delegator, &candidate)?;
			}
			CandidateDelegators::<T>::remove(&candidate);

			// Releasing the hold bonds of the candidate
			Self::release_candidate_bonds(&candidate)?;

			// Removing any information related the registration of the candidate in the pool
			Self::remove_candidate_registration_data(&candidate);

			Self::deposit_event(Event::CandidateRegistrationRemoved { candidate_id: candidate });

			Ok(())
		}

		pub(crate) fn register_as_candidate_inner(
			validator: &T::AccountId,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			// Hold the amount for candidate bond registration
			T::NativeBalance::hold(&HoldReason::CandidateBondReserved.into(), &validator, bond)?;

			// Update the registration list of candidates
			let mut candidate_registrations = CandidateRegistrations::<T>::get();
			candidate_registrations
				.try_push(CandidateRegistrationRequest { bond, request_by: validator.clone() })
				.map_err(|_| Error::<T>::TooManyValidators)?;
			CandidateRegistrations::<T>::set(candidate_registrations);

			// Store the amount held in our local storage.
			CandidateDetailMap::<T>::insert(
				&validator,
				CandidateDetail {
					bond,
					registered_at: frame_system::Pallet::<T>::block_number(),
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
		fn release_delegated_amount(
			delegator: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResult {
			T::NativeBalance::release(
				&HoldReason::DelegateAmountReserved.into(),
				&delegator,
				*amount,
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
