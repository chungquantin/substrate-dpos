use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct CandidateDetail<Balance, BlockNumber> {
	pub bond: Balance,
	pub total_delegations: Balance,
	pub registered_at: BlockNumber,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct CandidateRegitrationRequest<AccountId, Balance> {
	pub request_by: AccountId,
	pub bond: Balance,
}
