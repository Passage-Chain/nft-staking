# StakeRewards Smart Contract

## Overview

StakeRewards is a CosmWasm smart contract designed to manage external rewards for a staking system. It handles the distribution of rewards to users based on their staked amounts and the total staked amount in the system.

## Key Features

1. **Reward Distribution**: Calculates and distributes rewards to users based on their staked amounts.
2. **Time-based Rewards**: Implements a time-based reward system with a configurable duration.
3. **External Staking Integration**: Designed to work with an external staking contract.
4. **Claim Functionality**: Allows users to claim their accumulated rewards.

## Contract Structure

The main struct `StakeRewards` contains:

- `config`: Stores global configuration settings.
- `rewards`: Manages cumulative rewards data.
- `user_rewards`: Map of user addresses to their reward data.

## Key Functions

### Instantiate

- Initializes the contract with stake address, reward denomination, and duration.
- Sets up initial reward rate and period finish time.

### Execute Messages

1. `stake_change`:

   - Updates rewards when a user's staked amount changes.
   - Can only be called by the authorized stake contract.

2. `claim_rewards`:
   - Allows users to claim their accumulated rewards.
   - Updates reward calculations and transfers tokens to the user.
   - Can only be called by the authorized stake contract.

### Query Messages

1. `config`: Retrieves current contract configuration.
2. `rewards`: Gets the current cumulative rewards data.
3. `user_reward`: Queries the reward data for a specific user.

## Configuration

The contract stores a `Config` struct containing:

- `stake`: Address of the authorized stake contract.
- `denom`: Denomination of the reward tokens.
- `duration_sec`: Duration of the reward period in seconds.
- `period_finish`: Timestamp when the current reward period ends.
- `rewards_per_second`: Rate of reward distribution per second.

## Reward Calculation

- The contract calculates rewards based on the time elapsed and the user's staked amount.
- Rewards are accumulated over time and can be claimed by users.

## Security

- Only the authorized stake contract can call `stake_change` and `claim_rewards` functions.
- Implements checks to ensure valid inputs and prevent unauthorized access.

## Events

The contract emits events for important actions:

- `ConfigEvent`: Triggered on configuration changes.
- `UpdateRewardsEvent`: Emitted when global rewards are updated.
- `UpdateUserRewardsEvent`: Fired when a user's rewards are updated.

## Dependencies

- Uses `cosmwasm_std` for CosmWasm standard library functions.
- Implements `sylvia` for contract structure and entry points.
- Utilizes `cw_storage_plus` for storage management.
- Incorporates `cw_utils` for common utilities.

## Error Handling

- Uses a custom `ContractError` type to handle various error scenarios.
- Incorporates `CommonError` for standard error cases.

## Version

- Contract Name: Defined in `CARGO_PKG_NAME`
- Contract Version: Defined in `CARGO_PKG_VERSION`

This README provides an overview of the StakeRewards smart contract, highlighting its main features and functionality. For detailed implementation and usage, refer to the contract source code.
