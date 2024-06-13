use std::collections::BTreeMap;

use crate::{mock::*, *};
use constants::{
	AccountId, ACCOUNT_6, CANDIDATE_1, CANDIDATE_2, CANDIDATE_3, CANDIDATE_4, CANDIDATE_5,
	CANDIDATE_6, TEST_BLOCKS_PER_EPOCH,
};
use frame_support::{assert_ok, traits::FindAuthor};
use tests::{ros, test_helpers};
use types::{CandidateDelegationSet, EpochSnapshot};

fn is_active_validator_set_sorted<T: Config>(validators: &CandidateDelegationSet<T>) -> bool {
	validators.windows(2).all(|w| w[0].1 >= w[1].1)
}

fn find_author() -> Option<AccountId> {
	<mock::Test as pallet::Config>::FindAuthor::find_author::<Vec<_>>(Default::default())
}

#[test]
fn should_ok_return_sorted_winner_list() {
	let mut ext = TestExtBuilder::default();
	ext.epoch_duration(TEST_BLOCKS_PER_EPOCH).build().execute_with(|| {
		let active_validator_set = Dpos::select_active_validator_set();

		assert!(is_active_validator_set_sorted::<Test>(&active_validator_set));

		ext.run_to_block(TEST_BLOCKS_PER_EPOCH);

		assert_eq!(CurrentActiveValidators::<Test>::get().to_vec(), active_validator_set);
	});
}

#[test]
fn should_ok_epoch_snapshot_is_correct() {
	let mut ext = TestExtBuilder::default();

	// Ensure that the candiddate is registered with valid bond
	ext.epoch_duration(TEST_BLOCKS_PER_EPOCH).build().execute_with(|| {
		let active_validator_set = Dpos::select_active_validator_set();
		assert_eq!(
			Some(Dpos::get_epoch_snapshot(&active_validator_set)),
			Dpos::last_epoch_snapshot()
		);
	});
}

#[test]
fn should_ok_epoch_snapshot_update_with_new_candidates() {
	let mut ext = TestExtBuilder::default();
	ext.max_candidates(100)
		.min_active_validators(2)
		.epoch_duration(TEST_BLOCKS_PER_EPOCH)
		.genesis_candidates(vec![
			(CANDIDATE_1.id, 300),
			(CANDIDATE_2.id, 300),
			(CANDIDATE_3.id, 300),
		])
		.build()
		.execute_with(|| {
			ext.run_to_block(2);
			// Attemp to register as candidate without enough fund in the account
			assert_eq!(
				Dpos::get_epoch_snapshot(&Dpos::active_validators()),
				EpochSnapshot {
					validators: BTreeMap::from_iter(vec![
						(CANDIDATE_1.id, 300),
						(CANDIDATE_2.id, 300),
						(CANDIDATE_3.id, 300),
					]),
					delegations: BTreeMap::default()
				}
			);
			assert_eq!(
				Dpos::get_epoch_snapshot(&Dpos::active_validators())
					.validators
					.get(&CANDIDATE_4.id),
				None
			);
			test_helpers::register_new_candidate(CANDIDATE_4.id, CANDIDATE_4.balance, 500);

			ext.run_to_block(TEST_BLOCKS_PER_EPOCH);

			assert_eq!(
				Dpos::get_epoch_snapshot(&Dpos::active_validators())
					.validators
					.get(&CANDIDATE_4.id),
				Some(&500)
			);

			test_helpers::register_new_candidate(CANDIDATE_5.id, CANDIDATE_5.balance, 600);

			assert_eq!(
				Dpos::get_epoch_snapshot(&Dpos::active_validators())
					.validators
					.get(&CANDIDATE_5.id),
				None
			);

			ext.run_to_block_from(TEST_BLOCKS_PER_EPOCH, TEST_BLOCKS_PER_EPOCH);

			assert_eq!(
				Dpos::get_epoch_snapshot(&Dpos::active_validators())
					.validators
					.get(&CANDIDATE_5.id),
				Some(&600)
			);
		});
}

#[test]
fn should_ok_reward_distributed_for_validators() {
	// Simulation test for calculating the rewards for candidate and delegators
	// Based on the provided commission
	let mut ext = TestExtBuilder::default();

	// Ensure that the candiddate is registered with valid bond
	ext.balance_rate(100)
		.epoch_duration(TEST_BLOCKS_PER_EPOCH)
		.build()
		.execute_with(|| {
			let active_validator_set = Dpos::select_active_validator_set();
			assert!(Dpos::active_validators().len() > 0);
			assert_eq!(Dpos::active_validators(), active_validator_set);
			assert_eq!(
				Dpos::last_epoch_snapshot(),
				Some(test_helpers::get_genesis_epoch_snapshot(active_validator_set.clone()))
			);

			let mut epoch_rewards = vec![0; active_validator_set.len()];
			// Now we want to run for a certain number of rounds
			let rounds = 20;
			let epochs = rounds * TEST_BLOCKS_PER_EPOCH;

			for round in 0..epochs {
				let maybe_author = find_author();
				for (indx, (active_validator, bond, _)) in
					Dpos::active_validators().iter().enumerate()
				{
					if Some(*active_validator) == maybe_author {
						assert_eq!(
							Dpos::get_epoch_snapshot(&Dpos::active_validators())
								.validators
								.get(&active_validator),
							Some(bond)
						);
						// Calculate the rewards of the validator in every epoch
						epoch_rewards[indx] += Dpos::calculate_reward(
							*bond,
							<mock::Test as pallet::Config>::AuthorCommission::get(),
						);
					}
					// If the epoch ends...
					if round % TEST_BLOCKS_PER_EPOCH == 0 {
						// Check if the reward points are calculated correctly
						assert_eq!(Dpos::reward_points(active_validator), epoch_rewards[indx]);
					}
				}
				ext.next_block();
			}
		});
}

