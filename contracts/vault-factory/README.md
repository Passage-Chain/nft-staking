# VaultFactory Smart Contract

## Overview

VaultFactory is a CosmWasm smart contract that facilitates the creation and management of NFT stake vaults. It allows administrators to create new vaults with specific configurations and keeps track of all created vaults.

## Key Features

1. **Vault Creation**: Admins can create new NFT stake vaults with customizable parameters.
2. **Configuration Management**: The contract stores and allows updates to global configuration settings.
3. **Vault Tracking**: Maintains a record of all created vaults.

## Contract Structure

The main struct `VaultFactory` contains:

- `config`: Stores global configuration settings.
- `vaults`: A map to keep track of created vaults.

## Key Functions

### Instantiate

- Initializes the contract with vault and rewards code IDs.

### Execute Messages

1. `update_config`:

   - Allows admin to update vault and rewards code IDs.

2. `create_vault`:
   - Creates a new NFT stake vault with specified parameters.
   - Only callable by the contract admin.

### Query Messages

1. `vaults`:
   - Retrieves a list of created vaults with pagination support.

## Configuration

The contract stores a `Config` struct containing:

- `vault_code_id`: Code ID for the NFT vault contract.
- `rewards_code_id`: Code ID for the rewards contract.

## Security

- Admin-only functions are protected to ensure only authorized users can perform sensitive operations.

## Events

The contract emits events for important actions:

- `ConfigEvent`: Triggered on configuration changes.
- `VaultEvent`: Emitted when a new vault is created.

## Dependencies

- Uses `cosmwasm_std` for CosmWasm standard library functions.
- Implements `sylvia` for contract structure and entry points.
- Utilizes `cw_storage_plus` for storage management.
- Incorporates `uju_cw2_common` for common contract utilities.

## Version

- Contract Name: Defined in `CARGO_PKG_NAME`
- Contract Version: Defined in `CARGO_PKG_VERSION`

This README provides an overview of the VaultFactory smart contract, highlighting its main features and functionality. For detailed implementation and usage, refer to the contract source code.
