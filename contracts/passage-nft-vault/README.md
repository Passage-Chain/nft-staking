# PassageNftVault Smart Contract

## Overview

PassageNftVault is a CosmWasm smart contract designed for staking NFTs (Non-Fungible Tokens) and managing associated rewards. It allows users to stake and unstake NFTs, claim rewards, and interact with multiple reward accounts.

## Key Features

1. **NFT Staking**: Users can stake and unstake NFTs from specified collections.
2. **Reward Management**: Supports multiple reward accounts and allows users to claim rewards.
3. **Configurable Parameters**: Admins can update contract configuration and create new reward accounts.
4. **Stake Tracking**: Keeps track of staked NFTs, user stake amounts, and total staked amount.

## Contract Structure

The main struct `PassageNftVault` contains:

- `config`: Stores global configuration settings.
- `reward_accounts`: Manages multiple reward account addresses.
- `users_staked_nfts`: Indexed map of staked NFTs.
- `users_collection_staked_amounts`: Tracks staked amounts per user and collection.
- `total_staked_amount`: Snapshot of the total staked amount.
- `claims`: Manages claimable NFTs.

## Key Functions

### Instantiate

- Initializes the contract with configuration settings.

### Execute Messages

1. `update_config`:

   - Allows admin to update rewards code ID and unstaking duration.

2. `create_reward_account`:

   - Creates a new reward account with specified parameters.

3. `stake`:

   - Allows users to stake NFTs from approved collections.

4. `unstake`:

   - Enables users to unstake their NFTs.

5. `claim`:

   - Allows users to claim unstaked NFTs after the unstaking period.

6. `claim_rewards`:
   - Enables users to claim rewards from all reward accounts.

### Query Messages

1. `config`: Retrieves current contract configuration.
2. `reward_accounts`: Lists all reward account addresses.
3. `users_staked_nfts`: Queries staked NFTs for a specific user.
4. `users_collection_staked_amounts`: Retrieves staked amounts per collection for a user.
5. `total_staked_amount_at_height`: Gets the total staked amount at a specific block height.
6. `claims`: Lists claimable NFTs for a user.

## Configuration

The contract stores a `Config` struct containing:

- `rewards_code_id`: Code ID for reward contracts.
- `collections`: List of approved NFT collections.
- `unstaking_duration_sec`: Duration of the unstaking period.

## Security

- Admin-only functions are protected to ensure only authorized users can perform sensitive operations.
- Implements checks to verify NFT ownership and staking status.

## Events

The contract emits events for important actions:

- `ConfigEvent`: Triggered on configuration changes.
- `RewardAccountEvent`: Emitted when a new reward account is created.
- `claim-unstaked`: Fired when unstaked NFTs are claimed.
- `claim-rewards`: Triggered when rewards are claimed.

## Dependencies

- Uses `cosmwasm_std` for CosmWasm standard library functions.
- Implements `sylvia` for contract structure and entry points.
- Utilizes `cw_storage_plus` for advanced storage management.
- Incorporates `uju_cw2_common` and `uju_cw2_nft` for common utilities and NFT operations.

## Constants

- `MAX_CLAIMS`: Maximum number of claims (100).
- `MAX_NFTS`: Maximum number of NFTs per stake/unstake operation (20).

## Version

- Contract Name: Defined in `CARGO_PKG_NAME`
- Contract Version: Defined in `CARGO_PKG_VERSION`

This README provides an overview of the PassageNftVault smart contract, highlighting its main features and functionality. For detailed implementation and usage, refer to the contract source code.
