//! contribute() error handling — reviewed and hardened.
//!
//! This module documents and re-exports the error variants relevant to
//! `contribute()`, and provides helper functions used by scripts and
//! off-chain tooling to interpret contract errors.
//!
//! # Error taxonomy for `contribute()`
//!
//! | Code | Variant              | Trigger                                      |
//! |------|----------------------|----------------------------------------------|
//! |  2   | `CampaignEnded`      | `ledger.timestamp > deadline`                |
//! |  6   | `Overflow`           | contribution or total_raised would overflow  |
//! |  —   | panic "not init"     | storage key missing (contract not initialized)|
//! |  —   | panic "below min"    | `amount < min_contribution`                  |
//!
//! # Security assumptions
//!
//! - `contributor.require_auth()` is called before any state mutation, so
//!   unauthenticated callers are rejected at the host level.
//! - Token transfer happens before storage writes; if the transfer fails the
//!   transaction is rolled back atomically — no partial state is persisted.
//! - Overflow is caught with `checked_add` on both the per-contributor total
//!   and the global total, returning `ContractError::Overflow` rather than
//!   wrapping silently.
//! - The deadline check uses `>` (strict), so contributions at exactly the
//!   deadline timestamp are accepted — scripts should account for this.
//!
//! # Known limitations / improvement opportunities
//!
//! 1. `amount < min_contribution` currently panics instead of returning a
//!    typed error. Scripts cannot distinguish this from other panics.
//!    Recommendation: add `ContractError::BelowMinimum` and return it.
//! 2. There is no check that `amount > 0`. A zero-amount contribution passes
//!    the minimum check when `min_contribution == 0` and wastes gas.
//!    Recommendation: add `ContractError::ZeroAmount`.
//! 3. The campaign `Status` is not checked in `contribute()`. A cancelled or
//!    successfully-withdrawn campaign still accepts contributions until the
//!    deadline. Recommendation: guard on `Status::Active`.

/// Numeric error codes returned by the contract host for `contribute()`.
/// Mirrors `ContractError` repr values for use in off-chain scripts.
pub mod error_codes {
    /// `contribute()` was called after the campaign deadline.
    pub const CAMPAIGN_ENDED: u32 = 2;
    /// A checked arithmetic operation overflowed.
    pub const OVERFLOW: u32 = 6;
}

/// Returns a human-readable description for a `contribute()` error code.
///
/// # Example
/// ```
/// use contribute_error_handling::describe_error;
/// assert_eq!(describe_error(2), "Campaign has ended");
/// ```
pub fn describe_error(code: u32) -> &'static str {
    match code {
        error_codes::CAMPAIGN_ENDED => "Campaign has ended",
        error_codes::OVERFLOW => "Arithmetic overflow — contribution amount too large",
        _ => "Unknown error",
    }
}

/// Returns `true` if the error code is retryable by the caller.
///
/// `CampaignEnded` and `Overflow` are permanent for the current campaign
/// state; neither can be resolved by retrying the same call.
pub fn is_retryable(_code: u32) -> bool {
    // Neither known error is retryable without a state change.
    false
}
