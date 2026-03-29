//! # ProptestGeneratorBoundary — Standalone Property Tests
//!
//! @title   ProptestGeneratorBoundary Standalone Tests
//! @notice  Property-based and unit tests for boundary validators using pure
//!          functions (no Soroban `Env` required). Suitable for fast CI runs.
//! @dev     Complements `proptest_generator_boundary.test.rs` which exercises
//!          the on-chain contract client interface.
//!
//! ## New Edge Cases (Issue #423)
//!
//! - `is_ui_displayable_progress` — progress bar safety
//! - `compute_display_percent`    — percentage label correctness
//! - `is_contribution_ui_safe`    — token-decimal overflow guard
//! - `deadline_ui_state`          — three-state UI classification
//! - `compute_net_payout`         — creator payout after fee
//!
//! ## Security Notes
//!
//! - Deadline offset 100 (old buggy minimum) must be rejected.
//! - `goal == 0` must never reach division logic.
//! - `progress_bps` must never exceed `PROGRESS_BPS_CAP` (10 000).
//! Comprehensive tests for proptest generator boundary conditions.
//!
//! @title   ProptestGeneratorBoundary Standalone Tests
//! @notice  Property-based and unit tests for boundary validators using pure
//!          functions (no Soroban `Env` required). Suitable for fast CI runs.
//! @dev     These tests complement `proptest_generator_boundary.test.rs` which
//!          exercises the on-chain contract client interface.
//!
//! ## Security Notes
//!
//! - Deadline offset 100 (old buggy minimum) must be rejected.
//! - `goal == 0` must never reach division logic.
//! - `progress_bps` must never exceed `PROGRESS_BPS_CAP` (10 000).
//! - `compute_net_payout` must return `None` for `fee_bps > FEE_BPS_CAP`.
//! - `is_contribution_ui_safe` must reject `token_decimals > MAX_TOKEN_DECIMALS`.

use proptest::prelude::*;
use proptest::strategy::Just;

use crate::proptest_generator_boundary::{
    clamp_progress_bps, compute_display_percent, compute_net_payout, compute_progress_bps,
    deadline_ui_state, is_contribution_ui_safe, is_ui_displayable_progress,
    is_valid_contribution_amount, is_valid_deadline_offset, is_valid_goal,
    is_valid_min_contribution, DeadlineUiState, DEADLINE_ENDING_SOON_THRESHOLD,
    DEADLINE_OFFSET_MAX, DEADLINE_OFFSET_MIN, FEE_BPS_CAP, GOAL_MAX, GOAL_MIN,
    MAX_TOKEN_DECIMALS, MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP,
};

// ── Strategy Definitions ──────────────────────────────────────────────────────
    clamp_progress_bps, is_valid_contribution_amount, is_valid_deadline_offset,
    is_valid_goal, is_valid_min_contribution, DEADLINE_OFFSET_MAX, DEADLINE_OFFSET_MIN,
    FEE_BPS_CAP, GOAL_MAX, GOAL_MIN, MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP,
};

// ── Strategy definitions ─────────────────────────────────────────────────────
// ── Strategy Helpers ──────────────────────────────────────────────────────────

fn valid_deadline_offset() -> impl Strategy<Value = u64> {
    DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX
}

fn valid_goal() -> impl Strategy<Value = i128> {
    GOAL_MIN..=GOAL_MAX
}

fn valid_fee_bps() -> impl Strategy<Value = u32> {
    0u32..=FEE_BPS_CAP
}

fn valid_token_decimals() -> impl Strategy<Value = u32> {
    0u32..=MAX_TOKEN_DECIMALS
}

// ── Property Tests ────────────────────────────────────────────────────────────
#[allow(dead_code)]
fn valid_min_contribution_strategy(goal: i128) -> impl Strategy<Value = i128> {
    MIN_CONTRIBUTION_FLOOR..=goal
}

