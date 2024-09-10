# Passage NFT Staking

This system comprises three interconnected smart contracts that work together to create a comprehensive NFT staking and reward distribution platform. Here's an overview of each contract and how they interact:

## 1. PassageVaultFactory Contract

**Purpose**: Manages the creation and tracking of NFT stake vaults.

**Key Features**:

- Creates new NFT stake vaults (PassageNftVault instances)
- Tracks all created vaults
- Manages global configuration for vault creation

**Role in the System**:

- Entry point for administrators to set up new staking opportunities
- Ensures consistent configuration across all created vaults

## 2. PassageNftVault

**Purpose**: Handles the core NFT staking functionality.

**Key Features**:

- Allows users to stake and unstake NFTs
- Tracks staked NFTs and amounts per user and collection
- Manages the unstaking process and claiming of unstaked NFTs
- Interacts with reward contracts for reward distribution

**Role in the System**:

- Central contract for user interactions (staking, unstaking, claiming)
- Maintains the state of staked NFTs and user balances
- Triggers reward calculations and claims in the PassageStakeRewards

## 3. PassageStakeRewards

**Purpose**: Manages the calculation and distribution of rewards for stakers.

**Key Features**:

- Calculates rewards based on staking duration and amounts
- Allows users to claim accumulated rewards
- Supports time-based reward periods

**Role in the System**:

- Handles all reward-related calculations and token distributions
- Integrates with the PassageNftVault to update rewards when staking changes occur

## How They Combine

1. **Initialization**:

   - The PassageVaultFactory is deployed first, setting up the global configuration.
   - Administrators use the PassageVaultFactory to create new PassageNftVault instances for different NFT collections or staking opportunities.
   - For each PassageNftVault, one or more PassageStakeRewards instances are created to manage different reward tokens or periods.

2. **User Interactions**:

   - Users interact primarily with the PassageNftVault to stake and unstake their NFTs.
   - When users stake or unstake, the PassageNftVault updates its internal state and notifies the associated PassageStakeRewards(s).

3. **Reward Management**:

   - The PassageStakeRewards continuously calculates rewards based on the staking data provided by the PassageNftVault.
   - When users want to claim rewards, they initiate the process through the PassageNftVault, which then calls the PassageStakeRewards to calculate and distribute the rewards.

4. **Administration**:

   - Admins can create new vaults through the PassageVaultFactory as needed.
   - Each PassageNftVault can be configured with its own set of allowed NFT collections and reward parameters.

5. **Scalability**:
   - This modular design allows for easy expansion of the system.
   - New NFT collections can be added by creating new vaults.
   - Different reward structures can be implemented by deploying new PassageStakeRewards instances.

This system provides a flexible and scalable solution for NFT staking with customizable reward structures. It separates concerns between vault management, staking operations, and reward calculations, allowing for easier maintenance and upgrades of individual components.
