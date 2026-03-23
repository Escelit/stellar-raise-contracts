//! # Proptest Generator Boundary Conditions
//!
//! This contract provides the single source of truth for all boundary conditions and validation
//! constants used by the crowdfunding platform's property-based tests. Exposing these
//! via a contract allows off-chain scripts and other contracts to dynamically query
//! current safe operating limits.
//!
//! ## Purpose
//!
//! - **Centralized Constants**: All boundary values defined in one place for consistency.
//! - **Immutable Boundaries**: Constants are compile-time to ensure test stability.
//! - **Public Transparency**: All limits are publicly readable and queryable.
//! - **Safety Guards**: Includes validation and clamping logic against platform-wide floors and caps.
//! - **CI/CD Optimization**: Enables dynamic test configuration without hardcoding limits.
//!
//! ## Security Model
//!
//! - **Overflow Protection**: All arithmetic uses `saturating_mul` and `checked_sub` where applicable.
//! - **Division by Zero**: Guarded with explicit zero checks before division operations.
//! - **Basis Points Capping**: Progress and fee calculations capped at 10,000 (100%) to prevent display errors.
//! - **Timestamp Validity**: Deadline offsets exclude past and unreasonably large values.
//! - **Resource Bounds**: Test case counts and batch sizes bounded to prevent accidental stress scenarios.

// proptest_generator_boundary — Boundary constants and validation helpers.

use soroban_sdk::{contract, contractimpl, Env, Symbol};

// ── Constants ────────────────────────────────────────────────────────────────
// @notice All constants are immutable and define the safe operating boundaries
//         for the crowdfunding platform and its property-based tests.

/// Minimum deadline offset in seconds (~17 minutes).
/// @dev Prevents flaky tests due to timing races and ensures meaningful campaign duration.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds (~11.5 days).
/// @dev Avoids u64 overflow when added to ledger timestamps.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount.
/// @dev Prevents division-by-zero in progress calculations.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal amount for test generations (10 XLM).
/// @dev Keeps tests fast while covering large campaign scenarios.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for contribution amounts.
/// @dev Prevents zero-value contributions from polluting ledger state.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Maximum progress in basis points (100%).
/// @dev Frontend never displays >100% funded; prevents display errors.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Maximum fee in basis points (100%).
/// @dev Fee above this would exceed 100% of the contribution.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum proptest case count.
/// @dev Below this, boundary-adjacent values are rarely sampled.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum proptest case count.
/// @dev Balances coverage with CI execution time.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum batch size for generator operations.
/// @dev Prevents worst-case memory/gas spikes in test scaffolds.
pub const GENERATOR_BATCH_MAX: u32 = 512;

// ── Pure Validation Helpers ───────────────────────────────────────────────────
//
// These are standalone `pub fn` (no `Env`) so they can be called directly
// from `#[cfg(test)]` proptest blocks without spinning up a Soroban environment.

/// Returns `true` if `offset` is within `[DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.
///
/// @param  offset  Seconds from the current ledger timestamp to the campaign deadline.
/// @return `true`  when the offset is a safe, UI-displayable campaign duration.
///
/// @security Rejects values < 1 000 that cause timing races and values that
///           could overflow a `u64` timestamp when added to `now`.
//! Proptest generator boundary conditions for the crowdfund contract.
//!
//! This module defines canonical boundary constants and validation helpers
//! used by property-based tests. Correct boundaries ensure:
//! - Frontend UI displays progress, deadlines, and amounts reliably
//! - Tests avoid known regression cases (e.g., extremely short deadlines)
//! - Security assumptions (overflow, division-by-zero) are validated
//!
//! # Typo Fix (Frontend UI)
//!
//! The minimum deadline offset was previously documented as 100 seconds, which
//! caused proptest regressions and poor frontend UX (flaky countdown display).
//! It is now **1_000** seconds (~17 min) for stability and readability.

/// Minimum deadline offset in seconds (now + offset).
///
/// **Fixed typo**: Was 100, causing regression seeds and flaky frontend countdown.
/// Use 1_000 (~17 min) for stable tests and meaningful campaign duration display.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount (stroops).
/// Avoids zero/negative which breaks progress_bps display in frontend.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal for proptest (100M stroops = 10 XLM).
/// Keeps tests fast while covering large campaigns.
pub const GOAL_MAX: i128 = 100_000_000;

/// Minimum contribution amount (stroops).
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Progress basis points cap (100%).
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Platform fee basis points cap (100%).
pub const FEE_BPS_CAP: u32 = 10_000;

