# withdraw_event_emission

Bounded `withdraw()` event emission for the Stellar Raise crowdfund contract.

## Overview

This module improves the security and performance of `withdraw()` by:

1. **Bounding NFT minting** — caps mints at `MAX_NFT_MINT_BATCH` (50) per call, preventing unbounded gas consumption with large contributor lists.
2. **Single summary event** — emits one `nft_batch_minted` event instead of one per contributor (O(1) vs O(n)).
3. **Security-guarded emit helpers** — each helper asserts its inputs are valid before publishing, making invalid states impossible to emit silently.

## Public API

### `mint_nfts_in_batch(env, nft_contract) -> u32`

Mints NFTs to eligible contributors up to `MAX_NFT_MINT_BATCH`. Returns the count minted. Emits `("campaign", "nft_batch_minted")` only when count > 0.

### `emit_fee_transferred(env, platform, fee)`

Publishes `("campaign", "fee_transferred")` with `(Address, i128)`.  
**Panics** if `fee <= 0`.

### `emit_nft_batch_minted(env, minted_count)`

Publishes `("campaign", "nft_batch_minted")` with `u32` count.  
**Panics** if `minted_count == 0`.

### `emit_withdrawn(env, creator, creator_payout, nft_minted_count)`

Publishes `("campaign", "withdrawn")` with `(Address, i128, u32)`.  
**Panics** if `creator_payout <= 0`.

## Events Reference

| Topic 1    | Topic 2             | Data                        | When emitted                        |
|------------|---------------------|-----------------------------|-------------------------------------|
| `campaign` | `withdrawn`         | `(Address, i128, u32)`      | Every successful `withdraw()` call  |
| `campaign` | `fee_transferred`   | `(Address, i128)`           | When platform fee is configured     |
| `campaign` | `nft_batch_minted`  | `u32`                       | When ≥1 NFT is minted               |

> **Breaking change**: The `withdrawn` event now carries a third field (`nft_minted_count: u32`). Off-chain indexers decoding the old `(Address, i128)` tuple must be updated.

## Security Considerations

- **Reentrancy**: `TotalRaised` is zeroed before NFT minting and event emission, following checks-effects-interactions.
- **Overflow**: Fee calculation uses `checked_mul` / `checked_div`; payout uses `checked_sub`.
- **Authorization**: `creator.require_auth()` is called before any transfer.
- **Batch cap**: Contributors beyond `MAX_NFT_MINT_BATCH` are not permanently skipped — a subsequent `withdraw()` call (if the contract is upgraded to allow it) can mint the remainder.

## Usage

```rust
use crate::withdraw_event_emission::{emit_fee_transferred, emit_withdrawn, mint_nfts_in_batch};

// Inside withdraw():
let nft_contract: Option<Address> = env.storage().instance().get(&DataKey::NFTContract);
let nft_minted_count = mint_nfts_in_batch(&env, &nft_contract);
emit_withdrawn(&env, &creator, creator_payout, nft_minted_count);
```
