use crate::{
	self as pallet_dpos,
	constants::{AccountId, Balance, *},
	types::CandidatePool,
	BalanceOf, ReportNewValidatorSet,
};
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, FindAuthor, Hooks},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot};
use lazy_static::lazy_static;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet. We use the simpler syntax here.
frame_support::construct_runtime! {
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		Dpos: pallet_dpos,
	}
}

parameter_types! {
	pub static MaxCandidates: u32 = 20;
	pub static MaxCandidateDelegators: u32 = 5;
	pub static ExistentialDeposit : u128 = 1;
	pub static MaxActiveValidators: u32 = 10;
	pub static MinActiveValidators: u32 = 1;
	pub static MaxDelegateCount : u32 = 5;
	pub static Author: AccountId = 7;
}

pub const REGISTRATION_HOLD_AMOUNT: u128 = 200;

lazy_static! {
	pub static ref DEFAULT_ACTIVE_SET: CandidatePool<Test> = vec![
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
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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

pub struct DynamicAuthor;
impl FindAuthor<AccountId> for DynamicAuthor {
	fn find_author<'a, I>(_: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = ([u8; 4], &'a [u8])>,
	{
		Some(Author::get())
	}
}

pub struct DoNothing;
impl ReportNewValidatorSet<AccountId> for DoNothing {
	fn report_new_validator_set(_: Vec<AccountId>) {}
}

impl pallet_authorship::Config for Test {
	type FindAuthor = DynamicAuthor;
	type EventHandler = ();
}

impl pallet_dpos::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type MaxCandidates = MaxCandidates;
	type MaxCandidateDelegators = MaxCandidateDelegators;
	type ReportNewValidatorSet = DoNothing;
	type ForceOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxActiveValidators = MaxActiveValidators;
	type MinActiveValidators = MinActiveValidators;
	type MaxDelegateCount = MaxDelegateCount;
}

pub struct TestExtBuilder {
	epoch_duration: BlockNumberFor<Test>,
	min_candidate_bond: BalanceOf<Test>,
	min_delegate_amount: BalanceOf<Test>,
	gensis_candidates: CandidatePool<Test>,
	delay_deregister_candidate_duration: BlockNumberFor<Test>,
	delay_reward_payout_sent: BlockNumberFor<Test>,
	delay_undelegate_candidate: BlockNumberFor<Test>,
	validator_commission: u8,
	delegator_commission: u8,
}

impl Default for TestExtBuilder {
	fn default() -> Self {
		Self {
			epoch_duration: TEST_BLOCKS_PER_EPOCH,
			min_candidate_bond: 10,
			min_delegate_amount: 10,
			gensis_candidates: DEFAULT_ACTIVE_SET.to_vec(),
			delay_deregister_candidate_duration: TEST_BLOCKS_PER_EPOCH,
			delay_reward_payout_sent: TEST_BLOCKS_PER_EPOCH,
			delay_undelegate_candidate: TEST_BLOCKS_PER_EPOCH,
			validator_commission: 3,
			delegator_commission: 1,
		}
	}
}

impl TestExtBuilder {
	#[allow(dead_code)]
	pub fn epoch_duration(&mut self, epoch_duration: BlockNumberFor<Test>) -> &mut Self {
		self.epoch_duration = epoch_duration;
		self
	}

	pub fn min_candidate_bond(&mut self, min_candidate_bond: BalanceOf<Test>) -> &mut Self {
		self.min_candidate_bond = min_candidate_bond;
		self
	}

	#[allow(dead_code)]
	pub fn genesis_candidates(&mut self, candidates: CandidatePool<Test>) -> &mut Self {
		self.gensis_candidates = candidates;
		self
	}

	pub fn min_delegate_amount(&mut self, min_delegate_amount: BalanceOf<Test>) -> &mut Self {
		self.min_delegate_amount = min_delegate_amount;
		self
	}

	#[allow(dead_code)]
	pub fn validator_commission(&mut self, validator_commission: u8) -> &mut Self {
		self.validator_commission = validator_commission;
		self
	}

	#[allow(dead_code)]
	pub fn delegator_commission(&mut self, delegator_commission: u8) -> &mut Self {
		self.delegator_commission = delegator_commission;
		self
	}

	#[allow(dead_code)]
	pub fn delay_deregister_candidate_duration(
		&mut self,
		duration: BlockNumberFor<Test>,
	) -> &mut Self {
		self.delay_deregister_candidate_duration = duration;
		self
	}

	#[allow(dead_code)]
	pub fn delay_undelegate_candidate(&mut self, duration: BlockNumberFor<Test>) -> &mut Self {
		self.delay_reward_payout_sent = duration;
		self
	}

	pub fn build(&self) -> sp_io::TestExternalities {
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
			epoch_duration: self.epoch_duration,
			min_candidate_bond: self.min_candidate_bond,
			min_delegate_amount: self.min_delegate_amount,
			genesis_candidates: self.gensis_candidates.clone(),
			delegator_commission: self.delegator_commission,
			validator_commission: self.validator_commission,
			delay_deregister_candidate_duration: self.delay_deregister_candidate_duration,
			delay_undelegate_candidate: self.delay_undelegate_candidate,
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);

		ext.execute_with(|| {
			System::set_block_number(1);
			Dpos::on_initialize(1);
		});

		ext
	}

	pub fn next_block() {
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Dpos::on_initialize(System::block_number());
	}

	pub fn run_to_block(n: BlockNumberFor<Test>) {
		while System::block_number() < n {
			if System::block_number() > 1 {
				Dpos::on_finalize(System::block_number());
				System::on_finalize(System::block_number());
			}
			TestExtBuilder::next_block();
		}
	}

	pub fn run_to_block_from(
		from: BlockNumberFor<Test>,
		n: BlockNumberFor<Test>,
	) -> BlockNumberFor<Test> {
		Self::run_to_block(from + n);
		from + n
	}
}
