use crate::{mock::*, *};
use constants::{TEN_THOUSAND_BALANCE, TEST_BLOCKS_PER_EPOCH};
use types::ActiveValidatorSet;

fn is_active_validator_set_sorted<T: Config>(validators: &ActiveValidatorSet<T>) -> bool {
	validators.windows(2).all(|w| w[0].1 >= w[1].1)
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
fn should_ok_distribute_reward_for_block_production() {
	let mut ext = TestExtBuilder::default();
	ext.epoch_duration(TEST_BLOCKS_PER_EPOCH).build().execute_with(|| {
		ext.run_to_block(TEST_BLOCKS_PER_EPOCH);

		for (indx, (active_validator, total_staked)) in
			Dpos::select_active_validator_set().iter().enumerate()
		{
			let validator_reward =
				Dpos::calculate_reward(*total_staked, AuthorCommission::<Test>::get());
			assert!(validator_reward <= *total_staked);
			let initial_bond = DEFAULT_ACTIVE_SET[indx].1;
			assert_eq!(
				Balances::free_balance(active_validator),
				TEN_THOUSAND_BALANCE - initial_bond + validator_reward
			);
		}
	});
}

#[test]
fn should_ok_round_robin_style_return_author_in_test() {
	use frame_support::traits::FindAuthor;

	let mut ext = TestExtBuilder::default();
	ext.epoch_duration(TEST_BLOCKS_PER_EPOCH).build().execute_with(|| {
		let rounds = 20;
		for _ in 0..rounds * TEST_BLOCKS_PER_EPOCH {
			let maybe_author = <Test as pallet_authorship::Config>::FindAuthor::find_author::<Vec<_>>(
				Default::default(),
			);
			assert_ne!(maybe_author, None);

			// Every round, there must be a validator found that is in the active validator set
			assert!(DEFAULT_ACTIVE_SET.iter().any(|(v, _)| maybe_author == Some(*v)));
		}
	});
}
