use crate::{
	self as pallet_dpos,
	constants::{AccountId, Balance, *},
	types::CandidateSet,
	BalanceOf, OnSlashHandler, ReportNewValidatorSet,
};
use frame::{
	deps::{
		frame_support::{
			derive_impl, parameter_types,
			traits::{ConstU16, ConstU32, ConstU64, FindAuthor, Hooks},
		},
		frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot},
	},
	prelude::*,
};
use lazy_static::lazy_static;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame::deps::frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet. We use the simpler syntax here.
frame::deps::frame_support::construct_runtime! {
	pub struct Test {
		System: frame::deps::frame_system,
		Balances: pallet_balances,
		Dpos: pallet_dpos,
	}
}

parameter_types! {
	pub static MaxCandidates: u32 = 200;
	pub static MaxCandidateDelegators: u32 = 5;
	pub static ExistentialDeposit : u128 = 1;
	pub static MaxActiveValidators: u32 = 10;
	pub static MinActiveValidators: u32 = 1;
	pub static MaxDelegateCount : u32 = 20;
	pub static DelayDeregisterCandidateDuration : u64 = TEST_BLOCKS_PER_EPOCH;
	pub static DelayUndelegateCandidate : u64 = TEST_BLOCKS_PER_EPOCH;
	pub static EpochDuration : u64 = TEST_BLOCKS_PER_EPOCH;
	pub static MinCandidateBond : u128 = 10;
	pub static MinDelegateAmount : u128 = 10;
	pub static ValidatorCommission : u32 = 3; // 0.3
	pub static DelegatorCommission : u32 = 1; // 0.1
}

pub const REGISTRATION_HOLD_AMOUNT: u128 = 200;

lazy_static! {
	pub static ref DEFAULT_ACTIVE_SET: CandidateSet<Test> = vec![
		(CANDIDATE_1.id, REGISTRATION_HOLD_AMOUNT),
		(CANDIDATE_4.id, REGISTRATION_HOLD_AMOUNT * 3),
		(CANDIDATE_11.id, REGISTRATION_HOLD_AMOUNT * 6),
		(CANDIDATE_12.id, REGISTRATION_HOLD_AMOUNT * 6),
		(CANDIDATE_5.id, REGISTRATION_HOLD_AMOUNT * 3),
		(CANDIDATE_6.id, REGISTRATION_HOLD_AMOUNT * 4),
		(CANDIDATE_7.id, REGISTRATION_HOLD_AMOUNT * 4),
		(CANDIDATE_13.id, REGISTRATION_HOLD_AMOUNT * 7),
		(CANDIDATE_2.id, REGISTRATION_HOLD_AMOUNT * 2),
		(CANDIDATE_3.id, REGISTRATION_HOLD_AMOUNT * 2),
		(CANDIDATE_14.id, REGISTRATION_HOLD_AMOUNT * 7),
		(CANDIDATE_8.id, REGISTRATION_HOLD_AMOUNT * 5),
		(CANDIDATE_9.id, REGISTRATION_HOLD_AMOUNT * 5),
		(CANDIDATE_10.id, REGISTRATION_HOLD_AMOUNT * 6),
	];
}

// Feel free to remove more items from this, as they are the same as
// `frame_system::config_preludes::TestDefaultConfig`. We have only listed the full `type` list here
// for verbosity. Same for `pallet_balances::Config`.
// https://paritytech.github.io/polkadot-sdk/master/frame_support/attr.derive_impl.html
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame::deps::frame_system::Config for Test {
	type BaseCallFilter = frame::deps::frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame::deps::frame_support::traits::ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<10>;
}

pub struct RoundRobinAuthor;
impl FindAuthor<AccountId> for RoundRobinAuthor {
	fn find_author<'a, I>(_: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = ([u8; 4], &'a [u8])>,
	{
		let current_active_validators = Dpos::active_validators();
		let active_validator_ids = current_active_validators
			.iter()
			.map(|(id, _, _)| *id)
			.collect::<Vec<AccountId>>();

		if active_validator_ids.len() == 0 {
			return None;
		}
		active_validator_ids
			.get((System::block_number() % (active_validator_ids.len() as u64)) as usize)
			.cloned()
	}
}

pub struct DoNothing;
impl ReportNewValidatorSet<AccountId> for DoNothing {
	fn report_new_validator_set(_: Vec<AccountId>) {}
}
impl OnSlashHandler<AccountId, Balance> for DoNothing {
	fn on_slash(_who: &AccountId, _balance: Balance) {}
}

impl pallet_dpos::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type MaxCandidates = MaxCandidates;
	type MaxCandidateDelegators = MaxCandidateDelegators;
	type ReportNewValidatorSet = DoNothing;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxActiveValidators = MaxActiveValidators;
	type MinActiveValidators = MinActiveValidators;
	type MaxDelegateCount = MaxDelegateCount;
	type DelayDeregisterCandidateDuration = DelayDeregisterCandidateDuration;
	type DelayUndelegateCandidate = DelayUndelegateCandidate;
	type EpochDuration = EpochDuration;
	type MinCandidateBond = MinCandidateBond;
	type MinDelegateAmount = MinDelegateAmount;
	type AuthorCommission = ValidatorCommission;
	type DelegatorCommission = DelegatorCommission;
	type FindAuthor = RoundRobinAuthor;
	type ForceOrigin = EnsureRoot<AccountId>;
	type ConfigControllerOrigin = EnsureRoot<AccountId>;
	type OnSlashHandler = DoNothing;
}

pub struct TestExtBuilder {
	gensis_candidates: CandidateSet<Test>,
	balance_rate: u32,
	reward_distribution_disabled: bool,
}

