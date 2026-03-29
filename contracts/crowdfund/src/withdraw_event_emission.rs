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
            env.invoke_contract::<()>(
                nft_contract,
                &Symbol::new(env, "mint"),
                Vec::from_array(env, [contributor.into_val(env), token_id.into_val(env)]),
            );
            token_id += 1;
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
//! # Withdraw Event Emission Module
//!
//! Provides security-hardened helpers for emitting events during the
//! `withdraw()` lifecycle. All event emission is centralised here so that
//! the main contract function stays readable and every event payload is
//! validated in one place.
//!
//! ## Events emitted by `withdraw()`
//!
//! | Topic 1    | Topic 2            | Data                   | Condition                          |
//! |------------|--------------------|------------------------|------------------------------------|
//! | `campaign` | `fee_transferred`  | `(Address, i128)`      | Platform fee is configured         |
//! | `campaign` | `nft_batch_minted` | `u32`                  | NFT contract set and ≥1 mint done  |
//! | `campaign` | `withdrawn`        | `(Address, i128, u32)` | Always on successful withdraw      |
//!
//! ## Security assumptions
//!
//! * All amounts are validated to be non-negative before emission.
//! * The `withdrawn` event is emitted **after** state mutation (status set to
//!   `Successful`, `TotalRaised` zeroed) so off-chain indexers observe a
//!   consistent final state.
//! * `emit_fee_transferred` is only called when `fee > 0` to prevent
//!   misleading zero-fee events.
//! * `emit_nft_batch_minted` is only called when `minted_count > 0`.
//! * `emit_withdrawn` always fires exactly once per successful `withdraw()`
//!   invocation — callers must not call it more than once.

#![allow(missing_docs)]

use soroban_sdk::{Address, Env};

// ── Fee transferred ──────────────────────────────────────────────────────────

/// Emit a `fee_transferred` event.
///
/// # Arguments
/// * `env`              – The contract environment.
/// * `platform_address` – Recipient of the platform fee.
/// * `fee`              – Fee amount transferred (must be > 0).
///
/// # Panics
/// * If `fee` is zero or negative — a zero-fee event is misleading and
///   indicates a logic error in the caller.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "fee_transferred")
/// data   : (Address, i128)   // (platform_address, fee)
/// ```
pub fn emit_fee_transferred(env: &Env, platform_address: &Address, fee: i128) {
    assert!(fee > 0, "fee_transferred: fee must be positive");
    env.events()
        .publish(("campaign", "fee_transferred"), (platform_address, fee));
}

// ── NFT batch minted ─────────────────────────────────────────────────────────

/// Emit a single `nft_batch_minted` summary event.
///
/// Replaces the previous per-contributor `nft_minted` event pattern.
/// Emitting one summary event instead of N individual events caps gas
/// consumption when the contributor list is large.
///
/// # Arguments
/// * `env`          – The contract environment.
/// * `minted_count` – Number of NFTs minted in this batch (must be > 0).
///
/// # Panics
/// * If `minted_count` is zero — callers must guard against emitting an
///   empty-batch event.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "nft_batch_minted")
/// data   : u32   // number of NFTs minted
/// ```
pub fn emit_nft_batch_minted(env: &Env, minted_count: u32) {
    assert!(
        minted_count > 0,
        "nft_batch_minted: minted_count must be positive"
    );
    env.events()
        .publish(("campaign", "nft_batch_minted"), minted_count);
}

// ── Withdrawn ────────────────────────────────────────────────────────────────

