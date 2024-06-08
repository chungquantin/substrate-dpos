use crate::{self as pallet_dpos, weights::*, ReportNewValidatorSet};
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, FindAuthor, Hooks},
};
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
	pub const MinCandidateBond: u128 = 10;
	pub const EpochDuration: u32 = 100;
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
	type ReportNewValidatorSet = DoNothing;
	type WeightInfo = ();
	type MinCandidateBond = MinCandidateBond;
	type RuntimeHoldReason = RuntimeHoldReason;
	// Assuming blocks happen every 6 seconds, this will be 600 seconds, approximately 10 minutes.
	// But this is all just test config, but gives you an idea how this is all CONFIGURABLE
	type EpochDuration = EpochDuration;
}

pub struct TestExtBuilder;

impl Default for TestExtBuilder {
	fn default() -> Self {
		Self {}
	}
}

impl TestExtBuilder {
	pub fn build(&self) -> sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(1, 10),
				(2, 20),
				(3, 300),
				(4, 400),
				// This allows us to have a total_payout different from 0.
				(999, 1_000_000_000_000),
			],
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);

		ext.execute_with(|| {
			System::set_block_number(1);
			<Dpos as Hooks<u64>>::on_initialize(1);
		});

		ext
	}
}
