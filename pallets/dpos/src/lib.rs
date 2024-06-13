#![cfg_attr(not(feature = "std"), no_std)]

//!  # Delegated Proof of Stake (DPOS) Pallet
//!
//! The Substrate DPoS Pallet provides a Delegated Proof of Stake mechanism for a Substrate-based
//! blockchain. It allows token holders to delegate their tokens to validators who are responsible
//! for producing blocks and securing the network.
//!
//! ## Overview
//!
//! The DPoS pallet implements a governance mechanism where stakeholders can elect a set of
//! validators to secure the network. Token holders delegate their stake to validators, who then
//! participate in the block production process. This pallet includes functionality for delegating
//! stake, selecting validators, and handling rewards and penalties. Moreover, this pallet also
//! provides the ability to switch between **Direct Delegation mode** and **Multi Delegation mode**
//!
//! ## Terminology
//!
//! - [`Candidate`]: Node who want to register as a candidate. A candidate node can receive stake
//!   delegations from token holders (delegator). Becoming a candidate can participate into the
//!   delegation process and produce blocks to earn rewards.
//! - [`Delegator`]: Token holders who delegate their token to the validator in the candidate pool.
//!   Delegators can receive reward for blocks produced by the delegated active validators.
//! - [`Delegating`]: A process of the delegator to vote for the candidate for the next epoch's
//!   validator election using tokens.
//! - [`Candidate Registeration`]: A process of the validator registering itself as the candidate
//!   for the next epoch's validator election
//! - [`Validator Election`]: Choosing the top most delegated candidates from the candidate pool for
//!   the next epoch.
//! - [`Commission`]: The percentage that block author and its delegator receive for a successfully
//!   produced block.
//! - [`Slash`]: The punishment of an active validator if they misbehave.
//! - [`Epoch`]: A predefined period during which the set of active validators remains fixed. At the
//!   end of each epoch, a new set of validators can be elected based on the current delegations.

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

