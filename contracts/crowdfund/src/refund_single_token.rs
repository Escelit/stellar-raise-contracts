/// # refund_single_token
///
/// @title   RefundSingle — Single-contributor token refund logic
/// @notice  Encapsulates the token transfer step that returns a contributor's
///          funds during a failed or cancelled crowdfund campaign.
/// @dev     This module documents and validates the `refund_single` pattern
///          used inside the bulk `refund()` and `cancel()` flows of the
///          CrowdfundContract.  It is intentionally kept as a pure, testable
///          unit so that the transfer logic can be reasoned about in isolation.
///
/// ## Security Assumptions
/// 1. The caller (the contract itself) already holds the tokens to be
///    returned — no external pull is performed here.
/// 2. The contribution amount stored in persistent storage is the single
///    source of truth; it is zeroed **after** a successful transfer to
///    prevent double-refund.
/// 3. Zero-amount contributions are skipped to avoid wasting gas on no-op
///    transfers.
/// 4. Overflow is impossible because `amount` is an `i128` read directly
///    from storage and was validated at contribution time.
/// 5. The token client is constructed from the address stored at
///    initialisation — it cannot be substituted by a caller.
///
/// ## Token Transfer Flow (refund_single)
///
/// ```text
/// persistent storage
///   └─ Contribution(contributor) ──► amount: i128
///                                         │
///                                    amount > 0?
///                                    ┌────┴────┐
///                                   YES        NO
///                                    │          └─► skip (no-op)
///                                    ▼
///                          token_client.transfer(
///                            from  = contract_address,
///                            to    = contributor,
///                            value = amount
///                          )
///                                    │
///                                    ▼
///                          set Contribution(contributor) = 0
///                          extend_ttl(contribution_key, 100, 100)
///                                    │
///                                    ▼
///                          emit event ("campaign", "refund_single")
///                                 (contributor, amount)
/// ```

#![no_std]

use soroban_sdk::{token, Address, Env};

use crate::DataKey;

/// Refunds a single contributor by transferring their stored contribution
/// amount back from the contract to their address.
///
/// @notice This is the atomic unit of the bulk `refund()` loop.  It is safe
///         to call for contributors whose balance is already zero — the
///         function is a no-op in that case.
///
/// @param  env              The Soroban execution environment.
/// @param  token_address    The address of the token contract.
/// @param  contributor      The address of the contributor to refund.
///
/// @return                  The amount refunded (0 if nothing was owed).
///
/// @dev    Storage mutation order:
///           1. Read amount  (fail-safe: defaults to 0 if key absent)
///           2. Transfer tokens  (panics on token contract error)
///           3. Zero the storage entry  (prevents double-refund)
///           4. Extend TTL so the zeroed entry remains queryable
///           5. Emit event for off-chain indexers
pub fn refund_single(env: &Env, token_address: &Address, contributor: &Address) -> i128 {
    // ── Step 1: Read the stored contribution ────────────────────────────────
    // `unwrap_or(0)` ensures we never panic on a missing key; a missing key
    // is semantically equivalent to a zero contribution.
    let contribution_key = DataKey::Contribution(contributor.clone());
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&contribution_key)
        .unwrap_or(0);

    // ── Step 2: Skip zero-amount contributors ───────────────────────────────
    // Avoids a wasted cross-contract call and keeps the event log clean.
    if amount == 0 {
        return 0;
    }

    // ── Step 3: Transfer tokens from contract → contributor ─────────────────
    // The contract must hold at least `amount` tokens at this point.
    // If the token transfer fails (e.g. insufficient balance), the entire
    // transaction is rolled back — no storage mutation occurs.
    let token_client = token::Client::new(env, token_address);
    token_client.transfer(&env.current_contract_address(), contributor, &amount);

    // ── Step 4: Zero the contribution record ────────────────────────────────
    // Must happen AFTER the transfer to prevent a re-entrancy window where
    // the contributor could trigger another refund before the record is cleared.
    env.storage().persistent().set(&contribution_key, &0i128);

    // ── Step 5: Extend TTL so the zeroed record remains readable ────────────
    // Keeps the entry alive for 100 ledgers so off-chain tools can confirm
    // the refund without hitting a "key not found" error.
    env.storage()
        .persistent()
        .extend_ttl(&contribution_key, 100, 100);

    // ── Step 6: Emit refund event ────────────────────────────────────────────
    // Allows off-chain indexers and UIs to track individual refunds without
    // scanning storage.
    env.events()
        .publish(("campaign", "refund_single"), (contributor.clone(), amount));

    amount
}

/// Returns the stored contribution amount for a contributor without mutating
/// state.  Used by tests and read-only queries.
///
/// @param  env          The Soroban execution environment.
/// @param  contributor  The contributor address to query.
/// @return              The stored contribution amount (0 if absent).
pub fn get_contribution(env: &Env, contributor: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Contribution(contributor.clone()))
        .unwrap_or(0)
}
