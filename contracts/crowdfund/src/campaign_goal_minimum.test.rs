//! Comprehensive tests for `campaign_goal_minimum` constants and validation helpers.
//!
//! # Coverage
//!
//! | Area                    | Cases                                                    |
//! |-------------------------|----------------------------------------------------------|
//! | Constants               | Correct values, stability, scale invariant               |
//! | `validate_goal`         | Minimum, above minimum, large, zero, negative, i128::MIN |
//! | `validate_goal_amount`  | Typed `GoalTooLow`, exact threshold, zero, negative      |
//! | `validate_min_contribution` | Floor, large, zero, negative, i128::MIN              |
//! | `validate_deadline`     | Exact offset, future, one-past, equal-now, past, overflow|
//! | `validate_platform_fee` | Zero, typical, exact cap, one-above, u32::MAX            |
//! | `compute_progress_bps`  | Zero raised, half, quarter, exact, over, massive over,   |
//! |                         | zero goal, negative goal, 1-of-large, min/min, 99%, 1bps |
//!
//! # Security notes
//!
//! - All helpers are pure functions; no state is mutated.
//! - `validate_goal_amount` returns `ContractError::GoalTooLow` (not a panic)
//!   so the frontend can map the error code to a user-facing message.
//! - `validate_deadline` uses `saturating_add` to prevent overflow when `now`
//!   is near `u64::MAX`.
//! - `compute_progress_bps` guards against division by zero and caps at
//!   `MAX_PROGRESS_BPS` for over-funded campaigns.

use crate::campaign_goal_minimum::{
    compute_progress_bps, validate_deadline, validate_goal, validate_goal_amount,
    validate_min_contribution, validate_platform_fee, MAX_PLATFORM_FEE_BPS, MAX_PROGRESS_BPS,
    MIN_CONTRIBUTION_AMOUNT, MIN_DEADLINE_OFFSET, MIN_GOAL_AMOUNT, PROGRESS_BPS_SCALE,
};
use crate::ContractError;
use soroban_sdk::Env;

// ── Constant stability assertions ─────────────────────────────────────────────

/// Ensures constants have not been accidentally changed.
/// These values are part of the public API; changing them is a breaking change.
#[test]
fn constants_have_expected_values() {
    assert_eq!(MIN_GOAL_AMOUNT, 1i128);
    assert_eq!(MIN_CONTRIBUTION_AMOUNT, 1i128);
    assert_eq!(MAX_PLATFORM_FEE_BPS, 10_000u32);
    assert_eq!(PROGRESS_BPS_SCALE, 10_000i128);
    assert_eq!(MIN_DEADLINE_OFFSET, 60u64);
    assert_eq!(MAX_PROGRESS_BPS, 10_000u32);
}

/// `PROGRESS_BPS_SCALE` and `MAX_PROGRESS_BPS` must be equal so that a
/// fully-met goal produces exactly `MAX_PROGRESS_BPS`.
#[test]
fn progress_scale_equals_max_progress_bps() {
    assert_eq!(PROGRESS_BPS_SCALE as u32, MAX_PROGRESS_BPS);
}

// ── validate_goal ─────────────────────────────────────────────────────────────

#[test]
fn validate_goal_accepts_minimum() {
    assert!(validate_goal(MIN_GOAL_AMOUNT).is_ok());
}

#[test]
fn validate_goal_accepts_one_above_minimum() {
    assert!(validate_goal(MIN_GOAL_AMOUNT + 1).is_ok());
}

#[test]
fn validate_goal_accepts_large_value() {
    assert!(validate_goal(1_000_000_000).is_ok());
}

#[test]
fn validate_goal_accepts_i128_max() {
    assert!(validate_goal(i128::MAX).is_ok());
}

#[test]
fn validate_goal_rejects_zero() {
    let err = validate_goal(0).unwrap_err();
    assert!(
        err.contains("MIN_GOAL_AMOUNT"),
        "error should mention MIN_GOAL_AMOUNT, got: {err}"
    );
}

#[test]
fn validate_goal_rejects_negative_one() {
    assert!(validate_goal(-1).is_err());
}

#[test]
fn validate_goal_rejects_i128_min() {
    assert!(validate_goal(i128::MIN).is_err());
}

// ── validate_min_contribution ─────────────────────────────────────────────────