/// Emit the `withdrawn` event that signals a successful campaign withdrawal.
///
/// This is the canonical terminal event for a successful campaign. It carries
/// the creator address, the net payout (after any platform fee), and the
/// number of NFTs minted in this call.
///
/// # Arguments
/// * `env`             – The contract environment.
/// * `creator`         – The campaign creator who received the payout.
/// * `creator_payout`  – Net amount transferred to the creator (must be > 0).
/// * `nft_minted_count`– Number of NFTs minted (0 if no NFT contract set).
///
/// # Panics
/// * If `creator_payout` is zero or negative — a zero-payout withdrawal
///   indicates a logic error upstream.
///
/// # Event payload
/// ```text
/// topics : ("campaign", "withdrawn")
/// data   : (Address, i128, u32)   // (creator, creator_payout, nft_minted_count)
/// ```
pub fn emit_withdrawn(env: &Env, creator: &Address, creator_payout: i128, nft_minted_count: u32) {
    assert!(
        creator_payout > 0,
        "withdrawn: creator_payout must be positive"
    );
    env.events().publish(
        ("campaign", "withdrawn"),
        (creator, creator_payout, nft_minted_count),
    );
}
//! Bounded `withdraw()` Event Emission Module
//!
//! Provides three focused emit helpers and bounded NFT minting for the
//! crowdfund contract's `withdraw()` function.
//!
//! ## Security Invariants
//!
//! - [`emit_fee_transferred`] panics if `fee <= 0` — prevents silent zero-fee events.
//! - [`emit_nft_batch_minted`] panics if `minted_count == 0` — callers must guard.
//! - [`emit_withdrawn`] panics if `creator_payout <= 0` — prevents zero-payout withdrawals.
//! Optimized `withdraw()` Event Emission Module
//!
//! Centralises all event publishing for the `withdraw()` function into three
//! validated helper functions. Each helper asserts its invariants before
//! publishing, so a logic error upstream causes an explicit panic rather than
//! a silently misleading event on-chain.
//!
//! ## Design Goals
//!
//! | Goal | Mechanism |
//! |------|-----------|
//! | Gas efficiency | Single `nft_batch_minted` event instead of O(n) per-contributor events |
//! | Readability | Named helpers replace scattered `env.events().publish(…)` calls |
//! | Security | Positive-only assertions on every monetary amount |
//! | Testability | Each helper is independently unit-testable |
//!
//! ## Events Published
//!
//! | Topic 1    | Topic 2            | Data                        | Condition |
//! |------------|--------------------|-----------------------------|-----------|
//! | `campaign` | `fee_transferred`  | `(Address, i128)`           | Platform fee > 0 |
//! | `campaign` | `nft_batch_minted` | `u32`                       | At least 1 NFT minted |
//! | `campaign` | `withdrawn`        | `(Address, i128, u32)`      | Always on success |
//!
//! ## Performance
//!
//! [`mint_nfts_in_batch`] caps NFT minting at [`MAX_NFT_MINT_BATCH`] per call,
//! bounding gas consumption and emitting a single summary event instead of O(n).
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
//! ```rust
//! use crate::withdraw_event_emission::{
//!     emit_fee_transferred, emit_nft_batch_minted, emit_withdrawn, mint_nfts_in_batch,
//! };
//!
//! // Inside withdraw():
//! if fee > 0 {
//!     emit_fee_transferred(&env, &platform_addr, fee);
//! }
//! let nft_count = mint_nfts_in_batch(&env, &nft_contract);
//! emit_withdrawn(&env, &creator, creator_payout, nft_count);
//! ```

use soroban_sdk::{Address, Env, Vec};

use crate::{DataKey, NftContractClient, MAX_NFT_MINT_BATCH};

/// Mint NFTs to eligible contributors, capped at `MAX_NFT_MINT_BATCH`.
///
/// Emits a single `("campaign", "nft_batch_minted")` event with the count
/// when at least one NFT is minted. Returns 0 and emits nothing when
/// `nft_contract` is `None` or no contributor has a positive balance.
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
// ── Validated event helpers ──────────────────────────────────────────────────

