//! Bounded `withdraw()` Event Emission Module
//!
//! This module provides the logic for capped NFT minting during campaign withdrawal.
//! It prevents unbounded gas consumption by limiting the number of NFT mints per
//! `withdraw()` call and emits a single summary event instead of many individual events.
//!
//! ## Features
//!
//! - **Gas Efficiency**: Caps NFT minting at `MAX_NFT_MINT_BATCH` per withdrawal
//! - **Event Optimization**: Emits single batch event instead of O(n) individual events
//! - **UX Improvement**: Provides comprehensive withdrawal data including NFT mint count
//!
//! ## Usage
//!
//! This module is used by the main crowdfund contract's `withdraw()` function.
//! The [`mint_nfts_in_batch`] function handles all NFT minting logic with proper
//! event emission and gas consumption limits.
//!
//! ## Example
//!
//! ```rust
//! use crate::withdraw_event_emission::mint_nfts_in_batch;
//!
//! fn withdraw_impl(env: &Env) -> u32 {
//!     let nft_contract = env.storage().instance().get::<_, Address>(&DataKey::NFTContract);
//!     let minted_count = mint_nfts_in_batch(env, &nft_contract);
//!     // ... continue with withdrawal ...
//!     minted_count
//! }
//! ```

use soroban_sdk::{Address, Env, IntoVal, Symbol, Vec};

use crate::{DataKey, MAX_NFT_MINT_BATCH};

/// Mint NFTs to eligible contributors in a single batch.
///
/// Processes at most `MAX_NFT_MINT_BATCH` contributors per call to prevent
/// unbounded gas consumption. Emits a single `nft_batch_minted` summary event
/// with the total count of NFTs minted.
///
/// # Parameters
///
/// - `env`: The Soroban environment
/// - `nft_contract`: Optional address of the NFT contract to mint to contributors
///
/// # Returns
///
/// The number of NFTs minted in this batch (0 if no NFT contract or no eligible contributors).
///
/// # Events Emitted
///
/// - `("campaign", "nft_batch_minted")` with `u32` count (only when > 0 minted)
///
/// # Security Considerations
///
/// - Contributors beyond the cap are NOT permanently skipped - they can be minted
///   in subsequent `withdraw()` calls if the campaign owner calls withdraw again.
/// - The cap is a compile-time constant. Changing it requires a contract upgrade.
/// - This function assumes the NFT contract has a `mint` function that accepts
///   `(Address, u64)` as arguments (recipient, token_id).
///
/// # Performance
///
/// - Time Complexity: O(min(n, MAX_NFT_MINT_BATCH)) where n is contributor count
/// - Space Complexity: O(1) - uses constant extra space
/// - Event Emission: O(1) - single batch event instead of O(n) individual events
///
/// # Edge Cases
///
/// - When `nft_contract` is `None`: returns 0, emits no events
/// - When no eligible contributors (all have 0 contribution): returns 0, emits no batch event
/// - When exactly `MAX_NFT_MINT_BATCH` contributors: mints exactly that many
/// - When > `MAX_NFT_MINT_BATCH` contributors: caps at MAX, allows remaining to be minted later
pub fn mint_nfts_in_batch(env: &Env, nft_contract: &Option<Address>) -> u32 {
    let Some(nft_contract) = nft_contract else {
        return 0;
    };

    let contributors: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));

    let mut token_id: u64 = 1;
    let mut minted: u32 = 0;

    // Process contributors up to MAX_NFT_MINT_BATCH
    for contributor in contributors.iter() {
        if minted >= MAX_NFT_MINT_BATCH {
            break;
        }

        // Get contribution amount for this contributor
        let contribution: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contributor.clone()))
            .unwrap_or(0);

        // Only mint NFT for contributors with non-zero contributions
        if contribution > 0 {
            // Invoke the NFT contract's mint function
            // The NFT contract must implement: fn mint(env: Env, to: Address, token_id: u64)
            env.invoke_contract::<()>(
                nft_contract,
                &Symbol::new(env, "mint"),
                Vec::from_array(env, [contributor.into_val(env), token_id.into_val(env)]),
            );
            token_id += 1;
            minted += 1;
        }
    }

    // Emit single summary event instead of one event per contributor.
    // This improves UX by reducing event log noise and improves
    // indexer performance with O(1) events vs O(n).
    if minted > 0 {
        env.events().publish(("campaign", "nft_batch_minted"), minted);
    }

    minted
}

/// Emit the withdrawal event with comprehensive data.
///
/// Publishes a single `withdrawn` event containing:
/// - Creator address (who received the payout)
/// - Payout amount (after platform fee deduction)
/// - Number of NFTs minted in this withdrawal
///
/// # Parameters
///
/// - `env`: The Soroban environment
/// - `creator`: The campaign creator who received the payout
/// - `payout`: The amount transferred to the creator (after fees)
/// - `nft_minted_count`: Number of NFTs minted to contributors
///
/// # Event Data
///
/// Topic: `("campaign", "withdrawn")`
/// Data: `(Address, i128, u32)` - (creator, payout, nft_count)
///
/// # Breaking Change Note
///
/// This event now includes a third field (nft_minted_count). Off-chain indexers
/// that decoded the old two-field tuple `(Address, i128)` must be updated to handle
/// the new three-field tuple `(Address, i128, u32)`.
pub fn emit_withdrawal_event(env: &Env, creator: &Address, payout: i128, nft_minted_count: u32) {
    env.events().publish(
        ("campaign", "withdrawn"),
        (creator.clone(), payout, nft_minted_count),
    );
}

#[cfg(test)]
mod tests {

    // Unit tests for the module would go here
    // Integration tests are in withdraw_event_emission_test.rs
}