use crate::{mock::*, *};
use constants::{CANDIDATE_1, CANDIDATE_2, CANDIDATE_3, CANDIDATE_4, CANDIDATE_5, CANDIDATE_6};
use frame::deps::frame_support::assert_ok;
use tests::{ros, test_helpers};

#[test]
fn should_ok_should_claim_rewards() {
	let mut ext = TestExtBuilder::default();

	ext.genesis_candidates(vec![])
		.balance_rate(1000)
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
				Dpos::calculate_reward(600, test_helpers::get_author_commission()) * 2
			);
			assert_eq!(
				Dpos::reward_points(CANDIDATE_6.id),
				Dpos::calculate_reward(700, test_helpers::get_author_commission())
			);

			// Multiple accounts claim the reward
			let (candidate_balance_1, reward_points_1) =
				(Balances::free_balance(CANDIDATE_1.id), Dpos::reward_points(CANDIDATE_1.id));
			let (candidate_balance_2, reward_points_2) =
				(Balances::free_balance(CANDIDATE_2.id), Dpos::reward_points(CANDIDATE_2.id));
			let (candidate_balance_3, reward_points_3) =
				(Balances::free_balance(CANDIDATE_3.id), Dpos::reward_points(CANDIDATE_3.id));
			let (candidate_balance_4, reward_points_4) =
				(Balances::free_balance(CANDIDATE_4.id), Dpos::reward_points(CANDIDATE_4.id));
			let (candidate_balance_5, reward_points_5) =
				(Balances::free_balance(CANDIDATE_5.id), Dpos::reward_points(CANDIDATE_5.id));
			let (candidate_balance_6, reward_points_6) =
				(Balances::free_balance(CANDIDATE_6.id), Dpos::reward_points(CANDIDATE_6.id));
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_1.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_1.id),
				candidate_balance_1 + reward_points_1
			);
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_2.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_2.id),
				candidate_balance_2 + reward_points_2
			);
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_3.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_3.id),
				candidate_balance_3 + reward_points_3
			);
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_4.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_4.id),
				candidate_balance_4 + reward_points_4
			);
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_5.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_5.id),
				candidate_balance_5 + reward_points_5
			);
			assert_ok!(Dpos::claim_reward(ros(CANDIDATE_6.id)));
			assert_eq!(
				Balances::free_balance(CANDIDATE_6.id),
				candidate_balance_6 + reward_points_6
			);

			assert_eq!(Dpos::reward_points(CANDIDATE_1.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_2.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_3.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_4.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_5.id), 0);
			assert_eq!(Dpos::reward_points(CANDIDATE_6.id), 0);

			Dpos::do_try_state();
		});
}
