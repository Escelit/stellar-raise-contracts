# Stellar Token Minter Contract

The `StellarTokenMinter` contract is a basic NFT minter implementation for the Stellar Raise platform. It allows authorized contracts to mint reward NFTs for campaign contributors.

## Features

- **Initialization**: Sets up the owner (admin) and the authorized minter.
- **Minting**: Creates a new NFT with a specific token ID and assigns it to a recipient.
- **Authorization**: Restricts minting to the designated `minter` address.
- **Transparency**: Provides public functions to check token ownership and total supply.

## Storage Keys

The contract uses the following storage keys:

- `Admin`: The administrator of the contract.
- `Minter`: The address allowed to call the `mint` function.
- `TotalMinted`: The total number of NFTs minted so far.
- `TokenMetadata(token_id)`: Maps a token ID to the owner's address.

## Security Assumptions

- **Immutable Ownership**: Once minted, a token's owner is recorded in persistent storage.
- **Restricted Access**: Only the `minter` (or the `admin` via `set_minter`) can create new tokens.
- **No Overwriting**: Attempting to mint an existing `token_id` will result in a panic.

## Example Usage (Soroban SDK)

```rust
// Minter contract setup
let minter_client = StellarTokenMinterClient::new(&env, &contract_id);
minter_client.initialize(&admin, &minter);

// Minting a reward
minter_client.mint(&contributor_address, &token_id);
```

## Maintenance

The administrator can change the `minter` address at any time to point to a new campaign contract.
