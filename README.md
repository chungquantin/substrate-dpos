# Direct Delegated Proof of Stake Pallet
---
## Selected Challenge: Challenge Option 1: Direct Delegation Proof of Stake

The Substrate DPoS Pallet provides a Delegated Proof of Stake mechanism for a Substrate-based
blockchain. It allows token holders to delegate their tokens to validators who are responsible
for producing blocks and securing the network.

## Overview

The DPoS pallet implements a governance mechanism where stakeholders can elect a set of
validators to secure the network. Token holders delegate their stake to validators, who then
participate in the block production process. This pallet includes functionality for delegating
stake, selecting validators, and handling rewards and penalties.

## Table of Contents

- [Direct Delegated Proof of Stake Pallet](#direct-delegated-proof-of-stake-pallet)
  - [Overview](#overview)
  - [Table of Contents](#table-of-contents)
  - [Terminology](#terminology)
  - [Goals](#goals)
    - [_Optional_](#optional)
  - [Considerations](#considerations)
  - [Implementation Details](#implementation-details)
    - [Storage Types](#storage-types)
      - [Genesis \& Runtime Configuration](#genesis--runtime-configuration)
      - [Dispatchable Functions](#dispatchable-functions)
      - [Force Origin: Dispatchable Functions](#force-origin-dispatchable-functions)
    - [Scenarios](#scenarios)
      - [Candidate Regristration](#candidate-regristration)
      - [Candidate Request to Leave Pool](#candidate-request-to-leave-pool)
      - [Delegation \& Undelegation](#delegation--undelegation)
      - [Slashing candidate](#slashing-candidate)
      - [Validator Election](#validator-election)
      - [Reward Distribution](#reward-distribution)
    - [Further Improvements](#further-improvements)
  - [How to run the test?](#how-to-run-the-test)
  - [How to add the pallet to your runtime?](#how-to-add-the-pallet-to-your-runtime)
    - [Adding a pallet to your dependency TOML file](#adding-a-pallet-to-your-dependency-toml-file)
    - [Configuring the Runtime to use the pallet](#configuring-the-runtime-to-use-the-pallet)
  - [How to build your runtime \& chainspec?](#how-to-build-your-runtime--chainspec)
  - [How to run `omni-node`?](#how-to-run-omni-node)

## Terminology

- `Candidate`: Node who want to register as a candidate. A candidate node can receive stake
  delegations from token holders (delegator). Becoming a candidate can participate into the
  delegation process and produce blocks to earn rewards.
- `Delegator`: Token holders who delegate their token to the validator in the candidate pool.
  Delegators can receive reward for blocks produced by the delegated active validators.
- `Delegating`: A process of the delegator to vote for the candidate for the next epoch's validator election using tokens.
- `Candidate Registeration`: A process of the validator registering itself as the candidate for the next epoch's validator election
- `Validator Election`: Choosing the top most delegated candidates from the candidate pool for the next epoch.
- `Commission`: The percentage that block author and its delegator receive for a successfully produced block.
- `Slash`: The punishment of an active validator if they misbehave.
- `Epoch`: A predefined period during which the set of active validators remains fixed. At the end of each epoch, a new set of validators can be elected based on the current delegations.
- `Bond`: Staked tokens are bonded, meaning they are locked for a certain period, which secures the network and aligns incentives.

## Goals

- The pallet should have logic to manage "validators" and "delegators".
  - Validators register themselves as potential block producers.
  - Any other user can use their tokens to delegate (vote) for the set of validators they want.
- Where every N blocks (a session), the current "winners" are selected, and `Validators` are
  updated.
- Block rewards should be given to the current block producer, which is one of the previously
  selected validators, given to you by `Validators` and the delegators who backed them.

### _Optional_

- Try to support delegation chains, where a delegator can delegate to another delegator.
- Think about and implement some kind of slashing for validators if they “misbehave”.
- Integrate the Session pallet rather than using Aura directly.

## Considerations

At the outset of the project, a primary concern is designing a network architecture that effectively safeguards against malicious activities and mitigates centralization risks within the proof-of-stake model.

## Implementation Details

To view the detailed documentation for the Pallet implementation. Please run the below command:

```
cargo doc
```

### Storage Types

- `CandidatePool`:Mapping the validator ID with the registered candidate detail.
- `CurrentActiveValidators`:Selected validators for the current epoch.
- `LastEpochSnapshot`: Snapshot of the last epoch data, including active validator set, total bonds, and delegations.
- `RewardPoints`: Stores total claimable rewards for each account (validator or delegator), updated per block.
- `DelegateCountMap`: Number of candidates that delegators have delegated to.
- `DelegationInfos`: Stores delegation information from delegator accounts to validator accounts.
- `CandidateDelegators`: Maximum number of delegators that a candidate can have.

- `DelayActionRequests`: Stores requests for delayed actions that need execution after a specified delay duration.
- `BalanceRate`: Stores the balance rate configuration for inflation rebalancing of the DPoS network.

#### Genesis & Runtime Configuration

- `MaxCandidates`: The maximum number of authorities that the pallet can hold. Candidate pool is bounded using this value.
- `MaxCandidateDelegators`: The maximum number of delegators that a candidate can have. If the number of delegators reaches the maximum, the delegator with the lowest amount will be replaced by the new delegator if the new delegation is higher.
- `MaxActiveValidators`: The maximum number of candidates in the active validator set. This parameter is used for selecting the top N validators from the candidate pool.
- `MinActiveValidators`: The minimum number of candidates in the active validator set. If there are not enough active validators, block production won't happen until there are enough validators, ensuring network stability.
- `MaxDelegateCount`: The maximum number of candidates that delegators can delegate their tokens to.
- `MinCandidateBond`: The minimum number of bond that a candidate needs to provide to register in the candidate pool.
- `MinDelegateAmount`: The minimum amount of delegated tokens that a delegator needs to provide for one candidate.
- `EpochDuration`: A predefined period during which the set of active validators remains fixed. At the end of each epoch, a new set of validators can be elected based on the current delegations.
- `DelayDeregisterCandidateDuration`: Number of blocks required for the `deregister_candidate` method to work.
- `DelayUndelegateCandidate`: Number of blocks required for the `undelegate_candidate` method to work.
- `DelegatorCommission`: Percentage of commission that the delegator receives for their delegations.

#### Dispatchable Functions

- `register_as_candidate`: Allows a node to register itself as a candidate in the DPOS network.
- `candidate_bond_more`: Increases the bond amount for an existing candidate.
- `candidate_bond_less`: Decreases the bond amount for an existing candidate.
- `delegate_candidate`: Allows a delegator to delegate tokens to a candidate.
- `force_deregister_candidate`: Allows an authorized origin to force deregister a candidate from the network.
- `force_undelegate_candidate`: Allows an authorized origin to force undelegate tokens from a candidate.
- `delay_undelegate_candidate`: Initiates a request to delay undelegation of tokens from a candidate.
- `execute_deregister_candidate`: Executes the delayed deregistration of a candidate initiated by an authorized origin.
- `cancel_deregister_candidate_request`: Cancels the delayed deregistration request initiated by an authorized origin.
- `execute_undelegate_candidate`: Executes the delayed undelegation of tokens from a candidate initiated by an authorized origin.
- `cancel_undelegate_candidate_request`: Cancels the delayed undelegation request initiated by an authorized origin.
- `claim_reward`: Allows an account to claim their accumulated reward points.

#### Force Origin: Dispatchable Functions

- `force_set_balance_rate`: Allows an authorized origin to set the balance rate for inflation rebalancing of the DPoS network.
- `force_report_new_validators`: Forces a report of new validators to update the network state.

### Scenarios

#### Candidate Regristration

Before being qualified for the validator election round running every epoch `EpochDuration`, `Candidate` is required to register with a specific amount (higher than `MinCandidateBond`) of tokens.

- One candidate can't register twice `CandidateAlreadyExist`.
- Tokens are held by the network to secure the position of the candidate in the pool. Reason why tokens of the candidate are held instead of freezed is because `HOLD` is better if we need to slash the candidate held amount later in the future for misbehaviours.

  > From Polkadot SDK Docks: "_Holds are designed to be infallibly slashed, meaning that any logic using a Freeze must handle the possibility of the frozen amount being reduced, potentially to zero._"

- Candidate can stake more via `candidate_bond_more` to increase the position in the validator election.
- Candidate can also stake less via `candidate_bond_less` to decrease the amount of bond held. However, if the candidate bond is below a `MinCandidateBond`, candidate will be removed automatically by the network. And there is a mechanism to handle the unclaimed reward points of the leaved candidate.

#### Candidate Request to Leave Pool

1. Create a Delay Action Request to deregister

   - Request to leave the candidate pool will return back the held tokens to the candidate. However, this won't happen immediately but will create a request delayed for `DelayDeregisterCandidateDuration`.

   To prevent malicious actions from the candidates that potentially leading to the Sybil-attack, the request will take `DelayDeregisterCandidateDuration` number of blocks before it can be executed.

   - Requests are stored in the `DelayActionRequests`.
   - To execute the delay action request, candidate has to call the dispatchable function. By this way, this gives the candidate a chance to cancel the request to stay in the pool using the pallet call `cancel_delay_deregister_candidate`

2. Execute the delay request after the delay duration `DelayDeregisterCandidateDuration`
   - Leaving the pool intentionally instead of being slashed does not restrict the candidate from registering later in the pool.
   - Token holders who delegated the candidates will be returned back all the delegated tokens for the leaved candidate.

**Offline Status**: Currently, in the pallet implementation, when a candidate requests to leave a pool, their `ValidatorStatus` field within `CandidateDetail` is switched to `ValidatorStatus::Offline`. Offline validators can still produce blocks and receive rewards in the current epoch but they will be excluded from the Active Validator Set until their status is turned back to Online.

If the request is canceled, the validator status is flipped back to `ValidatorStatus::Online`.

#### Delegation & Undelegation

- Token holders can only delegate registered candidates. The delegated amount must be above a `MinDelegateAmount`. This ensure that the candidate won't receive too many pennies delegation. In this version, `MinDelegateAmount` is fixed by the network.

**NOTE:** In this pallet, there is an attribute called `MaxDelegateCount` which can be set to `1` if we want it to be _Direct Delegated Proof of Stake_. If the value is set to be higher than one, it is a normal _Delegated Proof of Stake_ system that allows delegations on multiple candidates.

- **Delay Undelegation**: Similar to the delay deregistering, undelegate from the candidate also require the delegator to submit a delayed request to the network and wait for `DelayUndelegateCandidateDuration` before it can be executed.

This delay period ensures that the network remains stable and secure by preventing sudden and large-scale withdrawals, which could potentially destabilize the system.

#### Slashing candidate

Candidate who misbehaves will be slashed from the network and the handler hook that the runtime can interact with is `OnSlashHandler`. Slashing mechanism is configured by other pallets.

- The `do_slash` in the pallet implementation will slash a held amount of the candidate based on the provided amount from the external system.
- If the left amount after being slashed is under the threshold `MinCandidateStake`, the canddiate will be removed completely from the pool following the logic mentioned in the **"Candidate Request to Leave Pool"** section

#### Validator Election

- Top validators under `MaxActiveValidators` and above `MinAciveValidators` are selected based on the total amount of delegated amount and the total amount they bonded.
- If there is not enough validators (under the configured `MinActiveValidators`), the active validator set is empty. By this way, there is no block produced and no reward distributed.
- In this pallet, the top validators will be sorted out and selected at the beginning of the new epoch.
- Offline validators won't be included in the validator election.
- Non-selected candidates in the pool will stay inactive during the epoch and don't produce a new block and receive rewards during the epoch.

#### Reward Distribution

- Reward for every block produced won't be distributed automatically but requires the validators and delegators to claim it themself. There is no deadline for claiming the reward.
- To distribute the reward, the network capture snapshot of the active validator set with its bond and the delegations of those elected validators at the beginning of an epoch in `LastEpochSnapshot`.
- The purpose of the `EpochSnapshot` is to avoid state of the validators and delegators change in the middle of the epoch. By that way, the reward is calculated using the amount caputred in the snapshot.
- The reward need to be claimed by the network stakeholders by calling the method `claim_reward(who)`. Reward is distributed to the candidate automatically when they leave pool.

- **Reward calculation and its related parameters**:

  - `AuthorCommission` and `DelegatorCommission` are the two commission percentages configured by the `ConfigControllerOrigin` to use for reward calculation.
  - Likewise, `BalanceFactor` is also managed by `ConfigControllerOrigin` and is used for controlling the inflation rate via reward of the delegators and validators.

  The `BalanceFactor` directly influences the rate at which new tokens are introduced into the system as rewards for validators and delegators. A higher BalanceFactor leads to higher inflation, as more tokens are distributed as rewards. Conversely, a lower BalanceFactor restricts inflation, resulting in fewer tokens being distributed.

The formula this pallet use to calculate the reward is:
$$P*S*B=R$$
With:

- $P$: Stands for the commission Percantage each actor is configured
- $S$: Total staked of the validators or the total delegations of the delegator on the active validator.
- $B$: Balance factor
- $R$: Final calculated reward

### Further Improvements

- #### [Delay Action] Limited number of accepted offline epochs

The delay action feature can be enhanced by limiting the number of epochs a validator node can remain offline. Allowing a validator node to stay offline indefinitely without being removed can destabilize the network by allocating resources to an inactive node.

- A new config type `MaxOfflineEpochs` and a new logic for removing the deprecated offline nodes can be added to implement this feature.

- #### [Slashing] Prevent candidate from joining the network without free participation after being slashed

- #### Multi delegation instead of direct delegation

Instead of limiting the delegation model to direct delegation, it can be expanded to foster greater network engagement among delegators through multi-delegation. This approach encourages delegators to participate in multiple network activities, thereby enhancing their involvement and contributions to the ecosystem.

Multi-delegation reduces the risk of centralization by spreading stake among several validators. This diversification enhances network resilience against potential attacks or failures from single validators, thereby improving overall security and reliability.

- #### Dynamic active validator set rotation

The current selection algorithm only focuses on the top staked validators which can lead to the centralization in the network participation. To address this, implementing a dynamic active validator set rotation mechanism is crucial for fostering decentralization and ensuring a more inclusive participation across the network.

A suggested design can be calculating and ordering the candidate pool based on its stake weight $W_i$ instead of its total staked $S$. Stake weight is calculated by

$$W_i = \frac{\text{Tokens staked with validator } i}{\text{Total Staked Tokens}} \times 100$$

with $T$ is the total delegations and $S$ is the total staked. By that way, we can choose the validator in a more balanced approach.

- #### Prevent cascading deletion when a Candidate is removed

How to distribute reward

- #### Mechanism to accept top delegations / bottom delegation

Previously, we discussed the issue of "penny" delegation, which MinDelegateAmount helps manage but doesn't ensure the delegation pool consistently meets the required threshold.

To address this, we propose implementing two lists of delegations: `CandidateTopDelegations` and `CandidateBottomDelegations`. Both are stored as maps where the key is CandidateId, and each contains a vector (Vec) of `DelegationInfo`.

When `CandidateTopDelegations` reaches its capacity, the least contributing delegators are moved to `CandidateBottomDelegations`. This process frees up space in the top list for delegators with higher delegated amounts. If a delegator is moved from `CandidateTopDelegations` to `CandidateBottomDelegations`, they can move back to the top list if their delegated amount increases, allowing them to be eligible for rewards when the candidate is elected.

This approach ensures that the delegation pool prioritizes delegators with substantial contributions while providing a mechanism for others to potentially move up based on their delegation amounts over time.

---

## How to run the test?

```
cd ./pallets/dpos & cargo test
```

Run with runtime bechmarks

```
cd ./pallets/dpos & cargo test --features runtime-benchmarks
```

## How to add the pallet to your runtime?

Follow the guidelines to add the pallet to your runtime.

### Adding a pallet to your dependency TOML file

```toml
pallet-dpos = { path = "../pallets/dpos", default-features = false }
```

### Configuring the Runtime to use the pallet

```rs
impl pallet_dpos::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type MaxCandidates = MaxCandidates;
	type MaxCandidateDelegators = MaxCandidateDelegators;
	type MaxActiveValidators = MaxActivevalidators;
	type MinActiveValidators = MinActiveValidators;
	type ReportNewValidatorSet = StoreNewValidatorSet;
	type WeightInfo = ();
	type OnSlashHandler = OnSlashHandler;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxDelegateCount = MaxDelegateCount;
	type DelayDeregisterCandidateDuration = DelayDeregisterCandidateDuration;
	type DelayUndelegateCandidate = DelayUndelegateCandidate;
	type EpochDuration = EpochDuration;
	type MinCandidateBond = MinCandidateBond;
	type MinDelegateAmount = MinDelegateAmount;
	type AuthorCommission = ValidatorCommission;
	type DelegatorCommission = DelegatorCommission;
	type FindAuthor = BlockAuthor;
	type ForceOrigin = EnsureRoot<AccountId>;
	type ConfigControllerOrigin = EnsureRoot<AccountId>;
}
```

Here are the list of example parameters that you can configure for your runtime:

```rs
parameter_types! {
	pub const MaxCandidates : u32 = 200;
	pub const MaxCandidateDelegators : u32 = 300;
	pub const MinCandidateBond: u32 = 1_000;
	pub const MaxActivevalidators: u32 = 100;
	pub const MinActiveValidators: u32 = 3;
	pub const MaxDelegateCount : u32 = 30;
	pub const EpochDuration : u32 = EPOCH_DURATION;
	pub const DelayDeregisterCandidateDuration : u32 = EPOCH_DURATION * 2;
	pub const DelayUndelegateCandidate : u32 = EPOCH_DURATION;
	pub const MinDelegateAmount : u128 = 150;
	pub const ValidatorCommission : u8 = 5;
	pub const DelegatorCommission : u8 = 3;
}
```

Add this to your `chainspec.json`

```json
{
  "balances": {
    "balances": [
      ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 100000],
      ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 100000],
      ["5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 100000],
      ["5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", 100000],
      ["5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw", 100000],
      ["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL", 100000],
      ["5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 100000],
      ["5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 100000],
      ["5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT", 100000],
      ["5HKPmK9GYtE1PSLsS1qiYU9xQ9Si1NcEhdeCq9sw5bqu4ns8", 100000],
      ["5FCfAonRZgTFrTd9HREEyeJjDpT397KMzizE6T3DvebLFE7n", 100000],
      ["5CRmqmsiNFExV6VbdmPJViVxrWmkaXXvBrSX8oqBT8R9vmWk", 100000],
      ["5Fxune7f71ZbpP2FoY3mhYcmM596Erhv1gRue4nsPwkxMR4n", 100000],
      ["5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 100000]
    ]
  },
  "dpos": {
    "balanceRate": 1000,
    "genesisCandidates": [
      ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 100000],
      ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 100000],
      ["5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 100000],
      ["5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", 100000],
      ["5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw", 100000],
      ["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL", 100000],
      ["5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 100000],
      ["5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 100000],
      ["5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT", 100000],
      ["5HKPmK9GYtE1PSLsS1qiYU9xQ9Si1NcEhdeCq9sw5bqu4ns8", 100000],
      ["5FCfAonRZgTFrTd9HREEyeJjDpT397KMzizE6T3DvebLFE7n", 100000],
      ["5CRmqmsiNFExV6VbdmPJViVxrWmkaXXvBrSX8oqBT8R9vmWk", 100000],
      ["5Fxune7f71ZbpP2FoY3mhYcmM596Erhv1gRue4nsPwkxMR4n", 100000],
      ["5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 100000]
    ]
  }
}
```

## How to build your runtime & chainspec?

```md
cd ./runtime

# Build the runtime

cargo build --release

# Generate chain-spec

chain-spec-builder create --chain-name DPOSChain -r ../target/release/wbuild/pba-runtime/pba_runtime.wasm default
```

## How to run `omni-node`?

```
pba-omni-node --chain ./runtime/chain_spec.json --tmp
```
