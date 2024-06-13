[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/K4L-g2pg)

 # Delegated Proof of Stake (DPOS) Pallet

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

 - [`Candidate`]: Node who want to register as a candidate. A candidate node can receive stake
   delegations from token holders (delegator). Becoming a candidate can participate into the
   delegation process and produce blocks to earn rewards.
 - [`Delegator`]: Token holders who delegate their token to the validator in the candidate pool.
   Delegators can receive reward for blocks produced by the delegated active validators.
 - [`Delegating`]: A process of the delegator to vote for the candidate for the next epoch's validator election using tokens.
 - [`Candidate Registeration`]: A process of the validator registering itself as the candidate for the next epoch's validator election
 - [`Validator Election`]: Choosing the top most delegated candidates from the candidate pool for the next epoch.
 - [`Commission`]: The percentage that block author and its delegator receive for a successfully produced block.
 - [`Slash`]: The punishment of an active validator if they misbehave. 
 - [`Epoch`]: A predefined period during which the set of active validators remains fixed. At the end of each epoch, a new set of validators can be elected based on the current delegations.
---
 ## Implementation Details
 ### Pallet Design
 #### Dispatchable Functions
 #### Runtime Configuration
 #### Hooks
 ### Scenarios

 ### A Note on Upgrades

 ## Economic Model

 ## Compromises

 ## Improvements

 ## Usage