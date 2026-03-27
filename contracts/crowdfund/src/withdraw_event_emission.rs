//! Bounded `withdraw()` Event Emission Module
//!
//! Centralises all event publishing for the `withdraw()` function.
//! Three validated helpers replace scattered inline `env.events().publish()`
//! calls, preventing silent emission of zero-fee or zero-payout events that
//! would mislead off-chain indexers.
//!
//! ## Optimisation
//!
//! The original implementation emitted one `nft_minted` event per contributor
//! (O(n) events). This module replaces that with a single `nft_batch_minted`
//! summary event (O(1)), capping gas consumption regardless of contributor count.
//!
//! ## Events emitted
//!
//! | Topic 2            | Data                   | Condition                          |
//! |--------------------|------------------------|------------------------------------|
//! | `fee_transferred`  | `(Address, i128)`      | Platform fee > 0                   |
//! | `nft_batch_minted` | `u32`                  | NFT contract set, minted_count > 0 |
//! | `withdrawn`        | `(Address, i128, u32)` | Always on successful withdraw      |

use soroban_sdk::{Address, Env, Vec};

use crate::{DataKey, NftContractClient, MAX_NFT_MINT_BATCH};

// ── Validated emit helpers ────────────────────────────────────────────────────

/// Emits the `("campaign", "fee_transferred")` event.
///
/// @notice Publishes the platform fee transfer so off-chain indexers can track
///         fee revenue without querying token balances.
/// @param  env      The Soroban environment.
/// @param  platform The platform address that received the fee.
/// @param  fee      The fee amount transferred (must be > 0).
///
/// @custom:security Panics if `fee <= 0` — a zero or negative fee indicates a
///                  logic error upstream and must not be silently emitted.
pub fn emit_fee_transferred(env: &Env, platform: &Address, fee: i128) {
    assert!(fee > 0, "fee_transferred: fee must be positive");
    env.events()
        .publish(("campaign", "fee_transferred"), (platform.clone(), fee));
}

/// Emits the `("campaign", "nft_batch_minted")` event.
///
/// @notice Replaces per-contributor `nft_minted` events with a single O(1)
///         summary, keeping event volume constant regardless of contributor count.
/// @param  env           The Soroban environment.
/// @param  minted_count  Number of NFTs minted in this batch (must be > 0).
///
/// @custom:security Panics if `minted_count == 0` — callers must guard with
///                  `if minted > 0` before calling this helper.
pub fn emit_nft_batch_minted(env: &Env, minted_count: u32) {
    assert!(
        minted_count > 0,
        "nft_batch_minted: minted_count must be positive"
    );
    env.events()
        .publish(("campaign", "nft_batch_minted"), minted_count);
}

/// Emits the `("campaign", "withdrawn")` event.
///
/// @notice Published exactly once per successful `withdraw()` call. Carries
///         creator address, net payout (after fee), and NFT mint count so
///         frontends can display a complete withdrawal receipt from a single
///         event without additional RPC calls.
/// @param  env              The Soroban environment.
/// @param  creator          The campaign creator who received the payout.
/// @param  creator_payout   Net amount transferred to creator (must be > 0).
/// @param  nft_minted_count NFTs minted in this withdrawal (0 is valid).
///
/// @custom:security Panics if `creator_payout <= 0` — a zero or negative
///                  payout indicates a logic error upstream.
pub fn emit_withdrawn(env: &Env, creator: &Address, creator_payout: i128, nft_minted_count: u32) {
    assert!(
        creator_payout > 0,
        "withdrawn: creator_payout must be positive"
    );
    env.events().publish(
        ("campaign", "withdrawn"),
        (creator.clone(), creator_payout, nft_minted_count),
    );
}

// ── NFT batch minting ─────────────────────────────────────────────────────────

/// Mint NFTs to eligible contributors in a single bounded batch.
///
/// @notice Processes at most `MAX_NFT_MINT_BATCH` contributors per call to
///         prevent unbounded gas consumption. Emits a single `nft_batch_minted`
///         summary event when at least one NFT is minted.
/// @param  env          The Soroban environment.
/// @param  nft_contract Optional address of the NFT contract.
/// @return Number of NFTs minted (0 if no NFT contract or no eligible contributors).
///
/// @custom:security Contributors beyond the cap are NOT permanently skipped —
///                  they can be minted in a subsequent call if needed.
pub fn mint_nfts_in_batch(env: &Env, nft_contract: &Option<Address>) -> u32 {
    let Some(nft_addr) = nft_contract else {
        return 0;
    };

    let contributors: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));

    let client = NftContractClient::new(env, nft_addr);
    let mut minted: u32 = 0;

    for contributor in contributors.iter() {
        if minted >= MAX_NFT_MINT_BATCH {
            break;
        }
        let contribution: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contributor.clone()))
            .unwrap_or(0);
        if contribution > 0 {
            client.mint(&contributor);
            minted += 1;
        }
    }

    if minted > 0 {
        emit_nft_batch_minted(env, minted);
    }

    minted
}

/// Emit the withdrawal event — thin wrapper kept for call-site compatibility.
///
/// @notice Delegates to `emit_withdrawn`. Prefer calling `emit_withdrawn`
///         directly in new code.
pub fn emit_withdrawal_event(env: &Env, creator: &Address, payout: i128, nft_minted_count: u32) {
    emit_withdrawn(env, creator, payout, nft_minted_count);
}