/// Only validators who staked more to be in the top ranks can produce
/// and receive rewards
#[test]
fn should_ok_only_top_validators_can_produce_and_receive_rewards() {
	let mut ext = TestExtBuilder::default();

	ext.genesis_candidates(vec![])
		.balance_rate(100)
		.min_candidate_bond(100)
		.max_active_validators(3)
		.epoch_duration(3)
		.build()
		.execute_with(|| {
			// Our initial state of the chain does not include the active set

			assert!(System::block_number() == 1);
			test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, 200);
			test_helpers::register_new_candidate(CANDIDATE_2.id, CANDIDATE_2.balance, 300);
			test_helpers::register_new_candidate(CANDIDATE_3.id, CANDIDATE_3.balance, 400);

			assert_eq!(Dpos::reward_points(CANDIDATE_1.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_2.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_3.id), 0);

			// At block 3, building a new active set

			ext.run_to_block(4);

			assert_eq!(Dpos::reward_points(CANDIDATE_1.id), 0);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission())
			);
			assert_eq!(Dpos::reward_points(CANDIDATE_3.id), 0);

			ext.next_block(); // block 5
			ext.next_block(); // block 6 - new epoch

			assert_eq!(Dpos::active_validators().len(), 3);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission())
			);
			test_helpers::register_new_candidate(CANDIDATE_4.id, CANDIDATE_4.balance, 500);
			test_helpers::register_new_candidate(CANDIDATE_5.id, CANDIDATE_5.balance, 600);
			test_helpers::register_new_candidate(CANDIDATE_6.id, CANDIDATE_6.balance, 700);

			ext.next_block(); // block 7
			ext.next_block(); // block 8

			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission())
			);
			assert_eq!(Dpos::reward_points(CANDIDATE_4.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_5.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_6.id), 0);

			ext.next_block(); // block 9 - new epoch
			assert_eq!(Dpos::active_validators().len(), 3);

			// Old validator set will stop producing blocks and receive reward from this epoch
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission() * 2)
			);
			// New validator set will replace because they have top delegations
			assert_eq!(Dpos::reward_points(CANDIDATE_4.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_5.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_6.id), 0);

			ext.next_block(); // block 10

			assert_eq!(Dpos::active_validators().len(), 3);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(Dpos::reward_points(CANDIDATE_4.id), 0);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_5.id),
				Dpos::calculate_reward(600, test_helpers::get_author_commission())
			);
			assert_eq!(Dpos::reward_points(CANDIDATE_6.id), 0);

			ext.next_block(); // block 11

			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_4.id),
				Dpos::calculate_reward(500, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_5.id),
				Dpos::calculate_reward(600, test_helpers::get_author_commission())
			);
			assert_eq!(Dpos::reward_points(CANDIDATE_6.id), 0);

			ext.next_block(); // block 12

			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			); // Author
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_4.id),
				Dpos::calculate_reward(500, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_5.id),
				Dpos::calculate_reward(600, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_6.id),
				Dpos::calculate_reward(700, test_helpers::get_author_commission())
			);

			ext.next_block(); // block 13

			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission() * 2)
			); // Author
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(300, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_3.id),
				Dpos::calculate_reward(400, test_helpers::get_author_commission() * 2)
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_4.id),
				Dpos::calculate_reward(500, test_helpers::get_author_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_5.id),
				Dpos::calculate_reward(600, test_helpers::get_author_commission()) * 2
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_6.id),
				Dpos::calculate_reward(700, test_helpers::get_author_commission())
			);
		});
}

