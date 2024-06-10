use crate::{mock::*, *};
use constants::TEST_BLOCKS_PER_EPOCH;
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

		TestExtBuilder::run_to_block(TEST_BLOCKS_PER_EPOCH);

		assert_eq!(CurrentActiveValidators::<Test>::get().to_vec(), active_validator_set);
	});
}

#[test]
fn should_ok_return_custom_author_in_test() {
	use frame_support::traits::FindAuthor;
	// initially, our author is 7, as defined in the mock file.
	// Now our pallet will read 8 as the block author
	assert_eq!(
		<Test as pallet_authorship::Config>::FindAuthor::find_author::<Vec<_>>(Default::default()),
		Some(7)
	);

	mock::Author::set(8);

	// Now our pallet will read 8 as the block author
	assert_eq!(
		<Test as pallet_authorship::Config>::FindAuthor::find_author::<Vec<_>>(Default::default()),
		Some(8)
	);
}
