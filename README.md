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
- `MinCandidateBond`: The minimum number of stake that a candidate needs to provide to register in the candidate pool.
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
- Tokens are held by the network to secure the position of the candidate in the pool.
- Candidate can stake more via `candidate_bond_more` to increase the position in the validator election.
- Candidate can also stake less via `candidate_bond_less` to decrease the amount of bond held. However, if the candidate bond is below a `MinCandidateBond`, candidate will be removed automatically by the network.

#### Candidate Request to Leave Pool

- Request to leave the candidate pool will return back the tokens to the candidate. However, this won't happen immediately but will create a request delayed for `DelayDeregisterCandidateDuration`.
- Leaving the pool intentionally instead of being slashed does not restrict the candidate from registering later in the pool.

#### Delegation & Undelegation

- Delay unstaking
- Delay deregister
- Reward distribution

#### Slashing candidate

#### Validator Election

- How to selelct top active candidators?
- What I implement and what can be improvied
- Below the minimum active validators
- Misbehaves

#### Reward Distribution

The reward will be paid based on the bond and delegations

### Further Improvements

#### Slashing will prevent candidate from joining the network without free participation

#### Better reward distribution

#### Multi delegation instead of direct delegation

#### Reconfigurable network parameters

#### Prevent cascading deletion when a Candidate is removed

#### Mark the validator as offline upon its request to leave the pool.

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
