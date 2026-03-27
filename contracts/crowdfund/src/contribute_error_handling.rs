//! contribute() error handling — typed errors and diagnostic helpers.
//!
//! @title   ContributeErrorHandling
//! @notice  Centralizes error codes and helpers for the `contribute()` entry
//!          point. All error conditions are represented as typed `ContractError`
//!          variants; this module re-exports their numeric codes so off-chain
//!          scripts can map raw codes to human-readable descriptions without
//!          embedding magic numbers.
//!
//! # Error taxonomy for `contribute()`
//!
//! | Code | Variant              | Trigger                                          |
//! |------|----------------------|--------------------------------------------------|
//! |  2   | `CampaignEnded`      | `ledger.timestamp > deadline`                    |
//! |  6   | `Overflow`           | contribution or total_raised would overflow      |
//! |  8   | `ZeroAmount`         | `amount == 0`                                    |
//! |  9   | `BelowMinimum`       | `amount < min_contribution`                      |
//! | 10   | `CampaignNotActive`  | campaign status is not `Active`                  |
//! | 11   | `NegativeAmount`     | `amount < 0`                                     |
//!
//! # Security assumptions
//!
//! - `contributor.require_auth()` is called before any state mutation.
//! - Negative amounts are rejected before zero/minimum checks.
//! - Campaign status is checked first; cancelled/succeeded campaigns are
//!   rejected before any other validation.
//! - Overflow is caught with `checked_add` on both per-contributor and global totals.
//! - The deadline check uses strict `>`, so contributions at exactly the
//!   deadline timestamp are accepted.

/// Numeric error codes returned by the contract host for `contribute()`.
///
/// These mirror the `#[repr(u32)]` values of `ContractError` and are intended
/// for use in off-chain scripts that inspect raw error codes.
pub mod error_codes {
    /// `contribute()` was called after the campaign deadline.
    pub const CAMPAIGN_ENDED: u32 = 2;
    /// A checked arithmetic operation overflowed.
    pub const OVERFLOW: u32 = 6;
    /// `amount` was zero.
    pub const ZERO_AMOUNT: u32 = 13;
    /// `amount` was below `min_contribution`.
    pub const BELOW_MINIMUM: u32 = 14;
    /// Campaign status is not `Active`.
    pub const CAMPAIGN_NOT_ACTIVE: u32 = 15;
    /// `amount` was negative.
    pub const NEGATIVE_AMOUNT: u32 = 16;
    /// Alias kept for off-chain scripts that used the old code 9.
    /// Prefer BELOW_MINIMUM (14).
    pub const AMOUNT_TOO_LOW: u32 = BELOW_MINIMUM;
}

/// Returns a human-readable description for a `contribute()` error code.
///
/// @param  code  The `ContractError` repr value (e.g. `e as u32`).
/// @return       A static string suitable for logging or user-facing messages.
pub fn describe_error(code: u32) -> &'static str {
    match code {
        error_codes::CAMPAIGN_ENDED => "Campaign has ended",
        error_codes::OVERFLOW => "Arithmetic overflow — contribution amount too large",
        error_codes::ZERO_AMOUNT => "Contribution amount must be greater than zero",
        error_codes::BELOW_MINIMUM => "Contribution amount is below the minimum required",
        error_codes::CAMPAIGN_NOT_ACTIVE => "Campaign is not active",
        error_codes::NEGATIVE_AMOUNT => "Contribution amount must not be negative",
        _ => "Unknown error",
    }
}

/// Returns `true` if the error is one the caller can fix by changing their
/// input and retrying (input errors), `false` for permanent campaign-state errors.
///
/// - `ZeroAmount`, `BelowMinimum`, `NegativeAmount` → retryable (fix the amount).
/// - `CampaignEnded`, `CampaignNotActive`, `Overflow` → not retryable.
pub fn is_retryable(code: u32) -> bool {
    matches!(
        code,
        error_codes::ZERO_AMOUNT | error_codes::BELOW_MINIMUM | error_codes::NEGATIVE_AMOUNT
    )
}

/// Emits a structured diagnostic event for a `contribute()` error.
///
/// # Event schema
///
/// | Field   | Value                        |
/// |---------|------------------------------|
/// | topic 0 | `"contribute_error"`         |
/// | topic 1 | `Symbol(<variant_name>)`     |
/// | data    | `u32` error code             |
///
/// # Security
///
/// Read-only diagnostic data only. Does not mutate contract state and cannot
/// be called externally — invoked exclusively from within `contribute()`.
pub fn log_contribute_error(env: &soroban_sdk::Env, error: crate::ContractError) {
    use soroban_sdk::Symbol;
    let (variant, code) = match error {
        crate::ContractError::CampaignEnded => (
            Symbol::new(env, "CampaignEnded"),
            error_codes::CAMPAIGN_ENDED,
        ),
        crate::ContractError::Overflow => (Symbol::new(env, "Overflow"), error_codes::OVERFLOW),
        crate::ContractError::ZeroAmount => {
            (Symbol::new(env, "ZeroAmount"), error_codes::ZERO_AMOUNT)
        }
        crate::ContractError::BelowMinimum => {
            (Symbol::new(env, "BelowMinimum"), error_codes::BELOW_MINIMUM)
        }
        crate::ContractError::CampaignNotActive => (
            Symbol::new(env, "CampaignNotActive"),
            error_codes::CAMPAIGN_NOT_ACTIVE,
        ),
        _ => return,
    };
    env.events().publish(("contribute_error", variant), code);
}