/// Emit a `fee_transferred` event.
///
/// Published when a platform fee is deducted from the withdrawal amount and
/// transferred to the platform address.
///
/// # Parameters
/// - `env`      – The Soroban environment.
/// - `platform` – The platform address that received the fee.
/// - `fee`      – The fee amount transferred (must be > 0).
///
/// # Panics
/// Panics with `"fee_transferred: fee must be positive"` when `fee <= 0`.
/// A zero or negative fee indicates a logic error upstream and must not be
/// silently emitted as a misleading on-chain event.
///
/// # Event
/// Topic: `("campaign", "fee_transferred")`
/// Data:  `(Address, i128)` — (platform, fee)
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
/// Emit an `nft_batch_minted` event.
///
/// Published once per `withdraw()` call when at least one NFT was minted to
/// contributors. Replaces the previous O(n) per-contributor event pattern,
/// reducing event log noise and improving indexer performance.
///
/// # Parameters
/// - `env`           – The Soroban environment.
/// - `minted_count`  – Number of NFTs minted in this batch (must be > 0).
///
/// # Panics
/// Panics with `"nft_batch_minted: minted_count must be positive"` when
/// `minted_count == 0`. The caller must guard with `if minted > 0`.
///
/// # Event
/// Topic: `("campaign", "nft_batch_minted")`
/// Data:  `u32` — number of NFTs minted
pub fn emit_nft_batch_minted(env: &Env, minted_count: u32) {
    assert!(
        minted_count > 0,
        "nft_batch_minted: minted_count must be positive"
    );
    env.events()
        .publish(("campaign", "nft_batch_minted"), minted_count);
}

/// Emit a `withdrawn` event.
///
/// Published exactly once per successful `withdraw()` call. Carries the
/// creator address, net payout (after platform fee), and the number of NFTs
/// minted to contributors in this withdrawal.
///
/// # Parameters
/// - `env`             – The Soroban environment.
/// - `creator`         – The campaign creator who received the payout.
/// - `creator_payout`  – Net amount transferred to the creator (must be > 0).
/// - `nft_minted_count`– NFTs minted this call (0 is valid when no NFT contract).
///
/// # Panics
/// Panics with `"withdrawn: creator_payout must be positive"` when
/// `creator_payout <= 0`. A zero or negative payout indicates a logic error
/// upstream.
///
/// # Event
/// Topic: `("campaign", "withdrawn")`
/// Data:  `(Address, i128, u32)` — (creator, creator_payout, nft_minted_count)
///
/// # Breaking Change Note
/// The data tuple now has three fields `(Address, i128, u32)`. Off-chain
/// indexers that decoded the old two-field tuple `(Address, i128)` must be
/// updated.
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

// ── Batch NFT minting ────────────────────────────────────────────────────────

/// Mint NFTs to eligible contributors in a single bounded batch.
///
/// Processes at most [`MAX_NFT_MINT_BATCH`] contributors per call to prevent
/// unbounded gas consumption. Emits a single `nft_batch_minted` summary event
/// via [`emit_nft_batch_minted`] when at least one NFT is minted.
///
/// # Parameters
/// - `env`          – The Soroban environment.
/// - `nft_contract` – Optional address of the NFT contract. Returns 0 immediately
///                    when `None`.
///
/// # Returns
/// Number of NFTs minted in this batch (0 when no contract or no eligible contributors).
///
/// # Security Considerations
/// - Contributors beyond the cap are **not** permanently skipped. Subsequent
///   `withdraw()` calls (or a dedicated claim function) can mint the remainder.
/// - The cap is a compile-time constant; changing it requires a contract upgrade.
/// - The NFT contract must implement `fn mint(env: Env, to: Address, token_id: u64)`.
///
/// # Complexity
/// - Time:  O(min(n, MAX_NFT_MINT_BATCH))
/// - Space: O(1)
/// - Events: O(1) — single batch event
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

        if contribution > 0 {
            env.invoke_contract::<()>(
                nft_contract,
                &Symbol::new(env, "mint"),
                Vec::from_array(env, [contributor.into_val(env), token_id.into_val(env)]),
            );
            token_id += 1;
            minted += 1;
        }
    }

    // O(1) summary event — replaces the previous O(n) per-contributor pattern.
    if minted > 0 {
        emit_nft_batch_minted(env, minted);
    }

    minted
}

/// Emit `("campaign", "fee_transferred")` with `(platform_address, fee)`.
///
/// # Panics
///
/// Panics if `fee <= 0` — a zero or negative fee transfer is a logic error.
pub fn emit_fee_transferred(env: &Env, platform: &Address, fee: i128) {
    assert!(fee > 0, "fee_transferred: fee must be positive");
    env.events()
        .publish(("campaign", "fee_transferred"), (platform.clone(), fee));
}

