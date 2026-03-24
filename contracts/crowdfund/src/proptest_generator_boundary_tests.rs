//! Comprehensive tests for proptest generator boundary conditions.
//!
//! Ensures boundary constants and validators behave correctly for frontend UI
//! display and property-based test stability.

use proptest::prelude::*;
use proptest::strategy::Just;

use crate::proptest_generator_boundary::{
    clamp_progress_bps, is_valid_contribution_amount, is_valid_deadline_offset, is_valid_goal,
    is_valid_min_contribution, DEADLINE_OFFSET_MAX, DEADLINE_OFFSET_MIN, FEE_BPS_CAP, GOAL_MAX,
    GOAL_MIN, MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP,
};

// ── Strategy definitions ─────────────────────────────────────────────────────

fn valid_deadline_offset_strategy() -> impl Strategy<Value = u64> {
    DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX
}

fn valid_goal_strategy() -> impl Strategy<Value = i128> {
    GOAL_MIN..=GOAL_MAX
}

#[allow(dead_code)]
fn valid_min_contribution_strategy(goal: i128) -> impl Strategy<Value = i128> {
    MIN_CONTRIBUTION_FLOOR..=goal
}

// ── Property tests ───────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Deadline offset in valid range is always accepted.
    #[test]
    fn prop_valid_deadline_offset_accepted(offset in valid_deadline_offset_strategy()) {
        prop_assert!(is_valid_deadline_offset(offset));
    }

    /// Goal in valid range is always accepted.
    #[test]
    fn prop_valid_goal_accepted(goal in valid_goal_strategy()) {
        prop_assert!(is_valid_goal(goal));
    }

    /// Deadline offset below DEADLINE_OFFSET_MIN is rejected (typo fix: was 100).
    #[test]
    fn prop_deadline_offset_below_min_rejected(offset in 0u64..DEADLINE_OFFSET_MIN) {
        prop_assert!(!is_valid_deadline_offset(offset));
    }

    /// Deadline offset above max is rejected.
    #[test]
    fn prop_deadline_offset_above_max_rejected(
        offset in (DEADLINE_OFFSET_MAX + 1)..=(DEADLINE_OFFSET_MAX + 100_000),
    ) {
        prop_assert!(!is_valid_deadline_offset(offset));
    }

    /// Goal below GOAL_MIN is rejected.
    #[test]
    fn prop_goal_below_min_rejected(goal in (-1_000_000i128..GOAL_MIN)) {
        prop_assert!(!is_valid_goal(goal));
    }

    /// Goal above GOAL_MAX is rejected.
    #[test]
    fn prop_goal_above_max_rejected(goal in (GOAL_MAX + 1)..=(GOAL_MAX + 1_000_000)) {
        prop_assert!(!is_valid_goal(goal));
    }

    /// Min contribution in [1, goal] is valid for that goal.
    #[test]
    fn prop_min_contribution_valid_for_goal(
        (goal, min) in (GOAL_MIN..=GOAL_MAX)
            .prop_flat_map(|g| (Just(g), MIN_CONTRIBUTION_FLOOR..=g)),
    ) {
        prop_assert!(is_valid_min_contribution(min, goal));
    }

    /// Contribution >= min_contribution is valid.
    #[test]
    fn prop_contribution_at_or_above_min_valid(
        (min_contribution, amount) in (MIN_CONTRIBUTION_FLOOR..=1_000_000i128)
            .prop_flat_map(|m| (Just(m), m..=(m + 10_000_000))),
    ) {
        prop_assert!(is_valid_contribution_amount(amount, min_contribution));
    }

    /// Clamp progress bps never exceeds PROGRESS_BPS_CAP.
    #[test]
    fn prop_clamp_progress_bps_capped(raw in -1000i128..=20000i128) {
        let clamped = clamp_progress_bps(raw);
        prop_assert!(clamped <= PROGRESS_BPS_CAP);
    }

    /// Clamp progress bps does not panic for extreme inputs.
    #[test]
    fn prop_clamp_progress_bps_no_panic(raw in -100_000i128..=100_000i128) {
        let _ = clamp_progress_bps(raw);
    }
}

// ── Unit tests for edge cases ────────────────────────────────────────────────

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn boundary_100_rejected_typo_fix() {
        assert!(!is_valid_deadline_offset(100));
    }

    #[test]
    fn boundary_1000_accepted() {
        assert!(is_valid_deadline_offset(1000));
    }

    #[test]
    fn goal_zero_rejected() {
        assert!(!is_valid_goal(0));
    }

    #[test]
    fn goal_negative_rejected() {
        assert!(!is_valid_goal(-1));
    }

    #[test]
    fn fee_bps_cap_is_10000() {
        assert_eq!(FEE_BPS_CAP, 10_000);
    }

    #[test]
    fn progress_bps_cap_is_10000() {
        assert_eq!(PROGRESS_BPS_CAP, 10_000);
    }

    #[test]
    fn regression_seed_goal_1m_valid() {
        assert!(is_valid_goal(1_000_000));
    }

    #[test]
    fn regression_seed_goal_2m_valid() {
        assert!(is_valid_goal(2_000_000));
    }

    #[test]
    fn contribution_100k_valid_when_min_lower() {
        assert!(is_valid_contribution_amount(100_000, 1_000));
    }
}