impl Default for TestExtBuilder {
	fn default() -> Self {
		Self {
			gensis_candidates: DEFAULT_ACTIVE_SET.to_vec(),
			reward_distribution_disabled: false,
			balance_rate: 1000,
		}
	}
}

#[allow(dead_code)]
impl TestExtBuilder {
	#[allow(dead_code)]
	pub fn epoch_duration(&mut self, epoch_duration: BlockNumberFor<Test>) -> &mut Self {
		EpochDuration::set(epoch_duration);
		self
	}

	pub fn min_candidate_bond(&mut self, min_candidate_bond: BalanceOf<Test>) -> &mut Self {
		MinCandidateBond::set(min_candidate_bond);
		self
	}

	pub fn max_active_validators(&mut self, max_active_validators: u32) -> &mut Self {
		MaxActiveValidators::set(max_active_validators);
		self
	}

	pub fn genesis_candidates(&mut self, candidates: CandidateSet<Test>) -> &mut Self {
		self.gensis_candidates = candidates;
		self
	}

	pub fn balance_rate(&mut self, balance_rate: u32) -> &mut Self {
		self.balance_rate = balance_rate;
		self
	}

	pub fn max_candidates(&mut self, max_candidates: u32) -> &mut Self {
		MaxCandidates::set(max_candidates);
		self
	}

	pub fn min_active_validators(&mut self, min_active_validators: u32) -> &mut Self {
		MinActiveValidators::set(min_active_validators);
		self
	}

	pub fn min_delegate_amount(&mut self, min_delegate_amount: BalanceOf<Test>) -> &mut Self {
		MinDelegateAmount::set(min_delegate_amount);
		self
	}

	pub fn max_candidate_delegators(&mut self, max_candidate_delegators: u32) -> &mut Self {
		MaxCandidateDelegators::set(max_candidate_delegators);
		self
	}

	pub fn max_delegate_count(&mut self, max_delegate_count: u32) -> &mut Self {
		MaxDelegateCount::set(max_delegate_count);
		self
	}

	pub fn validator_commission(&mut self, validator_commission: u32) -> &mut Self {
		ValidatorCommission::set(validator_commission);
		self
	}

	pub fn reward_distribution_disabled(&mut self) -> &mut Self {
		self.reward_distribution_disabled = true;
		self
	}

	pub fn delegator_commission(&mut self, delegator_commission: u32) -> &mut Self {
		DelegatorCommission::set(delegator_commission);
		self
	}

	pub fn delay_deregister_candidate_duration(
		&mut self,
		duration: BlockNumberFor<Test>,
	) -> &mut Self {
		DelayDeregisterCandidateDuration::set(duration);
		self
	}

	pub fn delay_undelegate_candidate(&mut self, duration: BlockNumberFor<Test>) -> &mut Self {
		DelayUndelegateCandidate::set(duration);
		self
	}

	fn with_storage(&self) -> sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				ACCOUNT_1.to_tuple(),
				ACCOUNT_2.to_tuple(),
				ACCOUNT_3.to_tuple(),
				ACCOUNT_4.to_tuple(),
				ACCOUNT_5.to_tuple(),
				ACCOUNT_6.to_tuple(),
				// Initializing the balances for active candidates
				CANDIDATE_1.to_tuple(),
				CANDIDATE_2.to_tuple(),
				CANDIDATE_3.to_tuple(),
				CANDIDATE_4.to_tuple(),
				CANDIDATE_5.to_tuple(),
				CANDIDATE_6.to_tuple(),
				CANDIDATE_7.to_tuple(),
				CANDIDATE_8.to_tuple(),
				CANDIDATE_9.to_tuple(),
				CANDIDATE_10.to_tuple(),
				CANDIDATE_11.to_tuple(),
				CANDIDATE_12.to_tuple(),
				CANDIDATE_13.to_tuple(),
				CANDIDATE_14.to_tuple(),
				// This allows us to have a total_payout different from 0.
				(999, 1_000_000_000_000),
			],
		}
		.assimilate_storage(&mut storage);

		let _ = pallet_dpos::GenesisConfig::<Test> {
			genesis_candidates: self.gensis_candidates.clone(),
			balance_rate: self.balance_rate,
		}
		.assimilate_storage(&mut storage);
		sp_io::TestExternalities::from(storage)
	}

	pub fn build(&self) -> sp_io::TestExternalities {
		let mut ext = self.with_storage();
		ext.execute_with(|| {
			System::set_block_number(1);
			Dpos::on_initialize(1);
		});

		ext
	}

	pub fn build_from_genesis(&self) -> sp_io::TestExternalities {
		let mut ext = self.with_storage();
		ext.execute_with(|| {
			if !self.reward_distribution_disabled {
				System::set_block_number(0);
				Dpos::on_initialize(0);
			}
		});

		ext
	}

	pub fn next_block(&self) {
		System::set_block_number(System::block_number() + 1);
		if !self.reward_distribution_disabled {
			System::on_initialize(System::block_number());
			Dpos::on_initialize(System::block_number());
		}
	}

	pub fn run_to_block(&self, n: BlockNumberFor<Test>) {
		while System::block_number() < n {
			if System::block_number() > 1 {
				if !self.reward_distribution_disabled {
					Dpos::on_finalize(System::block_number());
					System::on_finalize(System::block_number());
				}
			}
			self.next_block();
		}
	}

	pub fn run_to_block_from(
		&self,
		from: BlockNumberFor<Test>,
		n: BlockNumberFor<Test>,
	) -> BlockNumberFor<Test> {
		self.run_to_block(from + n);
		from + n
	}
}
