//! # `refund_single` Token Transfer Logic
//!
//! This module centralises every piece of logic needed to execute a single
//! pull-based contributor refund:
//!
//! - **`validate_refund_preconditions`** — pure guard that checks campaign
//!   status, deadline, goal, and contribution balance before any state change.
//! - **`execute_refund_single`** — atomic CEI (Checks-Effects-Interactions)
//!   execution: zero storage first, then transfer, then emit event.
//!
//! ## Security Assumptions
//!
//! 1. **Authentication** is the caller's responsibility (`contributor.require_auth()`
//!    must be called before `execute_refund_single`).
//! 2. **CEI order** — storage is zeroed *before* the token transfer so that a
//!    re-entrant call from the token contract cannot double-claim.
//! 3. **Overflow protection** — `total_raised` is decremented with `checked_sub`;
//!    the function returns `ContractError::Overflow` rather than wrapping.
//! 4. **Direction lock** — The token transfer explicitly uses the contract's
//!    address as the sender and the contributor as the recipient.
//! 5. **Token interface** — Transfers use Soroban [`token::Client`]; the campaign
//!    must hold sufficient balance; malicious tokens are out of scope (standard
//!    Stellar asset / SAC assumptions).
//!
//! ## Dependencies (review checklist)
//!
//! - [`crate::DataKey`] — persistent `Contribution(addr)` and instance `Token`, `TotalRaised`, `Status`.
//! - [`crate::ContractError`] — `NothingToRefund`, `Overflow`.
//! - `soroban_sdk::token` — SEP-41-style `transfer(from, to, amount)`.

use soroban_sdk::{token, Address, Env};
use crate::{ContractError, DataKey, Status};

// ── Storage helpers ───────────────────────────────────────────────────────────

/// @title Get contribution balance
/// @notice Reads `DataKey::Contribution(contributor)` from persistent storage; `0` if missing.
/// @param env Soroban environment (caller must be in correct contract context when used with storage).
/// @param contributor Account whose recorded contribution is queried.
/// @return Recorded amount in token smallest units.
/// @custom:security View-only; does not authenticate `contributor`.
pub fn get_contribution(env: &Env, contributor: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Contribution(contributor.clone()))
        .unwrap_or(0)
}

/// @title Legacy single-contributor refund helper
/// @notice Zeros `Contribution(contributor)` then transfers `amount` via [`refund_single_transfer`].
/// @notice Returns the balance that *was* stored (may be `0`); skips transfer when `amount <= 0` after read.
/// @dev Prefer the public `CrowdfundContract::refund_single` path (`validate` → `execute_refund_single`) for auth + status.
/// @dev Kept for tests and any internal batch paths that already validated state.
/// @custom:security Does **not** check `Status::Expired` or `require_auth`; callers must enforce invariants.
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

/// @title Token transfer primitive for refunds
/// @notice Invokes `token_client.transfer(contract_address, contributor, amount)`; no-op if `amount <= 0`.
/// @dev Publishes a debug event before transfer for testability and off-chain indexing.
/// @param token_client Soroban token client bound to the campaign asset.
/// @param contract_address Must equal `env.current_contract_address()` at the call site (funds source).
/// @param contributor Refund recipient.
/// @param amount Amount in token smallest units.
/// @custom:security Caller must ensure `contract_address` is the crowdfund contract and holds balance.
pub fn refund_single_transfer(
    token_client: &token::Client,
    contract_address: &Address,
    contributor: &Address,
    amount: i128,
) {
    if amount <= 0 {
        // Early return prevents gas waste on zero/non-positive amounts
        return;
    }

    token_client.env.events().publish(
        ("debug", "refund_transfer_attempt"),
        (contributor.clone(), amount),
    );

    token_client.transfer(contract_address, contributor, &amount);
}

// ── Precondition guard ────────────────────────────────────────────────────────

/// @title Refund precondition guard
/// @notice Read-only: validates `Status::Expired` and non-zero contribution.
/// @param env Soroban environment (instance + persistent storage).
/// @param contributor Address requesting a refund.
/// @return `Ok(amount)` when status is `Expired` and contribution > 0.
/// @return `Err(ContractError::NothingToRefund)` when contribution record is zero or missing.
///
/// # Panics
/// * `"campaign must be in Expired state to refund"` — status is `Active`, `Succeeded`, or `Cancelled`
///   (callers should use [`crate::CrowdfundContract::refund_available`] for a Result-based preview where applicable).
///
/// @custom:security Does not authenticate; [`crate::CrowdfundContract::refund_single`] wraps with `require_auth`.
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

/// @title Atomic single-refund (CEI)
/// @notice Zeros contribution, updates `total_raised`, transfers tokens, emits `("campaign","refund_single")`.
/// @dev Caller **must** have invoked `contributor.require_auth()` and passed `validate_refund_preconditions`
///      (or equivalent guarantees). **`amount` must equal the contribution that was stored**; passing a
///      mismatched value can corrupt accounting (contribution cleared before `total_raised` update) even when
///      signed `checked_sub` does not return [`ContractError::Overflow`].
/// @param env Soroban environment.
/// @param contributor Refund recipient (must match keyed `Contribution`).
/// @param amount Refund quantity; must match prior stored contribution for consistent accounting.
/// @return `Ok(())` on success, `Err(ContractError::Overflow)` if `checked_sub` overflows `i128`
///      (pathological totals; normal refunds use validated amounts).
///
/// @custom:security CEI prevents re-entrancy double-spend; token transfer is last.
pub fn execute_refund_single(
    env: &Env,
    contributor: &Address,
    amount: i128,
) -> Result<(), ContractError> {
    let contribution_key = DataKey::Contribution(contributor.clone());

    // ── Effects (zero storage before transfer) ────────────────────────────
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

    // ── Interactions (transfer after state is settled) ────────────────────
    let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
    let token_client = token::Client::new(env, &token_address);

    // Explicitly transfer from contract to contributor
    token_client.transfer(&env.current_contract_address(), contributor, &amount);

    env.events()
        .publish(("campaign", "refund_single"), (contributor.clone(), amount));

    Ok(())
}