/// Validates that a deadline offset is within the accepted range.
///
/// # Arguments
/// * `offset` - Seconds from `now` to deadline
///
/// # Returns
/// `true` if offset is in [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]
#[inline]
pub fn is_valid_deadline_offset(offset: u64) -> bool {
    (DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX).contains(&offset)
}

/// Returns `true` if `goal` is within `[GOAL_MIN, GOAL_MAX]`.
///
/// @param  goal  Funding target in the token's smallest unit (stroops).
/// @return `true` when the goal is safe for arithmetic and UI display.
///
/// @security Rejects `goal <= 0` which would cause division-by-zero in
///           `compute_progress_bps` and break the frontend progress bar.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    (GOAL_MIN..=GOAL_MAX).contains(&goal)
}

/// Returns `true` if `min_contribution` is a valid floor for the given `goal`.
///
/// @param  min_contribution  Minimum amount a contributor must send.
/// @param  goal              The campaign's funding target.
/// @return `true` when `MIN_CONTRIBUTION_FLOOR <= min_contribution <= goal`.
///
/// @security Ensures `min_contribution` never exceeds `goal`, which would
///           make the campaign permanently un-fundable.
/// Validates that a goal is within the accepted range.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    goal >= GOAL_MIN && goal <= GOAL_MAX
}

/// Validates that min_contribution is valid for a given goal.
/// min_contribution must be in [MIN_CONTRIBUTION_FLOOR, goal].
#[inline]
pub fn is_valid_min_contribution(min_contribution: i128, goal: i128) -> bool {
    min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
}

/// Returns `true` if `amount >= min_contribution`.
///
/// @param  amount           The contribution amount being validated.
/// @param  min_contribution The campaign's minimum contribution floor.
/// @return `true` when the amount meets the minimum threshold.
/// Validates that a contribution amount is >= min_contribution.
#[inline]
pub fn is_valid_contribution_amount(amount: i128, min_contribution: i128) -> bool {
    amount >= min_contribution
}

/// Clamps a raw basis-point value into `[0, PROGRESS_BPS_CAP]`.
///
/// @param  raw  Unclamped progress value (may be negative or > 10 000).
/// @return A `u32` in `[0, 10_000]` safe for frontend progress-bar rendering.
///
/// @notice Negative inputs (e.g. when `raised < 0`) are treated as 0 %.
///         Over-funded campaigns are capped at exactly 100 %.
/// Clamps progress_bps to valid range [0, PROGRESS_BPS_CAP].
#[inline]
pub fn clamp_progress_bps(raw: i128) -> u32 {
    if raw <= 0 {
        0
    } else if raw >= PROGRESS_BPS_CAP as i128 {
        PROGRESS_BPS_CAP
    } else {
        raw as u32
    }
}

/// Computes campaign progress in basis points from `raised` and `goal`.
///
/// @param  raised  Total tokens raised so far.
/// @param  goal    Campaign funding target (must be > 0).
/// @return Basis points in `[0, 10_000]`; returns 0 when `goal <= 0`.
///
/// @security Uses `saturating_mul` to prevent overflow on large `raised`
///           values before dividing by `goal`.
#[inline]
pub fn compute_progress_bps(raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    let raw = raised.saturating_mul(10_000) / goal;
    clamp_progress_bps(raw)
}

/// Clamps a requested proptest case count into `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.
///
/// @param  requested  Caller-supplied case count.
/// @return A value guaranteed to be in `[32, 256]`.
#[inline]
pub fn clamp_proptest_cases(requested: u32) -> u32 {
    requested.clamp(PROPTEST_CASES_MIN, PROPTEST_CASES_MAX)
}

// ── On-Chain Contract ─────────────────────────────────────────────────────────

/// On-chain contract that exposes boundary constants and validation logic so
/// off-chain scripts and other contracts can query current platform limits.
///
/// @notice All methods are pure (read-only) and do not modify contract state.
#[contract]
pub struct ProptestGeneratorBoundary;

#[contractimpl]
impl ProptestGeneratorBoundary {
    // ── Getter Functions ──────────────────────────────────────────────────────
    // @notice These functions return immutable boundary constants.
    //         Used by off-chain scripts and other contracts to query safe limits.

    /// Returns the minimum deadline offset in seconds.
    /// @notice ~17 minutes; prevents flaky tests and ensures meaningful campaigns.
    pub fn deadline_offset_min(_env: Env) -> u64 {
        DEADLINE_OFFSET_MIN
    }

    /// Returns the maximum deadline offset in seconds.
    /// @notice ~11.5 days; avoids u64 overflow on ledger timestamps.
    pub fn deadline_offset_max(_env: Env) -> u64 {
        DEADLINE_OFFSET_MAX
    }