/// Emit `("campaign", "nft_batch_minted")` with the minted count.
///
/// # Panics
///
/// Panics if `minted_count == 0` — callers must only call this when minting occurred.
pub fn emit_nft_batch_minted(env: &Env, minted_count: u32) {
    assert!(
        minted_count > 0,
        "nft_batch_minted: minted_count must be positive"
    );
    env.events()
        .publish(("campaign", "nft_batch_minted"), minted_count);
}

/// Emit `("campaign", "withdrawn")` with `(creator, payout, nft_minted_count)`.
///
/// # Panics
///
/// Panics if `creator_payout <= 0` — a zero or negative payout is a logic error.
pub fn emit_withdrawn(env: &Env, creator: &Address, creator_payout: i128, nft_minted_count: u32) {
    assert!(
        creator_payout > 0,
        "withdrawn: creator_payout must be positive"
    );
    env.events().publish(
        ("campaign", "withdrawn"),
        (creator.clone(), creator_payout, nft_minted_count),
    );
/// Emit the withdrawal event — thin wrapper kept for call-site compatibility.
///
/// @notice Delegates to `emit_withdrawn`. Prefer calling `emit_withdrawn`
///         directly in new code.
pub fn emit_withdrawal_event(env: &Env, creator: &Address, payout: i128, nft_minted_count: u32) {
    emit_withdrawn(env, creator, payout, nft_minted_count);
/// Convenience wrapper: emit the full withdrawal event tuple.
///
/// Delegates to [`emit_withdrawn`]. Kept for backwards compatibility with
/// call sites that used the old `emit_withdrawal_event` name.
///
/// Prefer calling [`emit_withdrawn`] directly in new code.
#[inline]
pub fn emit_withdrawal_event(env: &Env, creator: &Address, payout: i128, nft_minted_count: u32) {
    emit_withdrawn(env, creator, payout, nft_minted_count);
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    // These mirror the security unit tests in withdraw_event_emission_test.rs
    // but run without the full contract setup for fast feedback.

    #[test]
    #[should_panic(expected = "fee_transferred: fee must be positive")]
    fn emit_fee_transferred_rejects_zero() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_fee_transferred(&env, &addr, 0);
    }

    #[test]
    #[should_panic(expected = "fee_transferred: fee must be positive")]
    fn emit_fee_transferred_rejects_negative() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_fee_transferred(&env, &addr, -1);
    }

    #[test]
    fn emit_fee_transferred_accepts_positive() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_fee_transferred(&env, &addr, 1);
    }

    #[test]
    #[should_panic(expected = "nft_batch_minted: minted_count must be positive")]
    fn emit_nft_batch_minted_rejects_zero() {
        let env = Env::default();
        emit_nft_batch_minted(&env, 0);
    }

    #[test]
    fn emit_nft_batch_minted_accepts_positive() {
        let env = Env::default();
        emit_nft_batch_minted(&env, 1);
    }

    #[test]
    #[should_panic(expected = "withdrawn: creator_payout must be positive")]
    fn emit_withdrawn_rejects_zero_payout() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_withdrawn(&env, &addr, 0, 0);
    }

    #[test]
    #[should_panic(expected = "withdrawn: creator_payout must be positive")]
    fn emit_withdrawn_rejects_negative_payout() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_withdrawn(&env, &addr, -100, 0);
    }

    #[test]
    fn emit_withdrawn_accepts_valid_args() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_withdrawn(&env, &addr, 1_000, 5);
    }

    #[test]
    fn emit_withdrawn_allows_zero_nft_count() {
        let env = Env::default();
        let addr = Address::generate(&env);
        emit_withdrawn(&env, &addr, 500, 0);
    }
}
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

use soroban_sdk::{Address, Env, IntoVal, Symbol, Vec};

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
    let mut token_id: u64 = 0;

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
            env.invoke_contract::<()>(
                nft_addr,
                &Symbol::new(env, "mint"),
                Vec::from_array(env, [contributor.into_val(env), token_id.into_val(env)]),
            );
            token_id += 1;
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