// ── Property tests ───────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ── Existing boundary properties ──────────────────────────────────────────

    #[test]
    fn prop_valid_deadline_offset_accepted(offset in valid_deadline_offset()) {
        prop_assert!(is_valid_deadline_offset(offset));
    }

    #[test]
    fn prop_valid_goal_accepted(goal in valid_goal()) {
        prop_assert!(is_valid_goal(goal));
    }

    /// Deadline offset below DEADLINE_OFFSET_MIN is rejected.
    /// Deadline offset below DEADLINE_OFFSET_MIN is rejected (typo fix: was 100).
    #[test]
    fn prop_deadline_offset_below_min_rejected(offset in 0u64..DEADLINE_OFFSET_MIN) {
        prop_assert!(!is_valid_deadline_offset(offset));
    }

    #[test]
    fn prop_deadline_offset_above_max_rejected(
        offset in (DEADLINE_OFFSET_MAX + 1)..=(DEADLINE_OFFSET_MAX + 100_000),
    ) {
        prop_assert!(!is_valid_deadline_offset(offset));
    }

    #[test]
    fn prop_goal_below_min_rejected(goal in (-1_000_000i128..GOAL_MIN)) {
        prop_assert!(!is_valid_goal(goal));
    }

    #[test]
    fn prop_goal_above_max_rejected(goal in (GOAL_MAX + 1)..=(GOAL_MAX + 1_000_000)) {
        prop_assert!(!is_valid_goal(goal));
    }

    #[test]
    fn prop_min_contribution_valid_for_goal(
        (goal, min) in valid_goal()
            .prop_flat_map(|g| (Just(g), MIN_CONTRIBUTION_FLOOR..=g)),
    ) {
        prop_assert!(is_valid_min_contribution(min, goal));
    }

    #[test]
    fn prop_contribution_at_or_above_min_valid(
        (min_contribution, amount) in (MIN_CONTRIBUTION_FLOOR..=1_000_000i128)
            .prop_flat_map(|m| (Just(m), m..=(m + 10_000_000))),
    ) {
        prop_assert!(is_valid_contribution_amount(amount, min_contribution));
    }

    #[test]
    fn prop_clamp_progress_bps_capped(raw in -1_000i128..=20_000i128) {
    fn prop_clamp_progress_bps_capped(raw in -1000i128..=20000i128) {
        let clamped = clamp_progress_bps(raw);
        prop_assert!(clamped <= PROGRESS_BPS_CAP);
        prop_assert!(clamp_progress_bps(raw) <= PROGRESS_BPS_CAP);
    }

    #[test]
    fn prop_clamp_progress_bps_no_panic(raw in -100_000i128..=100_000i128) {
        let _ = clamp_progress_bps(raw);
    }

    #[test]
    fn prop_compute_progress_bps_bounded(
        raised in -1_000_000i128..=1_000_000i128,
        goal in GOAL_MIN..=GOAL_MAX,
    ) {
        prop_assert!(compute_progress_bps(raised, goal) <= PROGRESS_BPS_CAP);
    }

    // ── New properties: is_ui_displayable_progress (Issue #423) ──────────────

    /// Property: is_ui_displayable_progress is true iff bps <= PROGRESS_BPS_CAP.
    #[test]
    fn prop_ui_displayable_iff_within_cap(bps in 0u32..=u32::MAX) {
        prop_assert_eq!(is_ui_displayable_progress(bps), bps <= PROGRESS_BPS_CAP);
    }

    /// Property: clamped progress is always UI-displayable.
    #[test]
    fn prop_clamped_progress_always_displayable(raw in i128::MIN..=i128::MAX) {
        let clamped = clamp_progress_bps(raw);
        prop_assert!(is_ui_displayable_progress(clamped));
    }

    // ── New properties: compute_display_percent (Issue #423) ─────────────────

    /// Property: display percent never exceeds PROGRESS_BPS_CAP.
    #[test]
    fn prop_display_percent_bounded(bps in 0u32..=u32::MAX) {
        prop_assert!(compute_display_percent(bps) <= PROGRESS_BPS_CAP);
    }

    /// Property: display percent equals bps when bps is within cap.
    #[test]
    fn prop_display_percent_identity_within_cap(bps in 0u32..=PROGRESS_BPS_CAP) {
        prop_assert_eq!(compute_display_percent(bps), bps);
    }

    /// Property: display percent equals cap when bps exceeds cap.
    #[test]
    fn prop_display_percent_capped_above_cap(bps in (PROGRESS_BPS_CAP + 1)..=u32::MAX) {
        prop_assert_eq!(compute_display_percent(bps), PROGRESS_BPS_CAP);
    }

    // ── New properties: compute_net_payout (Issue #423) ──────────────────────

    /// Property: compute_net_payout returns None for fee_bps > FEE_BPS_CAP.
    #[test]
    fn prop_net_payout_none_for_invalid_fee(
        total in 0i128..=100_000_000i128,
        fee_bps in (FEE_BPS_CAP + 1)..=u32::MAX,
    ) {
        prop_assert_eq!(compute_net_payout(total, fee_bps), None);
    }

    /// Property: compute_net_payout returns Some for valid fee_bps.
    #[test]
    fn prop_net_payout_some_for_valid_fee(
        total in 0i128..=100_000_000i128,
        fee_bps in valid_fee_bps(),
    ) {
        prop_assert!(compute_net_payout(total, fee_bps).is_some());
    }

    /// Property: net payout is always <= total and >= 0 for valid inputs.
    #[test]
    fn prop_net_payout_in_valid_range(
        total in 0i128..=100_000_000i128,
        fee_bps in valid_fee_bps(),
    ) {
        if let Some(net) = compute_net_payout(total, fee_bps) {
            prop_assert!(net >= 0);
            prop_assert!(net <= total);
        }
    }

    /// Property: zero fee means net payout equals total.
    #[test]
    fn prop_net_payout_zero_fee_equals_total(total in 0i128..=100_000_000i128) {
        prop_assert_eq!(compute_net_payout(total, 0), Some(total));
    }

    /// Property: 100 % fee means net payout is zero.
    #[test]
    fn prop_net_payout_full_fee_is_zero(total in 0i128..=100_000_000i128) {
        prop_assert_eq!(compute_net_payout(total, FEE_BPS_CAP), Some(0));
    }

    // ── New properties: deadline_ui_state (Issue #423) ────────────────────────

    /// Property: seconds == 0 is always Expired.
    #[test]
    fn prop_deadline_zero_is_expired(_x in 0u32..1u32) {
        prop_assert_eq!(deadline_ui_state(0), DeadlineUiState::Expired);
    }

    /// Property: positive seconds is never Expired.
    #[test]
    fn prop_deadline_positive_not_expired(secs in 1u64..=u64::MAX) {
        prop_assert_ne!(deadline_ui_state(secs), DeadlineUiState::Expired);
    }

    /// Property: seconds in (0, DEADLINE_ENDING_SOON_THRESHOLD] is EndingSoon.
    #[test]
    fn prop_deadline_ending_soon_range(secs in 1u64..=DEADLINE_ENDING_SOON_THRESHOLD) {
        prop_assert_eq!(deadline_ui_state(secs), DeadlineUiState::EndingSoon);
    }

    /// Property: seconds > DEADLINE_ENDING_SOON_THRESHOLD is Active.
    #[test]
    fn prop_deadline_active_range(secs in (DEADLINE_ENDING_SOON_THRESHOLD + 1)..=u64::MAX) {
        prop_assert_eq!(deadline_ui_state(secs), DeadlineUiState::Active);
    }

    // ── New properties: is_contribution_ui_safe (Issue #423) ─────────────────

    /// Property: valid amount + valid decimals is UI-safe.
    #[test]
    fn prop_contribution_ui_safe_valid_inputs(
        amount in MIN_CONTRIBUTION_FLOOR..=1_000_000i128,
        decimals in valid_token_decimals(),
    ) {
        // Only assert safe when no overflow — small amounts with small decimals
        if amount.checked_mul(10i128.pow(decimals)).is_some() {
            prop_assert!(is_contribution_ui_safe(amount, MIN_CONTRIBUTION_FLOOR, decimals));
        }
    }

    /// Property: excessive decimals always rejected.
    #[test]
    fn prop_contribution_ui_safe_rejects_excess_decimals(
        amount in MIN_CONTRIBUTION_FLOOR..=GOAL_MAX,
        decimals in (MAX_TOKEN_DECIMALS + 1)..=255u32,
    ) {
        prop_assert!(!is_contribution_ui_safe(amount, MIN_CONTRIBUTION_FLOOR, decimals));
    }

    /// Property: below-minimum amount always rejected regardless of decimals.
    #[test]
    fn prop_contribution_ui_safe_rejects_below_min(
        amount in i128::MIN..MIN_CONTRIBUTION_FLOOR,
        decimals in valid_token_decimals(),
    ) {
        prop_assert!(!is_contribution_ui_safe(amount, MIN_CONTRIBUTION_FLOOR, decimals));
    }
}