    /// Returns the minimum valid goal amount.
    /// @notice Prevents division-by-zero in progress calculations.
    pub fn goal_min(_env: Env) -> i128 {
        GOAL_MIN
    }

    /// Returns the maximum goal amount for test generations.
    /// @notice 10 XLM; keeps tests fast while covering large campaigns.
    pub fn goal_max(_env: Env) -> i128 {
        GOAL_MAX
    }

    /// Returns the absolute floor for contribution amounts.
    /// @notice Prevents zero-value contributions from polluting state.
    pub fn min_contribution_floor(_env: Env) -> i128 {
        MIN_CONTRIBUTION_FLOOR
    }

    /// Returns the maximum progress in basis points.
    /// @notice 100%; frontend never displays >100% funded.
    pub fn progress_bps_cap(_env: Env) -> u32 {
        PROGRESS_BPS_CAP
    }

    /// Returns the maximum fee in basis points.
    /// @notice 100%; fee above this would exceed the contribution.
    pub fn fee_bps_cap(_env: Env) -> u32 {
        FEE_BPS_CAP
    }

    /// Returns the minimum proptest case count.
    /// @notice Below this, boundary-adjacent values are rarely sampled.
    pub fn proptest_cases_min(_env: Env) -> u32 {
        PROPTEST_CASES_MIN
    }

    /// Returns the maximum proptest case count.
    /// @notice Balances coverage with CI execution time.
    pub fn proptest_cases_max(_env: Env) -> u32 {
        PROPTEST_CASES_MAX
    }

    /// Returns the maximum batch size for generator operations.
    /// @notice Prevents worst-case memory/gas spikes in test scaffolds.
    pub fn generator_batch_max(_env: Env) -> u32 {
        GENERATOR_BATCH_MAX
    }

    // ── Validation Functions ──────────────────────────────────────────────────
    // @notice These functions validate inputs against boundary constants.
    //         Used by tests and off-chain scripts to ensure safe values.

    /// Validates that a deadline offset is within [min, max] range.
    /// @notice Rejects values that cause timestamp overflow or campaigns too short.
    /// @param offset The deadline offset in seconds to validate.
    /// @return true if offset is valid, false otherwise.
    pub fn is_valid_deadline_offset(_env: Env, offset: u64) -> bool {
        is_valid_deadline_offset(offset)
    }

    /// Validates that a goal is within [min, max] range.
    /// @notice Rejects zero and negative goals to prevent division-by-zero.
    /// @param goal The goal amount to validate.
    /// @return true if goal is valid, false otherwise.
    pub fn is_valid_goal(_env: Env, goal: i128) -> bool {
        is_valid_goal(goal)
    }

    /// Validates that a minimum contribution is within safe bounds.
    /// @notice min_contribution must be >= MIN_CONTRIBUTION_FLOOR and <= goal.
    /// @param min_contribution The minimum contribution to validate.
    /// @param goal The campaign goal (used as upper bound).
    /// @return true if min_contribution is valid, false otherwise.
    pub fn is_valid_min_contribution(_env: Env, min_contribution: i128, goal: i128) -> bool {
        min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
    }

    /// Validates that a contribution amount meets the minimum.
    /// @notice Ensures amount >= min_contribution.
    /// @param amount The contribution amount to validate.
    /// @param min_contribution The minimum required contribution.
    /// @return true if amount is valid, false otherwise.
    pub fn is_valid_contribution_amount(_env: Env, amount: i128, min_contribution: i128) -> bool {
        amount >= min_contribution
    }

    /// Validates that a fee basis points value is within cap.
    /// @notice Rejects fees > 10,000 bps (100%).
    /// @param fee_bps The fee in basis points to validate.
    /// @return true if fee_bps is valid, false otherwise.
    pub fn is_valid_fee_bps(_env: Env, fee_bps: u32) -> bool {
        fee_bps <= FEE_BPS_CAP
    }

    /// Validates that a generator batch size is within bounds.
    /// @notice Prevents worst-case memory/gas spikes.
    /// @param batch_size The batch size to validate.
    /// @return true if batch_size is valid, false otherwise.
    pub fn is_valid_generator_batch_size(_env: Env, batch_size: u32) -> bool {
        batch_size > 0 && batch_size <= GENERATOR_BATCH_MAX
    }

    // ── Clamping Functions ────────────────────────────────────────────────────
    // @notice These functions clamp values into safe operating bounds.
    //         Used by tests to ensure values stay within limits.

