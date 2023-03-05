use crate as pallet_voting_system;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		VotingSystem: pallet_voting_system,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

frame_support::parameter_types! {
	pub const SignatureLength: u32 = 32;
}

impl pallet_voting_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SignatureLength = SignatureLength;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(root_key: u64) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_voting_system::GenesisConfig::<Test> {
		central_authority: Some(root_key),
		candidates: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
		ballot_public_key: vec![1, 2, 3],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

pub fn new_test_ext_w_candidate(
	root_key: u64,
	candidate: Vec<<Test as system::Config>::AccountId>,
) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_voting_system::GenesisConfig::<Test> {
		central_authority: Some(root_key),
		candidates: candidate,
		ballot_public_key: vec![1, 2, 3],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
