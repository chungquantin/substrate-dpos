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

// TODO add weight & return weight with DispatchResultWithPostInfo
// TODO Writing benchmark code
// TODO add force_set_balance_rate(root_origin, new_balance_rate)
// TODO consider improving the sorting algorithm
// TODO add integrity testing
// TODO add documentation
// TODO integrate with pallet_session
#[frame_support::pallet]
pub mod pallet {
	use crate::{types::*, weights::WeightInfo};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::{ValueQuery, *},
		sp_runtime::traits::{CheckedAdd, CheckedSub, Zero},
		traits::{
			fungible::{self, Mutate, MutateHold},
			tokens::{Fortitude, Precision},
			FindAuthor,
		},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use sp_runtime::{traits::One, BoundedVec, Percent, Saturating};
	use sp_std::{
		cmp::Reverse,
		collections::{btree_map::BTreeMap, btree_set::BTreeSet},
		prelude::*,
		vec::Vec,
	};

	pub trait ReportNewValidatorSet<AccountId> {
		fn report_new_validator_set(_new_set: Vec<AccountId>) {}
	}

	pub trait OnSlashHandler<AccountId, Balance> {
		fn on_slash(_who: &AccountId, _amount: Balance) {}
	}

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
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
		/// Candidate pool is bounded using this value
		#[pallet::constant]
		type MaxCandidates: Get<u32>;

		/// The maximum number of delegators that the candidate can have
		#[pallet::constant]
		type MaxCandidateDelegators: Get<u32>;

		/// The maximum number of candidates in the active validator set
		/// The parameter is used for selecting top N validators from the candidate pool
		#[pallet::constant]
		type MaxActiveValidators: Get<u32>;

		/// The minimum number of candidates in the active validator set
		/// If there lacks active validators, block production won't happen
		/// until there is anough validators. This ensure the network stability
		#[pallet::constant]
		type MinActiveValidators: Get<u32>;

		/// The maximum number of candidates that delegators can delegate to
		#[pallet::constant]
		type MaxDelegateCount: Get<u32>;

		/// The minimum number of stake that the candidate need to provide to secure slot
		type MinCandidateBond: Get<BalanceOf<Self>>;

		/// The minimum number of delegate amount that the delegator need to provide for one
		/// candidate
		type MinDelegateAmount: Get<BalanceOf<Self>>;

		/// We use a configurable constant `BlockNumber` to tell us when we should trigger the
		/// validator set change. The runtime developer should implement this to represent the time
		/// they want validators to change, but for your pallet, you just care about the block
		/// number.
		type EpochDuration: Get<BlockNumberFor<Self>>;

		/// Number of blocks required for the deregister_candidate_method to work
		type DelayDeregisterCandidateDuration: Get<BlockNumberFor<Self>>;

		/// Number of blocks required for the undelegate_candidate to work
		type DelayUndelegateCandidate: Get<BlockNumberFor<Self>>;

		/// Percentage of commission that the delegator receives for their delegations
		type DelegatorCommission: Get<u32>;

		/// Percentage of commission that the active validator receives for their delegations
		#[pallet::constant]
		type AuthorCommission: Get<u32>;

		type ConfigControllerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
		/// thanks to the `#[runtime]` macro.
		type RuntimeHoldReason: From<HoldReason>;

		/// Find the author of a block. A fake provide for this type is provided in the runtime. You
		/// can use a similar mechanism in your tests.
		type FindAuthor: FindAuthor<Self::AccountId>;

