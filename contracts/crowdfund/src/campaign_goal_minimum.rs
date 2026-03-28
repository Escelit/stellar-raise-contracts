//! # Campaign goal minimum threshold
//!
//! Centralized enforcement of minimum campaign goal, contribution floor, deadline
//! offset, platform fee cap, and progress basis points. Prefer
//! [`validate_goal_amount`] for any on-chain path that must return a typed
//! [`crate::ContractError`]; the string-based helpers remain for legacy call sites
//! and off-chain tooling but are deprecated where a typed error exists.

use soroban_sdk::Env;

// ── Constants ─────────────────────────────────────────────────────────────────

/// @title Minimum funding goal
/// @notice Any campaign goal must be at least this many token base units.
/// @dev A goal of `0` would allow trivial “successful” campaigns with no funding
///      target — a known drain / logic-bypass pattern. The floor `1` is the
///      smallest positive `i128` goal.
/// @custom:security Baked into WASM; changing requires contract upgrade and
///                  coordinated indexer / client updates.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// @title Minimum per-contribution amount
/// @notice Each contribution must be at least this many token base units.
/// @custom:security Prevents zero-amount transfers that spam storage or events.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// @title Minimum deadline horizon
/// @notice Deadline must be at least this many seconds after the current ledger time.
/// @custom:security Uses `saturating_add` on `u64` so `now` near `u64::MAX` cannot wrap.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// @title Platform fee ceiling
/// @notice Fee in basis points must not exceed this value (10_000 = 100%).
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// @title Progress scale
/// @notice Denominator for converting raised/goal ratio to basis points.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// @title Maximum progress basis points
/// @notice Progress UI is capped so over-funded campaigns do not report &gt; 100%.
pub const MAX_PROGRESS_BPS: u32 = 10_000;

// ── Legacy string-error API (deprecated) ─────────────────────────────────────

/// @notice Returns `Ok(())` if `goal >= MIN_GOAL_AMOUNT`.
/// @dev **Deprecated** — use [`validate_goal_amount`] for Soroban paths that map to
///      [`crate::ContractError`]. Kept for backward compatibility and tests.
/// @param goal Campaign goal in token base units.
#[deprecated(
    note = "use validate_goal_amount(&env, goal) and map ContractError for on-chain initialization"
)]
#[inline]
pub fn validate_goal(goal: i128) -> Result<(), &'static str> {
    if goal < MIN_GOAL_AMOUNT {
        return Err("goal must be at least MIN_GOAL_AMOUNT");
    }
    Ok(())
}

/// @title Canonical on-chain goal floor check
/// @notice Returns `Ok(())` if `goal_amount >= MIN_GOAL_AMOUNT`.
/// @param _env Soroban environment (reserved for future ledger-aware rules).
/// @param goal_amount Campaign goal in token base units.
/// @return `Err(ContractError::GoalTooLow)` if below floor; otherwise `Ok(())`.
/// @dev Single signed comparison — no arithmetic, so no overflow.
/// @custom:security Must run before persisting campaign state that depends on `goal`.
#[inline]
pub fn validate_goal_amount(
    _env: &Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::GoalTooLow);
    }
    Ok(())
}

/// @notice Returns `Ok(())` if `min_contribution >= MIN_CONTRIBUTION_AMOUNT`.
/// @param min_contribution Minimum contribution in token base units.
#[inline]
pub fn validate_min_contribution(min_contribution: i128) -> Result<(), &'static str> {
    if min_contribution < MIN_CONTRIBUTION_AMOUNT {
        return Err("min_contribution must be at least MIN_CONTRIBUTION_AMOUNT");
    }
    Ok(())
}

/// @notice Returns `Ok(())` if `deadline >= now + MIN_DEADLINE_OFFSET` (saturating).
/// @param now Current ledger timestamp (seconds).
/// @param deadline Campaign deadline (seconds).
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    let min_deadline = now.saturating_add(MIN_DEADLINE_OFFSET);
    if deadline < min_deadline {
        return Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future");
    }
    Ok(())
}

/// @notice Returns `Ok(())` if `fee_bps <= MAX_PLATFORM_FEE_BPS`.
/// @param fee_bps Platform fee in basis points.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("fee_bps must not exceed MAX_PLATFORM_FEE_BPS");
    }
    Ok(())
}

// ── Progress computation ─────────────────────────────────────────────────────

/// @title Funding progress in basis points
/// @notice Computes `min(10_000, (total_raised * PROGRESS_BPS_SCALE) / goal)`.
/// @param total_raised Sum of contributions in token base units.
/// @param goal Campaign goal in token base units.
/// @return Basis points from 0 through [`MAX_PROGRESS_BPS`].
/// @dev Returns `0` if `goal <= 0` or `total_raised <= 0`. Uses `saturating_mul` so
///      `total_raised * PROGRESS_BPS_SCALE` never panics in debug builds when raised
///      is huge (e.g. `i128::MAX` with `goal == 1`); the quotient is then capped at
///      [`MAX_PROGRESS_BPS`].
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if total_raised <= 0 || goal <= 0 {
        return 0;
    }

    let raw_progress = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw_progress <= 0 {
        0
    } else if raw_progress >= MAX_PROGRESS_BPS as i128 {
        MAX_PROGRESS_BPS
    } else {
        raw_progress as u32
    }
}
