pub type Balance = u128;
pub type AccountId = u64;

#[derive(Debug)]
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
// Candidate Accounts
pub const CANDIDATE_1: TestAccount = TestAccount { id: 101, balance: 10_000 };
pub const CANDIDATE_2: TestAccount = TestAccount { id: 102, balance: 10_000 };
pub const CANDIDATE_3: TestAccount = TestAccount { id: 103, balance: 10_000 };
pub const CANDIDATE_4: TestAccount = TestAccount { id: 104, balance: 10_000 };
pub const CANDIDATE_5: TestAccount = TestAccount { id: 105, balance: 10_000 };
pub const CANDIDATE_6: TestAccount = TestAccount { id: 106, balance: 10_000 };
pub const CANDIDATE_7: TestAccount = TestAccount { id: 107, balance: 10_000 };
pub const CANDIDATE_8: TestAccount = TestAccount { id: 108, balance: 10_000 };
pub const CANDIDATE_9: TestAccount = TestAccount { id: 109, balance: 10_000 };
pub const CANDIDATE_10: TestAccount = TestAccount { id: 110, balance: 10_000 };
pub const CANDIDATE_11: TestAccount = TestAccount { id: 111, balance: 10_000 };
pub const CANDIDATE_12: TestAccount = TestAccount { id: 112, balance: 10_000 };
pub const CANDIDATE_13: TestAccount = TestAccount { id: 113, balance: 10_000 };
pub const CANDIDATE_14: TestAccount = TestAccount { id: 114, balance: 10_000 };
