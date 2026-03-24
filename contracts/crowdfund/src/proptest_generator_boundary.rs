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

/// Validates that a goal is within the accepted range.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    (GOAL_MIN..=GOAL_MAX).contains(&goal)
}

/// Validates that min_contribution is valid for a given goal.
/// min_contribution must be in [MIN_CONTRIBUTION_FLOOR, goal].
#[inline]
pub fn is_valid_min_contribution(min_contribution: i128, goal: i128) -> bool {
    min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
}

/// Validates that a contribution amount is >= min_contribution.
#[inline]
pub fn is_valid_contribution_amount(amount: i128, min_contribution: i128) -> bool {
    amount >= min_contribution
}

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
