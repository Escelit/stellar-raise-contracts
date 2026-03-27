//! # access_control
//!
//! @title   AccessControl — Role separation and pausable logic for the crowdfund contract.
//!
//! @notice  Implements three distinct roles:
//!          - `DEFAULT_ADMIN_ROLE`: can upgrade, unpause, and transfer roles.
//!          - `PAUSER_ROLE`: can pause the contract in an emergency.
//!          - `CAMPAIGN_CREATOR`: manages campaign content only (no system params).
//!
//!          Platform fees are gated behind a `GovernanceAddress` (multisig or DAO).
//!
//! ## Security Assumptions
//! 1. Only `DEFAULT_ADMIN_ROLE` or `PAUSER_ROLE` may pause.
//! 2. Only `DEFAULT_ADMIN_ROLE` may unpause — asymmetric by design.
//! 3. Only `GovernanceAddress` may call `set_platform_fee`.
//! 4. `CAMPAIGN_CREATOR` has zero access to pause, unpause, or fee mutation.
//! 5. All role-sensitive actions emit an event for off-chain monitoring.

#![allow(dead_code)]

use soroban_sdk::{Address, Env, Symbol};

use crate::{ContractError, DataKey, PlatformConfig};

// ── Role helpers ─────────────────────────────────────────────────────────────

/// Returns the stored `DEFAULT_ADMIN_ROLE` address.
/// Panics if the contract has not been initialized.
pub fn get_default_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::DefaultAdmin)
        .expect("DEFAULT_ADMIN_ROLE not set")
}

/// Returns the stored `PAUSER_ROLE` address.
/// Panics if the contract has not been initialized.
pub fn get_pauser(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Pauser)
        .expect("PAUSER_ROLE not set")
}

/// Returns the stored `GovernanceAddress`.
/// Panics if the contract has not been initialized.
pub fn get_governance(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::GovernanceAddress)
        .expect("GovernanceAddress not set")
}

// ── Pause / unpause ──────────────────────────────────────────────────────────

/// @notice Pause the contract — blocks `contribute()` and `withdraw()`.
/// @dev    Either `PAUSER_ROLE` or `DEFAULT_ADMIN_ROLE` may call this.
///         Pausing is intentionally low-privilege so an emergency can be
///         triggered without the full admin key.
///
/// # Security
/// - `caller.require_auth()` ensures the transaction is signed by `caller`.
/// - Emits a `paused` event so monitors can alert immediately.
pub fn pause(env: &Env, caller: &Address) {
    caller.require_auth();

    let pauser = get_pauser(env);
    let admin = get_default_admin(env);

    if *caller != pauser && *caller != admin {
        panic!("not authorized to pause");
    }

    env.storage().instance().set(&DataKey::Paused, &true);

    env.events().publish(
        (Symbol::new(env, "access"), Symbol::new(env, "paused")),
        caller.clone(),
    );
}

/// @notice Unpause the contract.
/// @dev    Only `DEFAULT_ADMIN_ROLE` may unpause — stricter than pause.
///         This asymmetry limits the blast radius of a compromised pauser key:
///         the attacker can freeze the contract but cannot unfreeze it.
///
/// # Security
/// - `caller.require_auth()` ensures the transaction is signed by `caller`.
/// - Emits an `unpaused` event.
pub fn unpause(env: &Env, caller: &Address) {
    caller.require_auth();

    let admin = get_default_admin(env);
    if *caller != admin {
        panic!("only DEFAULT_ADMIN_ROLE can unpause");
    }

    env.storage().instance().set(&DataKey::Paused, &false);

    env.events().publish(
        (Symbol::new(env, "access"), Symbol::new(env, "unpaused")),
        caller.clone(),
    );
}

/// @notice Panics if the contract is currently paused.
/// @dev    Call at the top of any state-mutating function that should be
///         blocked during an emergency (e.g. `contribute`, `withdraw`).
pub fn assert_not_paused(env: &Env) {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        panic!("contract is paused");
    }
}

/// Returns `true` if the contract is currently paused.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

// ── Platform fee governance ──────────────────────────────────────────────────

/// @notice Update the platform fee configuration.
/// @dev    Only `GovernanceAddress` may call this — it must be a multisig or
///         DAO contract, never a plain EOA.  This prevents a single compromised
///         key from redirecting platform revenue.
///
/// # Arguments
/// * `caller` – Must match the stored `GovernanceAddress`.
/// * `config` – New fee config; `fee_bps` must be <= 10_000 (100%).
///
/// # Errors
/// * [`ContractError::InvalidPlatformFee`] if `fee_bps > 10_000`.
///
/// # Security
/// - `caller.require_auth()` ensures the governance multisig signed the tx.
/// - Emits a `fee_updated` event so off-chain monitors can detect unexpected changes.
pub fn set_platform_fee(
    env: &Env,
    caller: &Address,
    config: PlatformConfig,
) -> Result<(), ContractError> {
    caller.require_auth();

    let governance = get_governance(env);
    if *caller != governance {
        panic!("only GovernanceAddress can set platform fee");
    }

    if config.fee_bps > 10_000 {
        return Err(ContractError::InvalidPlatformFee);
    }

    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);

    env.events().publish(
        (
            Symbol::new(env, "governance"),
            Symbol::new(env, "fee_updated"),
        ),
        (caller.clone(), config.fee_bps),
    );

    Ok(())
}

// ── Role transfer ────────────────────────────────────────────────────────────

/// @notice Transfer `DEFAULT_ADMIN_ROLE` to a new address.
/// @dev    Only the current `DEFAULT_ADMIN_ROLE` may call this.
///         Emits a `role_transferred` event.
///
/// # Security
/// - Two-step transfer (propose + accept) is recommended for production;
///   this single-step version is provided as the minimal safe baseline.
pub fn transfer_default_admin(env: &Env, caller: &Address, new_admin: &Address) {
    caller.require_auth();

    let current = get_default_admin(env);
    if *caller != current {
        panic!("only DEFAULT_ADMIN_ROLE can transfer admin role");
    }

    env.storage()
        .instance()
        .set(&DataKey::DefaultAdmin, new_admin);

    env.events().publish(
        (
            Symbol::new(env, "access"),
            Symbol::new(env, "role_transferred"),
        ),
        (caller.clone(), new_admin.clone()),
    );
}

/// @notice Transfer `PAUSER_ROLE` to a new address.
/// @dev    Only `DEFAULT_ADMIN_ROLE` may reassign the pauser.
pub fn transfer_pauser(env: &Env, caller: &Address, new_pauser: &Address) {
    caller.require_auth();

    let admin = get_default_admin(env);
    if *caller != admin {
        panic!("only DEFAULT_ADMIN_ROLE can transfer pauser role");
    }

    env.storage().instance().set(&DataKey::Pauser, new_pauser);

    env.events().publish(
        (
            Symbol::new(env, "access"),
            Symbol::new(env, "pauser_transferred"),
        ),
        (caller.clone(), new_pauser.clone()),
    );
}
