# Delegated Proof of Stake - System Design

## Pallets

- `pallet-dpos`: Managing core logic of the algorithm related to staking, delegation, validator selection and reward distribution
- `pallet-session`: Managing the session and round rotation logic.
- `pallet-authorship`: Managing the logic of finding the author of the current block.

## Constants

## Interfaces

### pallet-dpos

- `stake( T::Balance )`: Stake token to the validator.

  - If the validator `self-stake`, it will be viewed as `bond`.
  - External stakes will be considered as `delegations`.

- `delay_unstake( T::Balance )`: Remove stakes from the current validator.
  - Stake is only removed after a certain of blocks
  - Requiring a cooldown period for undelegation adds a layer of economic risk for malicious actors
  - Immediate removal of large amounts of stake can cause sudden and significant changes in the validator rankings and the overall network security
  - Removing delegated stakes from the validator can only happene in the next round (or 2).
- `set_auto_register()`: Automatically register as candidate when reach a minimum stake threshold.
  - Can only be called by the validator
- `delay_candidate_exit( T::AccountId )`: Leave the role of candidate from the active set.
  - Immediate exits could lead to a sudden reduction in the number of active validators, which might compromise network security and stability.
  - Critical actions need time for the network to adapt to the changes.
  - Validators who can leave immediately might behave maliciously and exit quickly to avoid penalties.
  - Validator has to wait until the next round (or 2) to leave the candidate pool.
- `cancel_candidate_exit( T::AccountId )`: Cancel the request to leave candidates.
  - Request `scheduled_leave_candidates` will be reset.
- `register_candidate( T::AccountId )`: Register the validator as a candidate to be delegated for the next selection round.
- `execute_block_reward_payout( )`: Distribute the reward back to the delegators and validators who produced the block.
  - Distribution strategy for the block reward.