// ── Unit Tests for Edge Cases ─────────────────────────────────────────────────
// ── Unit tests for edge cases ────────────────────────────────────────────────

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    // ── Existing edge cases ───────────────────────────────────────────────────

    /// @security Old buggy minimum of 100 must be rejected after the fix.
    #[test]
    fn boundary_100_rejected_typo_fix() {
        assert!(!is_valid_deadline_offset(100));
    }

    #[test]
    fn boundary_1000_accepted() {
        assert!(is_valid_deadline_offset(1_000));
    }

    /// @security goal == 0 must be rejected to prevent division-by-zero.
        assert!(is_valid_deadline_offset(1000));
    }

    /// @security goal == 0 must be rejected to prevent division-by-zero.
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
    fn contribution_100k_valid_when_min_lower() {
        assert!(is_valid_contribution_amount(100_000, 1_000));
    }

    // ── New edge cases: is_ui_displayable_progress (Issue #423) ──────────────

    #[test]
    fn ui_displayable_zero_is_valid() {
        assert!(is_ui_displayable_progress(0));
    }

    #[test]
    fn ui_displayable_cap_is_valid() {
        assert!(is_ui_displayable_progress(PROGRESS_BPS_CAP));
    }

    #[test]
    fn ui_displayable_above_cap_invalid() {
        assert!(!is_ui_displayable_progress(PROGRESS_BPS_CAP + 1));
        assert!(!is_ui_displayable_progress(u32::MAX));
    }

    // ── New edge cases: compute_display_percent (Issue #423) ─────────────────

    #[test]
    fn display_percent_zero() {
        assert_eq!(compute_display_percent(0), 0);
    }

    #[test]
    fn display_percent_half() {
        assert_eq!(compute_display_percent(5_000), 5_000);
    }

    #[test]
    fn display_percent_full() {
        assert_eq!(compute_display_percent(10_000), 10_000);
    }

    #[test]
    fn display_percent_over_cap_clamped() {
        assert_eq!(compute_display_percent(10_001), PROGRESS_BPS_CAP);
        assert_eq!(compute_display_percent(u32::MAX), PROGRESS_BPS_CAP);
    }

    // ── New edge cases: deadline_ui_state (Issue #423) ────────────────────────

    #[test]
    fn deadline_zero_is_expired() {
        assert_eq!(deadline_ui_state(0), DeadlineUiState::Expired);
    }

    #[test]
    fn deadline_one_second_is_ending_soon() {
        assert_eq!(deadline_ui_state(1), DeadlineUiState::EndingSoon);
    }

    #[test]
    fn deadline_at_threshold_is_ending_soon() {
        assert_eq!(
            deadline_ui_state(DEADLINE_ENDING_SOON_THRESHOLD),
            DeadlineUiState::EndingSoon
        );
    }

    #[test]
    fn deadline_just_above_threshold_is_active() {
        assert_eq!(
            deadline_ui_state(DEADLINE_ENDING_SOON_THRESHOLD + 1),
            DeadlineUiState::Active
        );
    }

    #[test]
    fn deadline_large_value_is_active() {
        assert_eq!(deadline_ui_state(u64::MAX), DeadlineUiState::Active);
    }

    // ── New edge cases: compute_net_payout (Issue #423) ───────────────────────

    #[test]
    fn net_payout_zero_fee() {
        assert_eq!(compute_net_payout(1_000, 0), Some(1_000));
    }

    #[test]
    fn net_payout_full_fee() {
        assert_eq!(compute_net_payout(1_000, 10_000), Some(0));
    }

    #[test]
    fn net_payout_10_percent_fee() {
        assert_eq!(compute_net_payout(1_000, 1_000), Some(900));
    }

    #[test]
    fn net_payout_zero_total() {
        assert_eq!(compute_net_payout(0, 5_000), Some(0));
    }

    #[test]
    fn net_payout_negative_total_returns_zero_payout() {
        // negative total → Some(0) because total <= 0 guard
        assert_eq!(compute_net_payout(-1_000, 1_000), Some(0));
    }

    #[test]
    fn net_payout_invalid_fee_returns_none() {
        assert_eq!(compute_net_payout(1_000, FEE_BPS_CAP + 1), None);
        assert_eq!(compute_net_payout(1_000, u32::MAX), None);
    }

    // ── New edge cases: is_contribution_ui_safe (Issue #423) ─────────────────

    #[test]
    fn contribution_ui_safe_xlm_decimals() {
        assert!(is_contribution_ui_safe(1_000, 1_000, 7));
    }

    #[test]
    fn contribution_ui_safe_usdc_decimals() {
        assert!(is_contribution_ui_safe(1_000, 1_000, 6));
    }

    #[test]
    fn contribution_ui_safe_zero_decimals() {
        assert!(is_contribution_ui_safe(1_000, 1_000, 0));
    }

    #[test]
    fn contribution_ui_safe_max_decimals() {
        // 1 * 10^18 fits in i128
        assert!(is_contribution_ui_safe(1, MIN_CONTRIBUTION_FLOOR, MAX_TOKEN_DECIMALS));
    }

    #[test]
    fn contribution_ui_safe_excess_decimals_rejected() {
        assert!(!is_contribution_ui_safe(1_000, 1_000, MAX_TOKEN_DECIMALS + 1));
    }

    #[test]
    fn contribution_ui_safe_below_min_rejected() {
        assert!(!is_contribution_ui_safe(999, 1_000, 7));
        assert!(!is_contribution_ui_safe(0, 1_000, 7));
        assert!(!is_contribution_ui_safe(-1, 1_000, 7));
    }

    #[test]
    fn contribution_ui_safe_overflow_rejected() {
        // i128::MAX * 10^18 overflows
        assert!(!is_contribution_ui_safe(i128::MAX, MIN_CONTRIBUTION_FLOOR, 18));
    }

    /// @security Regression: overfunded campaign must cap at 100 % in UI.
    #[test]
    fn regression_overfunded_capped() {
        let bps = compute_progress_bps(200_000_000, 100_000_000);
        assert_eq!(bps, PROGRESS_BPS_CAP);
        assert!(is_ui_displayable_progress(bps));
        assert_eq!(compute_display_percent(bps), PROGRESS_BPS_CAP);
    }

    /// @security Regression: net payout with fee > cap must return None.
    #[test]
    fn regression_net_payout_invalid_fee_none() {
        assert_eq!(compute_net_payout(1_000_000, FEE_BPS_CAP + 1), None);
    }
}