		/// The handler for on slashed action when validator misbehaves
		type OnSlashHandler: OnSlashHandler<Self::AccountId, BalanceOf<Self>>;
	}

	/// Mapping the validator ID with the reigstered candidate detail
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type CandidatePool<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, CandidateDetail<T>, OptionQuery>;

	/// Selected validators for the current epoch
	#[pallet::storage]
	#[pallet::getter(fn active_validators)]
	pub type CurrentActiveValidators<T: Config> = StorageValue<
		_,
		BoundedVec<(T::AccountId, BalanceOf<T>, BalanceOf<T>), <T as Config>::MaxActiveValidators>,
		ValueQuery,
	>;

	/// Unbounded storage in safe because the epoch snapshot just stores Vec values the
	/// BoundedVector
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn last_epoch_snapshot)]
	pub type LastEpochSnapshot<T: Config> = StorageValue<_, EpochSnapshot<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_points)]
	pub type RewardPoints<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// The number of candidates that delegators delegated to
	#[pallet::storage]
	#[pallet::getter(fn delegate_count)]
	pub type DelegateCountMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

	/// DelegationInfos[(delegator_id, validator_id, delegation_info)]
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

	/// Store the requests for delay actions (Format: delay_xxxxx())
	#[pallet::storage]
	pub type DelayActionRequests<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		DelayActionType,
		DelayActionRequest<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type BalanceRate<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis_candidates: CandidateSet<T>,
		pub balance_rate: u32,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				T::MaxActiveValidators::get() >= One::one(),
				"Need at least one active validator for the network to function"
			);

			assert!(
				T::AuthorCommission::get() > 0 && T::AuthorCommission::get() <= 100,
				"Validator commission must be in percentage"
			);

			assert!(
				T::DelegatorCommission::get() > 0 && T::DelegatorCommission::get() <= 100,
				"Delegator commission must be in percentage"
			);

			assert!(
				self.balance_rate > 0 && self.balance_rate < 10000,
				"Balance rate must be between 0 (0.1%) or 100 (100%)"
			);

			let mut visited: BTreeSet<T::AccountId> = BTreeSet::default();
			for (candidate, bond) in self.genesis_candidates.iter() {
				assert!(*bond >= T::MinCandidateBond::get(), "Invalid bond for genesis candidate");
				assert!(visited.insert(candidate.clone()), "Candidate registration duplicates");

				Pallet::<T>::register_as_candidate_inner(&candidate, *bond)
					.expect("Register candidate error");
			}

			BalanceRate::<T>::put(self.balance_rate);

			let active_validator_set = Pallet::<T>::select_active_validator_set().to_vec();
			CurrentActiveValidators::<T>::put(
				BoundedVec::try_from(active_validator_set.clone())
					.expect("Exceed limit number of the validators in the active set"),
			);
			LastEpochSnapshot::<T>::set(Some(Pallet::<T>::get_epoch_snapshot(
				&active_validator_set,
			)));
		}
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			// Default balance rate is kept as 100%
			GenesisConfig { genesis_candidates: vec![], balance_rate: 1000 }
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CandidateRegistered {
			candidate_id: T::AccountId,
			initial_bond: BalanceOf<T>,
		},
		CandidateMoreBondStaked {
			candidate_id: T::AccountId,
			additional_bond: BalanceOf<T>,
		},
		CandidateLessBondStaked {
			candidate_id: T::AccountId,
			deducted_bond: BalanceOf<T>,
		},
		CandidateBondSlashed {
			candidate_id: T::AccountId,
			slashed_amount: BalanceOf<T>,
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
		#[cfg(feature = "try-runtime")]
		fn try_state(_: BlockNumberFOr<T>) -> Result<(), sp_runtime::TryRuntimeError> {
			Self::do_try_state();
		}

		fn integrity_test() {
			assert!(
				T::MaxDelegateCount::get() != 0,
				"Maximum number of delegation per validator can't be zero"
			);

			assert!(
				T::MaxActiveValidators::get() != 0,
				"Maximum number of active validators can't be zero"
			);

			assert!(
				T::MinActiveValidators::get() < T::MaxActiveValidators::get(),
				"Minimum number of validators must be lower than the maximum number of validators"
			);
		}

		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if let Some(current_block_author) = Self::find_author() {
				// Whenever there is a block produced, we retrieve the snapshot of the epoch
				// The reward for validator and delegator will be calculated based on that
				if let Some(EpochSnapshot { validators, delegations }) =
					LastEpochSnapshot::<T>::get()
				{
					if let Some(total_bond) = validators.get(&current_block_author) {
						Self::sync_validator_rewards(
							&current_block_author,
							&delegations,
							total_bond,
						);
					}
				}
			}

			// This is a pretty lightweight check that we do EVERY block, but then tells us when an
			// Epoch has passed...
			let epoch_indx = n % T::EpochDuration::get();
			if epoch_indx == BlockNumberFor::<T>::zero() {
				let active_validator_set = Self::select_active_validator_set();

				// Update a new set of active validators
				CurrentActiveValidators::<T>::put(
					BoundedVec::try_from(active_validator_set.to_vec())
						.expect("Exceed limit number of the validators in the active set"),
				);
				// In new epoch, we want to set the CurrentEpochSnapshot to the current dataset
				LastEpochSnapshot::<T>::set(Some(Pallet::<T>::get_epoch_snapshot(
					&active_validator_set,
				)));
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
		InvalidMinimumDelegateAmount,
		InvalidMinimumCandidateBond,
		NoDelayActionRequestFound,
		ActionIsStillInDelayDuration,
		NoClaimableRewardFound,
		InvalidDelayActionPayload,
		InvalidZeroAmount,
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
		pub fn register_as_candidate(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond > Zero::zero(), Error::<T>::InvalidZeroAmount);
			ensure!(bond >= T::MinCandidateBond::get(), Error::<T>::BelowMinimumCandidateBond);

			let validator = ensure_signed(origin)?;

			// Only hold the funds of a user which has no holds already.
			ensure!(!Self::is_candidate(&validator), Error::<T>::CandidateAlreadyExist);

			Self::register_as_candidate_inner(&validator, bond)?;

			Self::deposit_event(Event::CandidateRegistered {
				candidate_id: validator,
				initial_bond: bond,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn candidate_bond_more(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond > Zero::zero(), Error::<T>::InvalidZeroAmount);

			let validator = ensure_signed(origin)?;

			let mut candidate_detail = Self::get_candidate(&validator)?;
			candidate_detail.bond = candidate_detail.bond.checked_add(&bond).expect("Overflow");
			CandidatePool::<T>::set(&validator, Some(candidate_detail));

			T::NativeBalance::hold(&HoldReason::CandidateBondReserved.into(), &validator, bond)?;

			Self::deposit_event(Event::CandidateMoreBondStaked {
				candidate_id: validator,
				additional_bond: bond,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn candidate_bond_less(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond > Zero::zero(), Error::<T>::InvalidZeroAmount);

			let validator = ensure_signed(origin)?;

			ensure!(Self::is_candidate(&validator), Error::<T>::CandidateDoesNotExist);

			let mut candidate_detail = Self::get_candidate(&validator)?;
			let new_candidate_bond = candidate_detail
				.bond
				.checked_sub(&bond)
				.ok_or(Error::<T>::InvalidMinimumCandidateBond)?;

			if new_candidate_bond.is_zero() {
				// If the candidate bond amount is removed completely, we want to remove
				// deregister the validator from candidate pool
				return Self::deregister_candidate_inner(validator);
			}

			// Reduce the total bond partially but make sure it is above the threshold
			Self::check_candidate_bond(new_candidate_bond)?;

			candidate_detail.update_bond(new_candidate_bond);
			CandidatePool::<T>::set(&validator, Some(candidate_detail));

			Self::release_candidate_bonds(&validator, bond)?;

			Self::deposit_event(Event::CandidateLessBondStaked {
				candidate_id: validator,
				deducted_bond: bond,
			});
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delegate_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(amount > Zero::zero(), Error::<T>::InvalidZeroAmount);

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
					Self::add_candidate_delegator(&candidate, &delegator)?;

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

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn force_deregister_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			ensure!(Self::is_candidate(&candidate), Error::<T>::CandidateDoesNotExist);

			Self::deregister_candidate_inner(candidate)?;
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn force_undelegate_candidate(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			ensure!(
				CandidatePool::<T>::contains_key(&candidate),
				Error::<T>::CandidateDoesNotExist
			);

			Self::undelegate_candidate_inner(delegator, candidate, amount)?;
			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let candidate = ensure_signed(origin)?;

			ensure!(
				!DelayActionRequests::<T>::contains_key(
					&candidate,
					DelayActionType::CandidateLeaved
				),
				Error::<T>::ActionIsStillInDelayDuration
			);

			Self::toggle_candidate_status(&candidate)?;

			Self::create_delay_action_request(
				candidate,
				None,
				None,
				DelayActionType::CandidateLeaved,
			)?;

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn delay_undelegate_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let delegator = ensure_signed(origin)?;
			ensure!(Self::is_candidate(&candidate), Error::<T>::CandidateDoesNotExist);

			ensure!(
				!DelayActionRequests::<T>::contains_key(
					&delegator,
					DelayActionType::CandidateUndelegated
				),
				Error::<T>::ActionIsStillInDelayDuration
			);

			DelegationInfos::<T>::try_get(&delegator, &candidate)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)?;

			Self::create_delay_action_request(
				delegator,
				Some(candidate),
				Some(amount),
				DelayActionType::CandidateUndelegated,
			)?;
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			// Default index of the deregister_candidate is 0 because we only allow 1 request at a
			// time
			Self::execute_delay_action_inner(executor, DelayActionType::CandidateLeaved)?;
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_deregister_candidate_request(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			// Default index of the deregister_candidate is 0 because we only allow 1 request at a
			// time
			Self::cancel_action_request_inner(executor.clone(), DelayActionType::CandidateLeaved)?;
			Self::toggle_candidate_status(&executor)?;
			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_undelegate_candidate(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::execute_delay_action_inner(executor, DelayActionType::CandidateUndelegated)?;
			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_undelegate_candidate_request(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::cancel_action_request_inner(executor, DelayActionType::CandidateUndelegated)?;
			Ok(())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn claim_reward(origin: OriginFor<T>) -> DispatchResult {
			let claimer = ensure_signed(origin)?;

			let reward_points = RewardPoints::<T>::try_get(&claimer)
				.map_err(|_| Error::<T>::NoClaimableRewardFound)?;
			ensure!(reward_points > Zero::zero(), Error::<T>::NoClaimableRewardFound);

			let _ = T::NativeBalance::mint_into(&claimer, reward_points);

			RewardPoints::<T>::remove(claimer);

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

	#[cfg(any(test, feature = "try-state"))]
	impl<T: Config> Pallet<T> {
		fn do_try_state() {
			unimplemented!()
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn toggle_candidate_status(candidate: &T::AccountId) -> DispatchResult {
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			candidate_detail.toggle_status();
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));
			Ok(())
		}

		pub(crate) fn select_active_validator_set() -> CandidateDelegationSet<T> {
			let total_in_active_set = T::MaxActiveValidators::get();
			if CandidatePool::<T>::count() < total_in_active_set {
				// If the number of candidates does not reached the threshold, return all
				return Self::get_candidate_delegations();
			}
			// Collect candidates with their total stake (bond + total delegations)
			let mut sorted_candidates: CandidateDelegationSet<T> =
				Self::get_candidate_delegations();

			// Sort candidates by their total stake in descending order
			sorted_candidates.sort_by_key(|&(_, _, total_stake)| Reverse(total_stake));

			// Select the top candidates based on the maximum active validators allowed
			let usize_total_in_active_set = total_in_active_set as usize;
			sorted_candidates.into_iter().take(usize_total_in_active_set).collect()
		}

		pub fn get_candidate_delegations() -> CandidateDelegationSet<T> {
			CandidatePool::<T>::iter()
				.map(|(candidate, candidate_detail)| {
					(candidate, candidate_detail.bond, candidate_detail.total())
				})
				.collect()
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
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			let total_delegated_amount = candidate_detail.sub_delegated_amount(*amount)?;
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));

			Ok(total_delegated_amount)
		}

		fn increase_candidate_delegations(
			candidate: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResultWithValue<BalanceOf<T>> {
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			let total_delegated_amount = candidate_detail.add_delegated_amount(*amount)?;
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));

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

		fn get_delay_action_duration(action_type: &DelayActionType) -> BlockNumberFor<T> {
			match action_type {
				DelayActionType::CandidateLeaved => T::DelayDeregisterCandidateDuration::get(),
				DelayActionType::CandidateUndelegated => T::DelayUndelegateCandidate::get(),
			}
		}

		fn cancel_action_request_inner(
			request_by: T::AccountId,
			action_type: DelayActionType,
		) -> DispatchResult {
			match DelayActionRequests::<T>::get(&request_by, &action_type) {
				Some(_) => {
					DelayActionRequests::<T>::set(&request_by, &action_type, None);
				},
				None => return Err(Error::<T>::NoDelayActionRequestFound.into()),
			}
			Ok(())
		}

		fn execute_delay_action_inner(
			request_by: T::AccountId,
			action_type: DelayActionType,
		) -> DispatchResult {
			let now = frame_system::Pallet::<T>::block_number();
			let request = DelayActionRequests::<T>::get(&request_by, &action_type)
				.ok_or(Error::<T>::NoDelayActionRequestFound)?;
			// Delay action is due, start executing the action
			ensure!(
				now.saturating_sub(request.created_at) >= request.delay_for,
				Error::<T>::ActionIsStillInDelayDuration
			);

			match action_type {
				DelayActionType::CandidateLeaved => {
					Self::deregister_candidate_inner(request_by.clone())?;
				},
				DelayActionType::CandidateUndelegated => {
					let candidate = request.target.ok_or(Error::<T>::InvalidDelayActionPayload)?;
					Self::undelegate_candidate_inner(
						request_by.clone(),
						candidate.clone(),
						request.amount.unwrap_or_default(),
					)?;
				},
			}
			DelayActionRequests::<T>::set(&request_by, &action_type, None);

			Ok(())
		}

		fn create_delay_action_request(
			request_by: T::AccountId,
			target: Option<T::AccountId>,
			consumed_amount: Option<BalanceOf<T>>,
			action_type: DelayActionType,
		) -> DispatchResult {
			DelayActionRequests::<T>::set(
				&request_by,
				&action_type,
				Some(DelayActionRequest {
					target,
					created_at: frame_system::Pallet::<T>::block_number(),
					delay_for: Self::get_delay_action_duration(&action_type),
					amount: consumed_amount,
				}),
			);
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
			let candidate_detail = Self::get_candidate(&candidate)?;
			Self::release_candidate_bonds(&candidate, candidate_detail.bond)?;

			// Removing any information related the registration of the candidate in the pool
			CandidatePool::<T>::remove(&candidate);

			Self::deposit_event(Event::CandidateRegistrationRemoved { candidate_id: candidate });

			Ok(())
		}

		fn undelegate_candidate_inner(
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(amount > Zero::zero(), Error::<T>::InvalidZeroAmount);

			let mut delegation_info = Self::get_delegation(&delegator, &candidate)?;
			let new_delegated_amount = delegation_info
				.amount
				.checked_sub(&amount)
				.ok_or(Error::<T>::InvalidMinimumDelegateAmount)?;

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

		pub fn add_candidate_delegator(
			candidate: &T::AccountId,
			delegator: &T::AccountId,
		) -> DispatchResult {
			let mut candidate_delegators = CandidateDelegators::<T>::get(&candidate);
			candidate_delegators
				.try_push(delegator.clone())
				.map_err(|_| Error::<T>::TooManyDelegatorsInPool)?;
			CandidateDelegators::<T>::set(&candidate, candidate_delegators);
			Ok(())
		}

		pub fn get_delegation(
			delegator: &T::AccountId,
			candidate: &T::AccountId,
		) -> DispatchResultWithValue<DelegationInfo<T>> {
			Ok(DelegationInfos::<T>::try_get(&delegator, &candidate)
				.map_err(|_| Error::<T>::DelegationDoesNotExist)?)
		}

		pub fn get_candidate(
			candidate: &T::AccountId,
		) -> DispatchResultWithValue<CandidateDetail<T>> {
			Ok(CandidatePool::<T>::try_get(&candidate)
				.map_err(|_| Error::<T>::CandidateDoesNotExist)?)
		}

		pub fn is_candidate(validator: &T::AccountId) -> bool {
			CandidatePool::<T>::contains_key(&validator)
		}

		pub(crate) fn register_as_candidate_inner(
			validator: &T::AccountId,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(
				CandidatePool::<T>::count().saturating_add(1) <= T::MaxCandidates::get(),
				Error::<T>::TooManyValidators
			);
			// Hold the amount for candidate bond registration
			T::NativeBalance::hold(&HoldReason::CandidateBondReserved.into(), &validator, bond)?;

			// Store the amount held in our local storage.
			CandidatePool::<T>::insert(&validator, CandidateDetail::new(bond));
			Ok(())
		}

		fn check_delegated_amount(amount: BalanceOf<T>) -> DispatchResult {
			ensure!(amount >= T::MinDelegateAmount::get(), Error::<T>::BelowMinimumDelegateAmount);
			Ok(())
		}

		fn check_candidate_bond(bond: BalanceOf<T>) -> DispatchResult {
			ensure!(bond >= T::MinCandidateBond::get(), Error::<T>::BelowMinimumCandidateBond);
			Ok(())
		}

		/// Releasing the hold balance amount of candidate
		pub fn release_candidate_bonds(
			candidate: &T::AccountId,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			T::NativeBalance::release(
				&HoldReason::CandidateBondReserved.into(),
				&candidate,
				bond,
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

		pub(crate) fn calculate_reward(total: BalanceOf<T>, percent: u32) -> BalanceOf<T> {
			Percent::from_rational(percent, 100) *
				Percent::from_rational(BalanceRate::<T>::get(), 100) *
				total
		}

		pub fn get_epoch_snapshot(
			active_validator_set: &CandidateDelegationSet<T>,
		) -> EpochSnapshot<T> {
			let mut epoch_snapshot = EpochSnapshot::<T>::default();
			for (active_validator_id, bond, _) in active_validator_set.to_vec().iter() {
				epoch_snapshot.add_validator(active_validator_id.clone(), bond.clone());
				for delegator in CandidateDelegators::<T>::get(active_validator_id) {
					if let Some(delegation_info) =
						DelegationInfos::<T>::get(&delegator, &active_validator_id)
					{
						epoch_snapshot.add_delegator(
							delegator,
							active_validator_id.clone(),
							delegation_info.amount,
						);
					}
				}
			}
			epoch_snapshot
		}

		pub fn sync_validator_rewards(
			validator: &T::AccountId,
			delegations: &BTreeMap<(T::AccountId, T::AccountId), BalanceOf<T>>,
			total_bond: &BalanceOf<T>,
		) {
			// Calculating the new reward of the block author
			let mut rewards = RewardPoints::<T>::get(&validator);
			rewards = rewards
				.saturating_add(Self::calculate_reward(*total_bond, T::AuthorCommission::get()));
			RewardPoints::<T>::set(validator.clone(), rewards);

			for ((delegator, candidate), amount) in delegations.iter() {
				if candidate != validator {
					continue;
				}
				// Calculating the new reward of the block author
				let mut rewards = RewardPoints::<T>::get(&delegator);
				rewards = rewards
					.saturating_add(Self::calculate_reward(*amount, T::DelegatorCommission::get()));
				RewardPoints::<T>::set(delegator, rewards);
			}
		}

		// Slashing the candidate bond, if under the minimum bond, candidate will be removed from
		// the pool
		pub fn do_slash(who: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			ensure!(!Self::is_candidate(&who), Error::<T>::CandidateAlreadyExist);

			let left_amount = T::NativeBalance::burn_held(
				&HoldReason::CandidateBondReserved.into(),
				&who,
				amount,
				Precision::BestEffort,
				Fortitude::Force,
			)?;

			let _ = T::OnSlashHandler::on_slash(&who, amount);

			let mut candidate_detail = Self::get_candidate(&who)?;
			if left_amount < T::MinCandidateBond::get() {
				return Self::deregister_candidate_inner(who);
			}

			candidate_detail.update_bond(left_amount);
			CandidatePool::<T>::set(&who, Some(candidate_detail));

			Self::release_candidate_bonds(&who, amount)?;

			Self::deposit_event(Event::CandidateBondSlashed {
				candidate_id: who,
				slashed_amount: amount,
			});

			Ok(())
		}

		// A function to get you an account id for the current block author.
		pub fn find_author() -> Option<T::AccountId> {
			// If you want to see a realistic example of the `FindAuthor` interface, see
			// `pallet-authorship`.
			T::FindAuthor::find_author::<'_, Vec<_>>(Default::default())
		}
	}
}