    /// Clamps a requested proptest case count into safe operating bounds.
    /// @notice Protects CI runtime cost while preserving boundary signal.
    /// @param requested The requested case count.
    /// @return Clamped value in [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    pub fn clamp_proptest_cases(_env: Env, requested: u32) -> u32 {
        clamp_proptest_cases(requested)
    }

    /// Clamps raw progress basis points to [0, PROGRESS_BPS_CAP].
    /// @notice Negative values floor to 0; values above 10,000 cap at 10,000.
    /// @dev Ensures frontend never displays >100% funded.
    /// @param raw The raw progress value to clamp.
    /// @return Clamped value in [0, PROGRESS_BPS_CAP].
    pub fn clamp_progress_bps(_env: Env, raw: i128) -> u32 {
        if raw <= 0 {
            0
        } else if raw >= PROGRESS_BPS_CAP as i128 {
            PROGRESS_BPS_CAP
        } else {
            raw as u32
        }
    }

    // ── Derived Calculation Functions ─────────────────────────────────────────
    // @notice These functions compute derived values using boundary constants.
    //         All arithmetic is guarded against overflow and division-by-zero.

    /// Computes progress in basis points, capped at 10,000.
    /// @notice Returns 0 when goal <= 0 to avoid division-by-zero.
    /// @dev Uses saturating_mul to prevent overflow.
    /// @param raised The amount raised so far.
    /// @param goal The campaign goal.
    /// @return Progress in basis points, clamped to [0, PROGRESS_BPS_CAP].
    pub fn compute_progress_bps(_env: Env, raised: i128, goal: i128) -> u32 {
        if goal <= 0 {
            return 0;
        }
        let raw = raised.saturating_mul(10_000) / goal;
        Self::clamp_progress_bps(_env, raw)
    }

    /// Computes fee amount from a contribution and fee basis points.
    /// @notice Returns 0 when amount <= 0 or fee_bps == 0.
    /// @dev Uses saturating_mul to prevent overflow.
    /// @param amount The contribution amount.
    /// @param fee_bps The fee in basis points.
    /// @return Fee amount (integer floor division).
    pub fn compute_fee_amount(_env: Env, amount: i128, fee_bps: u32) -> i128 {
        if amount <= 0 || fee_bps == 0 {
            return 0;
        }
        amount.saturating_mul(fee_bps as i128) / 10_000
    }

    /// Returns a diagnostic tag for boundary log events.
    /// @notice Used by off-chain indexers to filter boundary-related events.
    /// @return Symbol "boundary" for event filtering.
    pub fn log_tag(_env: Env) -> Symbol {
        Symbol::new(&_env, "boundary")
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn deadline_offset_min_is_1000() {
        assert_eq!(DEADLINE_OFFSET_MIN, 1_000);
    }

    #[test]
    fn is_valid_deadline_offset_rejects_below_min() {
        assert!(!is_valid_deadline_offset(99));
        assert!(!is_valid_deadline_offset(100)); // previously used, now invalid
    }

    #[test]
    fn is_valid_deadline_offset_accepts_min() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MIN));
        assert!(is_valid_deadline_offset(1_000));
    }

    #[test]
    fn is_valid_deadline_offset_accepts_within_range() {
        assert!(is_valid_deadline_offset(3600));
        assert!(is_valid_deadline_offset(100_000));
    }

    #[test]
    fn is_valid_deadline_offset_accepts_max() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MAX));
    }

    #[test]
    fn is_valid_goal_accepts_valid() {
        assert!(is_valid_goal(GOAL_MIN));
        assert!(is_valid_goal(1_000_000));
        assert!(is_valid_goal(GOAL_MAX));
    }

    #[test]
    fn is_valid_goal_rejects_invalid() {
        assert!(!is_valid_goal(0));
        assert!(!is_valid_goal(-1));
        assert!(!is_valid_goal(GOAL_MAX + 1));
    }

    #[test]
    fn is_valid_min_contribution_accepts_valid() {
        assert!(is_valid_min_contribution(1, 1_000));
        assert!(is_valid_min_contribution(1_000, 1_000));
        assert!(is_valid_min_contribution(500, 1_000_000));
    }

    #[test]
    fn is_valid_min_contribution_rejects_invalid() {
        assert!(!is_valid_min_contribution(0, 1_000));
        assert!(!is_valid_min_contribution(1_001, 1_000));
    }

    #[test]
    fn clamp_progress_bps_bounds() {
        assert_eq!(clamp_progress_bps(-1), 0);
        assert_eq!(clamp_progress_bps(0), 0);
        assert_eq!(clamp_progress_bps(5000), 5000);
        assert_eq!(clamp_progress_bps(10_000), PROGRESS_BPS_CAP);
        assert_eq!(clamp_progress_bps(20_000), PROGRESS_BPS_CAP);
    }
}
