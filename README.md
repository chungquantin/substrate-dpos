[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/K4L-g2pg)

# Direct Delegated Proof of Stake Pallet

The Substrate DPoS Pallet provides a Delegated Proof of Stake mechanism for a Substrate-based
blockchain. It allows token holders to delegate their tokens to validators who are responsible
for producing blocks and securing the network.

## Overview

The DPoS pallet implements a governance mechanism where stakeholders can elect a set of
validators to secure the network. Token holders delegate their stake to validators, who then
participate in the block production process. This pallet includes functionality for delegating
stake, selecting validators, and handling rewards and penalties. Moreover, this pallet also
provides the ability to switch between **Direct Delegation mode** and **Multi Delegation mode**

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
- Some parameters are used in the **Reward calculation**:
  - `BalanceFactor` is used for controlling the inflation rate via reward of the delegators and validators.
  - `AuthorCommission` and `DelegatorCommission` are the two commission percentages configured by the `ConfigControllerOrigin` to use for reward calculation. 

### Further Improvements

#### [Delay Action] Limited number of accepted offline epochs

The delay action feature can be enhanced by limiting the number of epochs a validator node can remain offline. Allowing a validator node to stay offline indefinitely without being removed can destabilize the network by allocating resources to an inactive node.

- A new config type `MaxOfflineEpochs` and a new logic for removing the deprecated offline nodes can be added to implement this feature.

#### [Slashing] Prevent candidate from joining the network without free participation after being slashed

#### [Reward distribution]

#### Multi delegation instead of direct delegation

#### Reconfigurable network parameters

#### Prevent cascading deletion when a Candidate is removed

## Game Theory & Economic Model

How to distribute reward

## Compromises

### Reward Distribution

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
