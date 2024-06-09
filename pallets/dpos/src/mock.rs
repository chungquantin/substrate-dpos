use crate::{self as pallet_dpos, BalanceOf, ReportNewValidatorSet};
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, FindAuthor, Hooks},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = u64;

// Configure a mock runtime to test the pallet. We use the simpler syntax here.
frame_support::construct_runtime! {
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		Dpos: pallet_dpos,
	}
}

parameter_types! {
	pub const MaxCandidates: u32 = 10;
	pub const MaxCandidateDelegators: u32 = 5;
	pub const ExistentialDeposit : u128 = 1;
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

pub struct AlwaysSeven;
impl FindAuthor<AccountId> for AlwaysSeven {
	fn find_author<'a, I>(_: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = ([u8; 4], &'a [u8])>,
	{
		Some(7)
	}
}

pub struct DoNothing;
impl ReportNewValidatorSet<AccountId> for DoNothing {
	fn report_new_validator_set(_: Vec<AccountId>) {}
}

impl pallet_authorship::Config for Test {
	type FindAuthor = AlwaysSeven;
	type EventHandler = ();
}

impl pallet_dpos::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type MaxCandidates = MaxCandidates;
	type MaxCandidateDelegators = MaxCandidateDelegators;
	type ReportNewValidatorSet = DoNothing;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
}

pub struct TestAccount {
	pub id: AccountId,
	pub balance: u128,
}

impl TestAccount {
	pub fn to_tuple(self) -> (AccountId, u128) {
		(self.id, self.balance)
	}
}

pub const ACCOUNT_1: TestAccount = TestAccount { id: 1, balance: 10 };
pub const ACCOUNT_2: TestAccount = TestAccount { id: 2, balance: 20 };
pub const ACCOUNT_3: TestAccount = TestAccount { id: 3, balance: 300 };
pub const ACCOUNT_4: TestAccount = TestAccount { id: 4, balance: 400 };
pub const ACCOUNT_5: TestAccount = TestAccount { id: 5, balance: 500 };
pub const ACCOUNT_6: TestAccount = TestAccount { id: 6, balance: 10_000 };

pub struct TestExtBuilder {
	epoch_duration: BlockNumberFor<Test>,
	min_candidate_bond: BalanceOf<Test>,
	max_delegate_count: u32,
	min_delegate_amount: BalanceOf<Test>,
	max_total_delegate_amount: BalanceOf<Test>,
}

impl Default for TestExtBuilder {
	fn default() -> Self {
		Self {
			epoch_duration: 10,
			min_candidate_bond: 10,
			max_delegate_count: 4,
			min_delegate_amount: 10,
			max_total_delegate_amount: 300,
		}
	}
}

impl TestExtBuilder {
	#[allow(dead_code)]
	pub fn epoch_duration(&mut self, epoch_duration: BlockNumberFor<Test>) -> &mut Self {
		self.epoch_duration = epoch_duration;
		self
	}
	#[allow(dead_code)]
	pub fn min_candidate_bond(&mut self, min_candidate_bond: BalanceOf<Test>) -> &mut Self {
		self.min_candidate_bond = min_candidate_bond;
		self
	}
	#[allow(dead_code)]
	pub fn max_delegate_count(&mut self, max_delegate_count: u32) -> &mut Self {
		self.max_delegate_count = max_delegate_count;
		self
	}
	#[allow(dead_code)]
	pub fn min_delegate_amount(&mut self, min_delegate_amount: BalanceOf<Test>) -> &mut Self {
		self.min_delegate_amount = min_delegate_amount;
		self
	}
	#[allow(dead_code)]
	pub fn max_total_delegate_amount(
		&mut self,
		max_total_delegate_amount: BalanceOf<Test>,
	) -> &mut Self {
		self.max_total_delegate_amount = max_total_delegate_amount;
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
				// This allows us to have a total_payout different from 0.
				(999, 1_000_000_000_000),
			],
		}
		.assimilate_storage(&mut storage);

		let _ = pallet_dpos::GenesisConfig::<Test> {
			epoch_duration: self.epoch_duration,
			min_candidate_bond: self.min_candidate_bond,
			max_delegate_count: self.max_delegate_count,
			min_delegate_amount: self.min_delegate_amount,
			max_total_delegate_amount: self.max_total_delegate_amount,
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
}