#[test]
fn validate_min_contribution_accepts_floor() {
    assert!(validate_min_contribution(MIN_CONTRIBUTION_AMOUNT).is_ok());
}

#[test]
fn validate_min_contribution_accepts_one_above_floor() {
    assert!(validate_min_contribution(MIN_CONTRIBUTION_AMOUNT + 1).is_ok());
}

#[test]
fn validate_min_contribution_accepts_large_value() {
    assert!(validate_min_contribution(1_000_000).is_ok());
}

#[test]
fn validate_min_contribution_rejects_zero() {
    let err = validate_min_contribution(0).unwrap_err();
    assert!(
        err.contains("MIN_CONTRIBUTION_AMOUNT"),
        "error should mention MIN_CONTRIBUTION_AMOUNT, got: {err}"
    );
}

#[test]
fn validate_min_contribution_rejects_negative_one() {
    assert!(validate_min_contribution(-1).is_err());
}

#[test]
fn validate_min_contribution_rejects_i128_min() {
    assert!(validate_min_contribution(i128::MIN).is_err());
}

// ── validate_deadline ─────────────────────────────────────────────────────────

#[test]
fn validate_deadline_accepts_exact_offset() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now + MIN_DEADLINE_OFFSET).is_ok());
}

#[test]
fn validate_deadline_accepts_one_second_past_offset() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now + MIN_DEADLINE_OFFSET + 1).is_ok());
}

#[test]
fn validate_deadline_accepts_well_in_future() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now + 3_600).is_ok());
}

#[test]
fn validate_deadline_rejects_one_second_before_offset() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now + MIN_DEADLINE_OFFSET - 1).is_err());
}

#[test]
fn validate_deadline_rejects_equal_to_now() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now).is_err());
}

#[test]
fn validate_deadline_rejects_deadline_in_past() {
    let now: u64 = 1_000;
    assert!(validate_deadline(now, now - 1).is_err());
}

/// `saturating_add` must prevent a panic when `now` is near `u64::MAX`.
///
/// @security  Without `saturating_add`, `now + MIN_DEADLINE_OFFSET` would
///            wrap to a small value, making any deadline appear valid.
#[test]
fn validate_deadline_saturating_add_prevents_overflow() {
    let now = u64::MAX - 10;
    // saturating_add clamps to u64::MAX, so deadline == u64::MAX is still
    // rejected (u64::MAX < u64::MAX is false, but u64::MAX - 1 < u64::MAX).
    // The important thing is that this does not panic.
    let _ = validate_deadline(now, u64::MAX);
    let _ = validate_deadline(now, u64::MAX - 5);
}

// ── validate_platform_fee ─────────────────────────────────────────────────────

#[test]
fn validate_platform_fee_accepts_zero() {
    assert!(validate_platform_fee(0).is_ok());
}

#[test]
fn validate_platform_fee_accepts_typical_fee() {
    // 2.5 % — a realistic platform fee
    assert!(validate_platform_fee(250).is_ok());
}

#[test]
fn validate_platform_fee_accepts_exact_cap() {
    assert!(validate_platform_fee(MAX_PLATFORM_FEE_BPS).is_ok());
}

#[test]
fn validate_platform_fee_rejects_one_above_cap() {
    let err = validate_platform_fee(MAX_PLATFORM_FEE_BPS + 1).unwrap_err();
    assert!(
        err.contains("MAX_PLATFORM_FEE_BPS"),
        "error should mention MAX_PLATFORM_FEE_BPS, got: {err}"
    );
}

#[test]
fn validate_platform_fee_rejects_u32_max() {
    assert!(validate_platform_fee(u32::MAX).is_err());
}

// ── compute_progress_bps ─────────────────────────────────────────────────────

#[test]
fn compute_progress_bps_zero_raised() {
    assert_eq!(compute_progress_bps(0, 1_000_000), 0);
}

#[test]
fn compute_progress_bps_half_goal() {
    // 500_000 / 1_000_000 = 50 % = 5_000 bps
    assert_eq!(compute_progress_bps(500_000, 1_000_000), 5_000);
}

#[test]
fn compute_progress_bps_quarter_goal() {
    assert_eq!(compute_progress_bps(250_000, 1_000_000), 2_500);
}