/// Reward distribution simulation test for validators and delegators
/// This test go through number of fixed epoches and calculate the number of reward points
/// that the active validator and delegator receives every block produced.
#[test]
fn should_ok_reward_distributed_for_validators_and_delegators() {
	// Simulation test for calculating the rewards for candidate and delegators
	// Based on the provided commission
	let mut ext = TestExtBuilder::default();

	// Ensure that the candiddate is registered with valid bond
	ext.genesis_candidates(vec![])
		.balance_rate(100)
		.min_candidate_bond(100)
		.epoch_duration(3)
		.build()
		.execute_with(|| {
			// Start at block 1
			assert!(System::block_number() == 1);
			// - Register a new candidate 1, and the reward point is zero because
			test_helpers::register_new_candidate(CANDIDATE_1.id, CANDIDATE_1.balance, 200);
			// There is no block produced by the candidate yet
			assert_eq!(Dpos::reward_points(CANDIDATE_1.id), 0);

			// Go to next block
			ext.next_block();
			assert!(System::block_number() == 2);

			// Block 2: Register a new candidate 2, and the reward point is zero
			test_helpers::register_new_candidate(CANDIDATE_2.id, CANDIDATE_2.balance, 100);
			assert_eq!(Dpos::reward_points(CANDIDATE_2.id), 0);

			// New epoch start: Start collecting the active set based on the staked bond
			ext.next_block();
			assert!(System::block_number() == 3);

			assert_eq!(
				Dpos::last_epoch_snapshot(),
				Some(EpochSnapshot {
					delegations: BTreeMap::default(),
					validators: BTreeMap::from_iter(vec![
						(CANDIDATE_1.id, 200),
						(CANDIDATE_2.id, 100)
					])
				})
			);

			// Move to the next block 4
			ext.next_block();
			assert!(System::block_number() == 4);

			// Because the FindAuthor we are using is round robin, so candidate 2 is a block author
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission())
			);

			ext.next_block();
			assert!(System::block_number() == 5);

			// Now candidate 2 will be the block author and receive reward
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(100, test_helpers::get_author_commission())
			);

			// At this block height, ACCOUNT 6 delegate to the candidate 1
			// (Reward points are distributed in the next epoch 9)
			test_helpers::delegate_candidate(ACCOUNT_6.id, CANDIDATE_1.id, 300);

			ext.next_block();
			assert!(System::block_number() == 6);
			// New epoch start: New snap shot includes the deleagator
			assert_eq!(
				Dpos::last_epoch_snapshot(),
				Some(EpochSnapshot {
					delegations: BTreeMap::from_iter(vec![((ACCOUNT_6.id, CANDIDATE_1.id), 300)]),
					validators: BTreeMap::from_iter(vec![
						(CANDIDATE_1.id, 200),
						(CANDIDATE_2.id, 100)
					])
				})
			);
			// Candidate 1 is the block producer of this block height
			// Reward points are still updated using the last epoch snapshot
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission()) * 2
			);
			// Deleagor don't receive the reward at this stage yet as it has just been delegated
			assert_eq!(Dpos::reward_points(ACCOUNT_6.id), Dpos::calculate_reward(300, 0));
			assert_eq!(Balances::free_balance(ACCOUNT_6.id), ACCOUNT_6.balance - 300);

			// Now the snapshot is updated
			ext.next_block();
			assert!(System::block_number() == 7);

			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(100, test_helpers::get_author_commission()) * 2
			);
			assert_eq!(Dpos::reward_points(ACCOUNT_6.id), 0);

			ext.next_block();
			assert!(System::block_number() == 8);
			assert_eq!(
				Dpos::reward_points(ACCOUNT_6.id),
				Dpos::calculate_reward(300, test_helpers::get_delegator_commission())
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission()) * 3
			);

			// NEXT EPOCH STARTS: Now the snapshot is updated
			ext.next_block();
			assert!(System::block_number() == 9);

			ext.next_block();
			assert!(System::block_number() == 10);

			assert_eq!(
				Dpos::reward_points(ACCOUNT_6.id),
				Dpos::calculate_reward(300, test_helpers::get_delegator_commission()) * 2
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_2.id),
				Dpos::calculate_reward(100, test_helpers::get_author_commission()) * 3
			);

			// Account 6 try to claim its reward points
			assert_eq!(
				Dpos::reward_points(ACCOUNT_6.id),
				Dpos::calculate_reward(300, test_helpers::get_delegator_commission()) * 2
			);
			assert_ok!(Dpos::claim_reward(ros(ACCOUNT_6.id)));
			assert_eq!(
				Balances::free_balance(ACCOUNT_6.id),
				ACCOUNT_6.balance - 300 +
					Dpos::calculate_reward(300, test_helpers::get_delegator_commission()) * 2
			);
			assert_eq!(Dpos::reward_points(ACCOUNT_6.id), 0);

			ext.next_block();
			assert!(System::block_number() == 11);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_1.id),
				Dpos::calculate_reward(200, test_helpers::get_author_commission()) * 4
			);
			assert_eq!(Dpos::reward_points(ACCOUNT_6.id), 0);
		});
}

#[test]
fn should_ok_round_robin_style_return_author_in_test() {
	let mut ext = TestExtBuilder::default();
	ext.epoch_duration(TEST_BLOCKS_PER_EPOCH).build().execute_with(|| {
		let rounds = 1;

		for _ in 0..rounds * TEST_BLOCKS_PER_EPOCH {
			let maybe_author = find_author();
			assert_ne!(maybe_author, None);

			// Every round, there must be a validator found that is in the active validator set
			assert!(DEFAULT_ACTIVE_SET.iter().any(|(v, _)| maybe_author == Some(*v)));
			ext.next_block();
		}
	});
}
