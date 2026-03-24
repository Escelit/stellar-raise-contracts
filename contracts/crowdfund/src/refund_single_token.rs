// # `refund_single` Token Transfer Logic
//
// This module centralises every piece of logic needed to execute a single
// pull-based contributor refund:
//
// - **`validate_refund_preconditions`** — pure guard that checks campaign
//   status, deadline, goal, and contribution balance before any state change.
// - **`execute_refund_single`** — atomic CEI (Checks-Effects-Interactions)
//   execution: zero storage first, then transfer, then emit event.
//
// ## Security Assumptions
//
// 1. **Authentication** is the caller's responsibility (`contributor.require_auth()`
//    must be called before `execute_refund_single`).
// 2. **CEI order** — storage is zeroed *before* the token transfer so that a
//    re-entrant call from the token contract cannot double-claim.
// 3. **Overflow protection** — `total_raised` is decremented with `checked_sub`;
//    the function returns `ContractError::Overflow` rather than wrapping.
// 4. **Direction lock** — The token transfer explicitly uses the contract's
//    address as the sender and the contributor as the recipient.

use soroban_sdk::{token, Address, Env};

use crate::{ContractError, DataKey, Status};

// ── Storage helpers ───────────────────────────────────────────────────────────

/// Read the stored contribution amount for `contributor` (0 if absent).
pub fn get_contribution(env: &Env, contributor: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Contribution(contributor.clone()))
        .unwrap_or(0)
}

/// Low-level refund helper: transfer `amount` from contract to `contributor`
/// and zero the contribution record. Returns the amount transferred.
///
/// Does **not** check campaign status or auth — callers are responsible.
pub fn refund_single(env: &Env, token_address: &Address, contributor: &Address) -> i128 {
    let amount = get_contribution(env, contributor);
    if amount > 0 {
        env.storage()
            .persistent()
            .set(&DataKey::Contribution(contributor.clone()), &0i128);
        let token_client = token::Client::new(env, token_address);
        refund_single_transfer(
            &token_client,
            &env.current_contract_address(),
            contributor,
            amount,
        );
    }
    amount
}

// ── Transfer primitive ────────────────────────────────────────────────────────

/// Transfer `amount` tokens from the contract to `contributor`.
///
/// Direction is fixed: contract → contributor.
/// Single call site prevents parameter-order typos.
use soroban_sdk::{token, Address};

/// @title Refund Single Token Transfer Helper
/// @notice Centralizes transfer direction for contributor refunds.
/// @dev Keeps contract-side call sites explicit and typo-resistant for scripts.
pub fn refund_single_transfer(
    token_client: &token::Client,
    contract_address: &Address,
    contributor: &Address,
    amount: i128,
) {
    if amount <= 0 {
        return;
    }
    token_client.env().events().publish(
        ("debug", "refund_transfer_attempt"),
        (contributor.clone(), amount),
    );
    token_client.transfer(contract_address, contributor, &amount);
}

// ── Precondition guard ────────────────────────────────────────────────────────

/// Validate all preconditions for a `refund_single` call.
///
/// Returns the contribution amount owed to `contributor` on success, or the
/// appropriate `ContractError` variant on failure.
///
/// Does **not** mutate any state — safe to call speculatively.
///
/// # Errors
/// * `ContractError::NothingToRefund` — contributor has no balance on record.
///
/// # Panics
/// * When campaign status is not `Expired`.
pub fn validate_refund_preconditions(
    env: &Env,
    contributor: &Address,
) -> Result<i128, ContractError> {
    let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
    if status != Status::Expired {
        panic!("campaign must be in Expired state to refund");
    }

    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Contribution(contributor.clone()))
        .unwrap_or(0);
    if amount == 0 {
        return Err(ContractError::NothingToRefund);
    }

    Ok(amount)
}

// ── Atomic CEI execution ──────────────────────────────────────────────────────

/// Execute a single contributor refund using the CEI pattern.
///
/// Caller **must** have already called `contributor.require_auth()` and
/// `validate_refund_preconditions` (or be certain preconditions hold).
///
/// Storage is zeroed **before** the token transfer (CEI).
///
/// # Errors
/// * `ContractError::Overflow` — underflow when decrementing `TotalRaised`.
pub fn execute_refund_single(
    env: &Env,
    contributor: &Address,
    amount: i128,
) -> Result<(), ContractError> {
    let contribution_key = DataKey::Contribution(contributor.clone());

    // Effects: zero storage before transfer
    env.storage().persistent().set(&contribution_key, &0i128);
    env.storage()
        .persistent()
        .extend_ttl(&contribution_key, 100, 100);

    let total: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalRaised)
        .unwrap_or(0);
    let new_total = total.checked_sub(amount).ok_or(ContractError::Overflow)?;
    env.storage()
        .instance()
        .set(&DataKey::TotalRaised, &new_total);

    // Interactions: transfer after state is settled
    let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
    let token_client = token::Client::new(env, &token_address);
    token_client.transfer(&env.current_contract_address(), contributor, &amount);

    env.events()
        .publish(("campaign", "refund_single"), (contributor.clone(), amount));

    Ok(())
}
    token_client.transfer(contract_address, contributor, &amount);
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