// TODO OnSlashHandler test
// TODO polkadot_sdk_frame
// TODO DefaultTestConfig
// TODO video & diagrams
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

	/// Reporting a new set of validators to an external system or component.
	pub trait ReportNewValidatorSet<AccountId> {
		fn report_new_validator_set(_new_set: Vec<AccountId>) {}
	}

	/// A hook to act on if there is a validator in the active validator set misbehaves
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
		/// If the number of delegators reaches the maximum, delegator with the lowest amount
		/// will be replaced by the new delegator if the new delegation is higher
		#[pallet::constant]
		type MaxCandidateDelegators: Get<u32>;

		/// The maximum number of candidates in the active validator set
		/// The parameter is used for selecting top N validators from the candidate pool
		#[pallet::constant]
		type MaxActiveValidators: Get<u32>;

		/// The minimum number of candidates in the active validator set
		/// If there lacks active validators, block production won't happen
		/// until there is enough validators. This ensure the network stability
		#[pallet::constant]
		type MinActiveValidators: Get<u32>;

		/// The maximum number of candidates that delegators can delegate their tokens to.
		#[pallet::constant]
		type MaxDelegateCount: Get<u32>;

		/// The minimum number of stake that the candidate need to provide to register
		/// in the candidate pool
		#[pallet::constant]
		type MinCandidateBond: Get<BalanceOf<Self>>;

		/// The minimum number of delegated amount that the delegator need to provide for one
		/// candidate.
		#[pallet::constant]
		type MinDelegateAmount: Get<BalanceOf<Self>>;

		/// A predefined period during which the set of active validators remains fixed. At the end
		/// of each epoch, a new set of validators can be elected based on the current delegations.
		#[pallet::constant]
		type EpochDuration: Get<BlockNumberFor<Self>>;

		/// Number of blocks required for the deregister_candidate_method to work
		#[pallet::constant]
		type DelayDeregisterCandidateDuration: Get<BlockNumberFor<Self>>;

		/// Number of blocks required for the undelegate_candidate to work
		#[pallet::constant]
		type DelayUndelegateCandidate: Get<BlockNumberFor<Self>>;

		/// Percentage of commission that the delegator receives for their delegations
		#[pallet::constant]
		type DelegatorCommission: Get<u32>;

		/// Percentage of commission that the active validator receives for their delegations
		#[pallet::constant]
		type AuthorCommission: Get<u32>;

		/// Origin that has the authority to control the parameters in the delegated proof of stake
		/// network
		type ConfigControllerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that has the authority to perform privileged actions
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Report the new validators to the runtime. This is done through a custom trait defined in
		/// this pallet.
		type ReportNewValidatorSet: ReportNewValidatorSet<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Overarching hold reason. Our `HoldReason` below will become a part of this "Outer Enum"
		/// thanks to the `#[runtime]` macro.
		type RuntimeHoldReason: From<HoldReason>;

		/// Find the author of a block.
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

	/// Snapshot of the last epoch data, which includes the active validator set along with their
	/// total bonds and delegations. This storage is unbounded but safe, as it only stores `Vec`
	/// values within a `BoundedVec`. The total number of delegations is limited by the size
	/// `MaxActiveValidators * MaxCandidateDelegators`.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn last_epoch_snapshot)]
	pub type LastEpochSnapshot<T: Config> = StorageValue<_, EpochSnapshot<T>, OptionQuery>;

	/// Stores the total claimable rewards for each account, which can be a validator or a
	/// delegator. The reward points are updated with each block produced
	#[pallet::storage]
	#[pallet::getter(fn reward_points)]
	pub type RewardPoints<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// The number of candidates that delegators delegated to
	#[pallet::storage]
	#[pallet::getter(fn delegate_count)]
	pub type DelegateCountMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

	/// Stores delegation information mapping delegator accounts to validator accounts and their
	/// corresponding delegation details.
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

	/// Stores requests for delayed actions that cannot be executed immediately but need to be
	/// executed after a specified delay duration.
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

	/// Stores the balance rate configuration, which controls the inflation rebalancing of the DPoS
	/// network. Value of the balance rate must be between 0 (0.0%) to 1000 (100%)
	#[pallet::storage]
	pub type BalanceRate<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type EpochIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

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
				self.balance_rate > 0 && self.balance_rate <= 1000,
				"Balance rate must be between 0 (0.1%) or 100 (100%)"
			);

			// Populates the provided genesis candidates with bond in storage.
			// Ensures that there are no duplicate candidates in the `genesis_candidates`.
			let mut visited: BTreeSet<T::AccountId> = BTreeSet::default();
			for (candidate, bond) in self.genesis_candidates.iter() {
				assert!(*bond >= T::MinCandidateBond::get(), "Invalid bond for genesis candidate");
				assert!(visited.insert(candidate.clone()), "Candidate registration duplicates");

				Pallet::<T>::register_as_candidate_inner(&candidate, *bond)
					.expect("Register candidate error");
			}

			BalanceRate::<T>::put(self.balance_rate);

			// Update the active validator set using the data stored in the candidate pool
			let active_validator_set = Pallet::<T>::select_active_validator_set().to_vec();
			CurrentActiveValidators::<T>::put(
				BoundedVec::try_from(active_validator_set.clone())
					.expect("Exceed limit number of the validators in the active set"),
			);
			// Capture the snapshot of the last epoch
			LastEpochSnapshot::<T>::set(Some(Pallet::<T>::capture_epoch_snapshot(
				&active_validator_set,
			)));

			let new_set = CurrentActiveValidators::<T>::get()
				.iter()
				.map(|(active_validator, _, _)| active_validator.clone())
				.collect::<Vec<T::AccountId>>();

			Pallet::<T>::report_new_validators(new_set);
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
		/// Event emitted when there is a new candidate registered
		CandidateRegistered { candidate_id: T::AccountId, initial_bond: BalanceOf<T> },
		/// Event emitted when candidate add more tho their total bond
		CandidateMoreBondStaked { candidate_id: T::AccountId, additional_bond: BalanceOf<T> },
		/// Event emitted when candidates reduce their bond amount
		CandidateLessBondStaked { candidate_id: T::AccountId, deducted_bond: BalanceOf<T> },
		/// Event emitted when candidate misbehaves
		CandidateBondSlashed { candidate_id: T::AccountId, slashed_amount: BalanceOf<T> },
		/// Event emitted when candidate is removed from the candidate pool
		CandidateRegistrationRemoved { candidate_id: T::AccountId },
		/// Event emitted when candidate is delegated
		CandidateDelegated {
			candidate_id: T::AccountId,
			delegated_by: T::AccountId,
			amount: BalanceOf<T>,
			total_delegated_amount: BalanceOf<T>,
		},
		/// Event emitted when candidate is delegated
		CandidateUndelegated {
			candidate_id: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
			left_delegated_amount: BalanceOf<T>,
		},
		/// Event emitted when the reward is claimed
		RewardClaimed { claimer: T::AccountId, total_reward: BalanceOf<T> },
		/// Event emitted when candidate is delegated
		NextEpochMoved {
			last_epoch: u32,
			next_epoch: u32,
			at_block: BlockNumberFor<T>,
			total_candidates: u64,
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
				LastEpochSnapshot::<T>::set(Some(Pallet::<T>::capture_epoch_snapshot(
					&active_validator_set,
				)));

				let new_set = CurrentActiveValidators::<T>::get()
					.iter()
					.map(|(active_validator, _, _)| active_validator.clone())
					.collect::<Vec<T::AccountId>>();

				Pallet::<T>::report_new_validators(new_set);
				Self::move_to_next_epoch(active_validator_set);
			}
			// We return a default weight because we do not expect you to do weights for your
			// project... Except for extra credit...
			return Weight::default();
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Thrown when there are too many validators exceeding the pool limit
		TooManyValidators,
		/// Thrown when a delegator vote too many candidates exceeding the allowed limit
		TooManyCandidateDelegations,
		/// Thrown when candidate has too many delegations exceeding the delegator pool limit
		TooManyDelegatorsInPool,
		/// Thrown when candidate is in the pool already
		CandidateAlreadyExist,
		/// Thrown when candidate is not registered yet
		CandidateDoesNotExist,
		/// Thrown when there is no record of delegation between the delegator and the candidate
		DelegationDoesNotExist,
		/// Thrown when the delegated amount is below the minimum threshold
		BelowMinimumDelegateAmount,
		/// Thrown when the candidate bond is below the minimum threshold
		BelowMinimumCandidateBond,
		/// Thrown when the delegated amount is invalid (Example: 0)
		InvalidMinimumDelegateAmount,
		/// Thrown when the provided candidate bond amount is invalid (Example: 0)
		InvalidMinimumCandidateBond,
		/// Thrown when there is no delay action request found
		NoDelayActionRequestFound,
		/// Thrown when the action is still in the delay duration and can't be executed
		ActionIsStillInDelayDuration,
		/// Thrown when there is no reward to be claimed
		NoClaimableRewardFound,
		/// Thrown when the payload for delay action is invalid
		InvalidDelayActionPayload,
		/// Thrown when the zero input amount is not accepted
		InvalidZeroAmount,
		/// Thrown when the provided number is not a percentage
		IsNotPercentage,
	}

	/// A reason for the pallet dpos placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Hold the candidate balance to reserve it for registration to the candidate pool.
		#[codec(index = 0)]
		CandidateBondReserved,
		/// Hold the amount delegated to the candidate
		#[codec(index = 1)]
		DelegateAmountReserved,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Nodes can register themselves as a candidate in the DPoS (Delegated Proof of Stake)
		/// network.
		///
		/// Requires the caller to provide a bond amount greater than zero and at least equal to the
		/// minimum required candidate bond configured in the pallet's runtime configuration
		/// (`MinCandidateBond`).
		///
		/// If successful, the caller's account is registered as a candidate with the specified bond
		/// amount, and a `CandidateRegistered` event is emitted.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction.
		/// - `bond`: The amount of funds to bond as part of the candidate registration.
		///
		/// Errors:
		/// - `InvalidZeroAmount`: Raised if `bond` is zero.
		/// - `BelowMinimumCandidateBond`: Raised if `bond` is less than `MinCandidateBond`.
		/// - `CandidateAlreadyExist`: Raised if the caller is already registered as a candidate.
		///
		/// Emits:
		/// - `CandidateRegistered`: When a candidate successfully registers, including the
		///   candidate's account ID (`candidate_id`) and the initial bond amount (`initial_bond`).
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `register_as_candidate`.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn register_as_candidate(origin: OriginFor<T>, bond: BalanceOf<T>) -> DispatchResult {
			// Number of initial bond that the candidate secure for their registration
			ensure!(bond > Zero::zero(), Error::<T>::InvalidZeroAmount);
			ensure!(bond >= T::MinCandidateBond::get(), Error::<T>::BelowMinimumCandidateBond);

			// Origin of the candidate account
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

		/// Increases the bond amount for an existing candidate in the DPoS (Delegated Proof of
		/// Stake) network.
		///
		/// Requires the caller to provide a bond amount greater than zero.
		///
		/// If successful, the candidate's bond amount is increased by the specified amount, and the
		/// corresponding funds are held from the caller's account (`validator`) using the
		/// `NativeBalance` pallet.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction.
		/// - `bond`: The additional amount of funds to bond with the candidate.
		///
		/// Errors:
		/// - `InvalidZeroAmount`: Raised if `bond` is zero.
		/// - `CandidateNotFound`: Raised if the caller is not registered as a candidate.
		/// - `BalanceOverflow`: Raised if the addition of `bond` to the candidate's existing bond
		///   amount results in overflow.
		///
		/// Effects:
		/// - Increases the bond amount for the candidate identified by `validator`.
		/// - Holds `bond` amount from the caller's account as candidate bond.
		///
		/// Emits:
		/// - `CandidateMoreBondStaked`: When a candidate successfully increases their bond,
		///   including the candidate's account ID (`candidate_id`) and the additional bond amount
		///   (`additional_bond`).
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `candidate_bond_more`.
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

		/// Reduces the bond amount for an existing candidate in the DPoS (Delegated Proof of Stake)
		/// network.
		///
		/// Requires the caller to provide a bond amount greater than zero.
		///
		/// If successful and the new bond amount is above the minimum threshold, the candidate's
		/// bond amount is reduced by the specified amount, and the corresponding funds are released
		/// back to the caller's account (`validator`) using the `NativeBalance` pallet.
		///
		/// If the new bond amount becomes zero, the candidate is deregistered from the candidate
		/// pool.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction.
		/// - `bond`: The amount of funds to reduce from the candidate's bond.
		///
		/// Errors:
		/// - `InvalidZeroAmount`: Raised if `bond` is zero.
		/// - `CandidateDoesNotExist`: Raised if the caller is not registered as a candidate.
		/// - `InvalidMinimumCandidateBond`: Raised if reducing `bond` would result in a bond amount
		///   below the minimum required candidate bond.
		///
		/// Effects:
		/// - Reduces the bond amount for the candidate identified by `validator`, potentially
		///   deregistering them if the bond amount becomes zero.
		/// - Releases `bond` amount back to the caller's account if the reduction is successful and
		///   the new bond amount is above the threshold.
		///
		/// Emits:
		/// - `CandidateLessBondStaked`: When a candidate successfully reduces their bond, including
		///   the candidate's account ID (`candidate_id`) and the deducted bond amount
		///   (`deducted_bond`).
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `candidate_bond_less`.
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

		/// Delegates a specified amount of funds to a candidate in the DPoS (Delegated Proof of
		/// Stake) network.
		///
		/// Requires the caller to provide an amount greater than zero.
		///
		/// If the delegator has previously delegated to the candidate, the delegated amount is
		/// updated by adding the new amount to the existing delegation. If it's the first time
		/// delegation, a new delegation record is initialized.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction.
		/// - `candidate`: The account ID of the candidate to delegate funds to.
		/// - `amount`: The amount of funds to delegate.
		///
		/// Errors:
		/// - `InvalidZeroAmount`: Raised if `amount` is zero.
		/// - `TooManyCandidateDelegations`: Raised if the delegator exceeds the maximum allowed
		///   number of candidate delegations.
		/// - `BalanceOverflow`: Raised if adding `amount` to an existing delegated amount results
		///   in overflow.
		///
		/// Effects:
		/// - Updates the delegated amount for the specified candidate and delegator.
		/// - Increases the count of candidates delegated to by the delegator if it's the first time
		///   delegating to this candidate.
		/// - Holds `amount` from the delegator's account as delegated amount.
		///
		/// Emits:
		/// - `CandidateDelegated`: When a delegator successfully delegates funds to a candidate,
		///   including the candidate's account ID (`candidate_id`), delegator's account ID
		///   (`delegated_by`), the delegated amount (`amount`), and the total delegated amount to
		///   the candidate after the delegation (`total_delegated_amount`).
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for `delegate_candidate`.
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

		/// Forcefully deregisters a candidate from the DPoS (Delegated Proof of Stake) network.
		///
		/// Requires the caller to have the privilege defined by `ForceOrigin`.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be authorized by `ForceOrigin`.
		/// - `candidate`: The account ID of the candidate to be deregistered.
		///
		/// Errors:
		/// - `CandidateDoesNotExist`: Raised if the candidate specified does not exist in the
		///   candidate pool.
		///
		/// Effects:
		/// - Deregisters the candidate identified by `candidate` from the candidate pool.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `force_deregister_candidate`.
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

		/// Forcefully undelegates a specified amount of funds from a candidate in the DPoS
		/// (Delegated Proof of Stake) network.
		///
		/// Requires the caller to have the privilege defined by `ForceOrigin`.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be authorized by `ForceOrigin`.
		/// - `delegator`: The account ID of the delegator who wants to undelegate funds.
		/// - `candidate`: The account ID of the candidate from whom funds will be undelegated.
		/// - `amount`: The amount of funds to undelegate.
		///
		/// Errors:
		/// - `CandidateDoesNotExist`: Raised if the specified candidate does not exist in the
		///   candidate pool.
		/// - Errors from `undelegate_candidate_inner` function, such as insufficient funds to
		///   undelegate.
		///
		/// Effects:
		/// - Undelegates the specified `amount` of funds from the `candidate` by the `delegator`.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `force_undelegate_candidate`.
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

		/// Initiates a delayed undelegation action for a specified candidate in the DPoS (Delegated
		/// Proof of Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the delegator
		///   initiating the action.
		/// - `candidate`: The account ID of the candidate from whom funds will be undelegated after
		///   the delay period.
		/// - `amount`: The amount of funds to undelegate.
		///
		/// Errors:
		/// - `CandidateDoesNotExist`: Raised if the specified candidate does not exist in the
		///   candidate pool.
		/// - `ActionIsStillInDelayDuration`: Raised if there is already a pending delay action for
		///   undelegating the candidate.
		/// - `DelegationDoesNotExist`: Raised if there is no existing delegation from the
		///   `delegator` to the `candidate`.
		///
		/// Effects:
		/// - Creates a delay action request to undelegate `amount` of funds from `candidate` by
		///   `delegator` after a specified delay duration.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `delay_undelegate_candidate`.
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

		/// Executes a delayed deregistration action for a candidate in the DPoS (Delegated Proof of
		/// Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the executor
		///   triggering the action.
		///
		/// Errors:
		/// - None. This function will always succeed as it executes a predefined delayed action.
		///
		/// Effects:
		/// - Executes the delayed action to deregister a candidate from the candidate pool.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `execute_deregister_candidate`.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_deregister_candidate(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::execute_delay_action_inner(executor, DelayActionType::CandidateLeaved)?;
			Ok(())
		}

		/// Cancels a previously initiated request to deregister a candidate from the DPoS
		/// (Delegated Proof of Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the executor who
		///   initiated the cancellation request.
		///
		/// Errors:
		/// - None. This function will succeed regardless of the current state of the cancellation
		///   request.
		///
		/// Effects:
		/// - If a deregistration request was pending for the executor, it will be canceled.
		/// - The candidate's status will be toggled to active.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `cancel_deregister_candidate_request`.
		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_deregister_candidate_request(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::cancel_action_request_inner(executor.clone(), DelayActionType::CandidateLeaved)?;
			Self::toggle_candidate_status(&executor)?;
			Ok(())
		}

		/// Executes a delayed undelegation action for a candidate in the DPoS (Delegated Proof of
		/// Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the executor who
		///   initiated the execution.
		///
		/// Errors:
		/// - None. This function will succeed regardless of the current state of the undelegation
		///   request.
		///
		/// Effects:
		/// - If an undelegation request was pending for the executor, it will be executed.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `execute_undelegate_candidate`.
		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn execute_undelegate_candidate(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::execute_delay_action_inner(executor, DelayActionType::CandidateUndelegated)?;
			Ok(())
		}

		/// Cancels a pending undelegation request for a candidate in the DPoS (Delegated Proof of
		/// Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the executor who
		///   initiated the cancellation.
		///
		/// Errors:
		/// - If no pending undelegation request exists for the executor, the function will return
		///   an `Error`.
		///
		/// Effects:
		/// - Cancels the pending undelegation request, if it exists.
		///
		/// Emits:
		/// - No events are emitted directly by this function.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `cancel_undelegate_candidate_request`.
		#[pallet::call_index(12)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn cancel_undelegate_candidate_request(origin: OriginFor<T>) -> DispatchResult {
			let executor = ensure_signed(origin)?;
			Self::cancel_action_request_inner(executor, DelayActionType::CandidateUndelegated)?;
			Ok(())
		}

		/// Claims the accumulated reward points as native tokens for the claimer (validator or
		/// delegator) in the DPoS (Delegated Proof of Stake) network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be signed by the claimer
		///   (validator or delegator).
		///
		/// Errors:
		/// - If no claimable rewards are found for the claimer, the function will return an
		///   `Error`.
		///
		/// Effects:
		/// - Mints native tokens into the claimer's account equivalent to their accumulated reward
		///   points.
		/// - Removes the claimer's accumulated reward points from storage after claiming.
		/// - Emits a `RewardClaimed` event upon successful claim.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for `claim_reward`.
		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn claim_reward(origin: OriginFor<T>) -> DispatchResult {
			let claimer = ensure_signed(origin)?;

			let reward_points = RewardPoints::<T>::try_get(&claimer)
				.map_err(|_| Error::<T>::NoClaimableRewardFound)?;
			ensure!(reward_points > Zero::zero(), Error::<T>::NoClaimableRewardFound);

			Self::claim_reward_inner(claimer, reward_points);

			Ok(())
		}

		/// Sets the balance rate to control the inflation of the DPoS (Delegated Proof of Stake)
		/// network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be authorized by the
		///   `ConfigControllerOrigin`.
		/// - `new_balance_rate`: The new balance rate value to be set, controlling the inflation
		///   rate as a percentage.
		///
		/// Errors:
		/// - If the origin is not authorized to execute this function, it will return an `Error`.
		/// - If the provided `new_balance_rate` is not within the valid range (1 to 999 inclusive),
		///   it will return an `Error`.
		///
		/// Effects:
		/// - Updates the `BalanceRate` storage value to the new specified rate.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `force_set_balance_rate`.
		#[pallet::call_index(14)]
		#[pallet::weight(<T as Config>::WeightInfo::default())]
		pub fn force_set_balance_rate(
			origin: OriginFor<T>,
			new_balance_rate: u32,
		) -> DispatchResult {
			T::ConfigControllerOrigin::ensure_origin(origin)?;

			ensure!(new_balance_rate > 0 && new_balance_rate < 1000, Error::<T>::IsNotPercentage);

			BalanceRate::<T>::set(new_balance_rate);

			Ok(())
		}

		/// Forces the reporting of a new validator set in the DPoS (Delegated Proof of Stake)
		/// network.
		///
		/// Parameters:
		/// - `origin`: The origin of the transaction, which must be authorized by `ForceOrigin`.
		/// - `new_set`: The new set of validator accounts to be reported.
		///
		/// Errors:
		/// - If the origin is not authorized to execute this function, it will return an `Error`.
		/// - Errors from `report_new_validators` if the reporting fails.
		///
		/// Effects:
		/// - Calls `report_new_validators` with the provided `new_set` to update the validator set.
		///
		/// Weight: Determined by the pallet's `WeightInfo` implementation for
		/// `force_report_new_validators`.
		#[pallet::call_index(99)]
		#[pallet::weight(<T as Config>::WeightInfo::force_report_new_validators())]
		pub fn force_report_new_validators(
			origin: OriginFor<T>,
			new_set: Vec<T::AccountId>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			ensure!(
				(new_set.len() as u32) < T::MaxCandidates::get(),
				Error::<T>::TooManyValidators
			);
			Self::report_new_validators(new_set);
			Ok(())
		}
	}

	#[cfg(any(test, feature = "try-state"))]
	impl<T: Config> Pallet<T> {
		pub fn do_try_state() {
			assert!(
				BalanceRate::<T>::get() > 0 && BalanceRate::<T>::get() <= 1000,
				"Balance rate must be between 0 (0.1%) or 1000 (100%)"
			);

			let mut visited: BTreeSet<T::AccountId> = BTreeSet::default();
			for (candidate, candidate_detail) in CandidatePool::<T>::iter() {
				assert!(
					candidate_detail.bond >= T::MinCandidateBond::get(),
					"Invalid bond for genesis candidate"
				);
				assert!(visited.insert(candidate.clone()), "Candidate registration duplicates");
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Toggles the online status of a candidate in the DPoS network.
		pub(crate) fn toggle_candidate_status(candidate: &T::AccountId) -> DispatchResult {
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			candidate_detail.toggle_status();
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));
			Ok(())
		}

		/// Update the epoch index and move to the next epoch
		pub(crate) fn move_to_next_epoch(active_valivdator_set: CandidateDelegationSet<T>) {
			let epoch_index = EpochIndex::<T>::get();
			let next_epoch_index = epoch_index.saturating_add(1);
			EpochIndex::<T>::set(next_epoch_index);

			Self::deposit_event(Event::NextEpochMoved {
				last_epoch: epoch_index,
				next_epoch: next_epoch_index,
				at_block: frame_system::Pallet::<T>::block_number(),
				total_candidates: CandidatePool::<T>::count() as u64,
				total_validators: active_valivdator_set.len() as u64,
			});
		}

		/// Filters top staked validators for the active set based on their stake (bond + total
		/// delegations), ensuring the set does not exceed the configured maximum.
		pub(crate) fn select_active_validator_set() -> CandidateDelegationSet<T> {
			// If the number of candidates is below the threshold for active set, network won't
			// function
			if CandidatePool::<T>::count() < T::MinActiveValidators::get() {
				return vec![];
			}
			let total_in_active_set = T::MaxActiveValidators::get();
			if CandidatePool::<T>::count() < total_in_active_set {
				// If the number of candidates does not reached the threshold, return all
				return Self::get_online_candidate_set();
			}
			// Collect candidates with their total stake (bond + total delegations)
			let mut sorted_candidates: CandidateDelegationSet<T> = Self::get_online_candidate_set();

			// Sort candidates by their total stake in descending order
			sorted_candidates.sort_by_key(|&(_, _, total_stake)| Reverse(total_stake));

			// Select the top candidates based on the maximum active validators allowed
			let usize_total_in_active_set = total_in_active_set as usize;
			sorted_candidates.into_iter().take(usize_total_in_active_set).collect()
		}

		/// Get the candidate information associated with the delegations of the candidate
		/// Offline candidates can't participate into the active validator set until they turn back
		/// to online
		pub fn get_online_candidate_set() -> CandidateDelegationSet<T> {
			CandidatePool::<T>::iter()
				.filter_map(|(candidate, candidate_detail)| match candidate_detail.status {
					ValidatorStatus::Online =>
						Some((candidate, candidate_detail.bond, candidate_detail.total())),
					ValidatorStatus::Offline => None,
				})
				.collect()
		}

		/// Reporting new validator set to the external system
		pub fn report_new_validators(new_set: Vec<T::AccountId>) {
			T::ReportNewValidatorSet::report_new_validator_set(new_set);
		}

		/// Storage call to decrease the delegation amount of the candidate in the candidate pool
		fn decrease_candidate_delegations(
			candidate: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResultWithValue<BalanceOf<T>> {
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			let total_delegated_amount = candidate_detail.sub_delegated_amount(*amount)?;
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));

			Ok(total_delegated_amount)
		}

		/// Storage call to increase the delegation amount of the candidate in the candidate pool
		fn increase_candidate_delegations(
			candidate: &T::AccountId,
			amount: &BalanceOf<T>,
		) -> DispatchResultWithValue<BalanceOf<T>> {
			let mut candidate_detail = Self::get_candidate(&candidate)?;
			let total_delegated_amount = candidate_detail.add_delegated_amount(*amount)?;
			CandidatePool::<T>::set(&candidate, Some(candidate_detail));

			Ok(total_delegated_amount)
		}

		/// Remove the delegation data between the candidate and the delegator
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

		/// Get delay duration based on the action type
		fn get_delay_action_duration(action_type: &DelayActionType) -> BlockNumberFor<T> {
			match action_type {
				DelayActionType::CandidateLeaved => T::DelayDeregisterCandidateDuration::get(),
				DelayActionType::CandidateUndelegated => T::DelayUndelegateCandidate::get(),
			}
		}

		/// Core logic to cancel action request
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

		/// Core logic to execute delay action request
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

		/// Core logic to create a delay action request
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

		/// Core logic to deregister the candidate
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

			// If there are reward points when candidate leaves the pool, send it to them
			let reward_points = RewardPoints::<T>::get(&candidate);
			if reward_points > Zero::zero() {
				Self::claim_reward_inner(candidate.clone(), reward_points);
			}

			// Removing any information related the registration of the candidate in the pool
			CandidatePool::<T>::remove(&candidate);

			Self::deposit_event(Event::CandidateRegistrationRemoved { candidate_id: candidate });

			Ok(())
		}

		/// Core logic to undelegate the candidate
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

		/// Core logic to add candidate delegator
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

		/// Core logic to register candidate
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
				Percent::from_rational(BalanceRate::<T>::get(), 1000) *
				total
		}

		/// Captures an epoch snapshot containing information about the active validators and their
		/// delegations. It iterates over the provided active validator set to populate the snapshot
		/// with validator IDs, their bonded amounts, and delegator details including the amount
		/// delegated. This function constructs and returns an EpochSnapshot object populated with
		/// the gathered data.
		pub fn capture_epoch_snapshot(
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

		/// This function updates the reward points for a validator and its delegators based on
		/// block production rewards.
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

		fn claim_reward_inner(claimer: T::AccountId, reward_points: BalanceOf<T>) {
			let _ = T::NativeBalance::mint_into(&claimer, reward_points);

			RewardPoints::<T>::remove(&claimer);

			Self::deposit_event(Event::RewardClaimed { claimer, total_reward: reward_points });
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