#[test]
fn compute_progress_bps_99_percent() {
    // 9_900 / 10_000 = 99 % = 9_900 bps
    assert_eq!(compute_progress_bps(9_900, 10_000), 9_900);
}

#[test]
fn compute_progress_bps_1_bps() {
    // 1 / 10_000 = 0.01 % = 1 bps
    assert_eq!(compute_progress_bps(1, 10_000), 1);
}

#[test]
fn compute_progress_bps_exact_goal() {
    assert_eq!(compute_progress_bps(1_000_000, 1_000_000), MAX_PROGRESS_BPS);
}

#[test]
fn compute_progress_bps_over_goal_capped() {
    // 2× goal must still return MAX_PROGRESS_BPS, not 20_000.
    assert_eq!(compute_progress_bps(2_000_000, 1_000_000), MAX_PROGRESS_BPS);
}

#[test]
fn compute_progress_bps_massively_over_goal_capped() {
    assert_eq!(compute_progress_bps(i128::MAX, 1), MAX_PROGRESS_BPS);
}

#[test]
fn compute_progress_bps_zero_goal_returns_zero() {
    // Division-by-zero guard — must not panic.
    assert_eq!(compute_progress_bps(1_000, 0), 0);
}

#[test]
fn compute_progress_bps_negative_goal_returns_zero() {
    assert_eq!(compute_progress_bps(1_000, -1), 0);
}

#[test]
fn compute_progress_bps_one_token_of_large_goal() {
    // 1 / 1_000_000 rounds down to 0 bps — integer division.
    assert_eq!(compute_progress_bps(1, 1_000_000), 0);
}

#[test]
fn compute_progress_bps_minimum_goal_minimum_raised() {
    // 1 / 1 = 100 % = 10_000 bps
    assert_eq!(compute_progress_bps(1, 1), MAX_PROGRESS_BPS);
}

// ── validate_goal_amount (typed ContractError::GoalTooLow) ───────────────────

/// Goal exactly at the threshold is accepted.
#[test]
fn validate_goal_amount_accepts_exact_threshold() {
    let env = Env::default();
    assert!(validate_goal_amount(&env, MIN_GOAL_AMOUNT).is_ok());
}

/// Goal well above the threshold is accepted.
#[test]
fn validate_goal_amount_accepts_well_above_threshold() {
    let env = Env::default();
    assert!(validate_goal_amount(&env, 1_000_000_000).is_ok());
}

/// Goal one below threshold returns `ContractError::GoalTooLow`.
///
/// @security  This is the primary on-chain enforcement test.  If this fails,
///            zero-goal campaigns can be created and immediately drained.
#[test]
fn validate_goal_amount_rejects_below_threshold() {
    let env = Env::default();
    assert_eq!(
        validate_goal_amount(&env, MIN_GOAL_AMOUNT - 1),
        Err(ContractError::GoalTooLow)
    );
}

/// Zero goal returns `ContractError::GoalTooLow`.
#[test]
fn validate_goal_amount_rejects_zero() {
    let env = Env::default();
    assert_eq!(
        validate_goal_amount(&env, 0),
        Err(ContractError::GoalTooLow)
    );
}

/// Negative goal returns `ContractError::GoalTooLow`.
#[test]
fn validate_goal_amount_rejects_negative_one() {
    let env = Env::default();
    assert_eq!(
        validate_goal_amount(&env, -1),
        Err(ContractError::GoalTooLow)
    );
}

/// `i128::MIN` goal returns `ContractError::GoalTooLow` without panicking.
#[test]
fn validate_goal_amount_rejects_i128_min() {
    let env = Env::default();
    assert_eq!(
        validate_goal_amount(&env, i128::MIN),
        Err(ContractError::GoalTooLow)
    );
}

/// Calling `validate_goal_amount` twice with the same env is idempotent.
#[test]
fn validate_goal_amount_is_idempotent() {
    let env = Env::default();
    assert!(validate_goal_amount(&env, 100).is_ok());
    assert!(validate_goal_amount(&env, 100).is_ok());
}

/// `GoalTooLow` discriminant is stable and does not collide with other errors.
///
/// @security  Discriminant stability is required so that off-chain scripts
///            that map numeric codes to messages continue to work after upgrades.
#[test]
fn goal_too_low_discriminant_is_stable() {
    assert_eq!(ContractError::GoalTooLow as u32, 18);
}
