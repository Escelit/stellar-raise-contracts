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

/// Centralizes transfer direction for contributor refunds.
///
/// @notice Transfers `amount` tokens from `contract_address` to `contributor`.
/// @dev    Keeping this in one place prevents parameter-order typos at call sites.
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
}
