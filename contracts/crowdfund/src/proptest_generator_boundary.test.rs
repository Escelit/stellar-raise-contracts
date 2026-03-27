//! Comprehensive tests for the ProptestGeneratorBoundary contract.
//!
//! @title   ProptestGeneratorBoundary Tests
//! @notice  Validates correct return of boundary constants and logic for clamping/validation.
//! @dev     Includes both unit tests and property-based tests for boundary safety.
//!
//! ## Test Coverage
//!
//! - **Constant Sanity Checks**: Verify all constants return correct values.
//! - **Validation Functions**: Unit tests for each is_valid_* function.
//! - **Clamping Functions**: Unit tests for clamp_* functions.
//! - **Derived Calculations**: Unit tests for compute_* functions.
//! - **Property-Based Tests**: Proptest with 64+ cases per property.
//! - **Edge Cases**: Boundary values, overflow scenarios, zero/negative inputs.
//! - **Regression Seeds**: Known problematic values from CI failures.
//!
//! Target: ≥95% line coverage.

#[cfg(test)]
mod tests {
    use crate::proptest_generator_boundary::{
        ProptestGeneratorBoundary, ProptestGeneratorBoundaryClient, DEADLINE_OFFSET_MAX,
        DEADLINE_OFFSET_MIN, FEE_BPS_CAP, GENERATOR_BATCH_MAX, GOAL_MAX, GOAL_MIN,
        MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP, PROPTEST_CASES_MAX, PROPTEST_CASES_MIN,
    };
    use proptest::prelude::*;
    use soroban_sdk::{Env, Symbol};

    // ── Setup Helper ──────────────────────────────────────────────────────────

    /// Setup a fresh test environment with the boundary contract registered.
    fn setup() -> (Env, ProptestGeneratorBoundaryClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(ProptestGeneratorBoundary, ());
        let client = ProptestGeneratorBoundaryClient::new(&env, &contract_id);
        (env, client)
    }

    // ── Constant Sanity Checks ────────────────────────────────────────────────

    #[test]
    fn test_constants_return_correct_values() {
        let (_env, client) = setup();
        assert_eq!(client.deadline_offset_min(), DEADLINE_OFFSET_MIN);
        assert_eq!(client.deadline_offset_max(), DEADLINE_OFFSET_MAX);
        assert_eq!(client.goal_min(), GOAL_MIN);
        assert_eq!(client.goal_max(), GOAL_MAX);
        assert_eq!(client.min_contribution_floor(), MIN_CONTRIBUTION_FLOOR);
        assert_eq!(client.progress_bps_cap(), PROGRESS_BPS_CAP);
        assert_eq!(client.fee_bps_cap(), FEE_BPS_CAP);
        assert_eq!(client.proptest_cases_min(), PROPTEST_CASES_MIN);
        assert_eq!(client.proptest_cases_max(), PROPTEST_CASES_MAX);
        assert_eq!(client.generator_batch_max(), GENERATOR_BATCH_MAX);
    }

    #[test]
    fn test_constants_are_ordered_correctly() {
        assert!(DEADLINE_OFFSET_MIN < DEADLINE_OFFSET_MAX);
        assert!(GOAL_MIN < GOAL_MAX);
        assert!(PROPTEST_CASES_MIN < PROPTEST_CASES_MAX);
        assert!(PROGRESS_BPS_CAP > 0);
        assert!(FEE_BPS_CAP > 0);
        assert!(GENERATOR_BATCH_MAX > 0);
    }

    // ── Deadline Offset Validation ────────────────────────────────────────────

    #[test]
    fn test_is_valid_deadline_offset_boundary_values() {
        let (_env, client) = setup();
        // Lower boundary
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MIN));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MIN - 1)));
        // Upper boundary
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MAX));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MAX + 1)));
        // Mid-range
        assert!(client.is_valid_deadline_offset(&500_000));
    }

    #[test]
    fn test_is_valid_deadline_offset_edge_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_deadline_offset(&0));
        assert!(!client.is_valid_deadline_offset(&999));
        assert!(!client.is_valid_deadline_offset(&u64::MAX));
    }

    // ── Goal Validation ──────────────────────────────────────────────────────

    #[test]
    fn test_is_valid_goal_boundary_values() {
        let (_env, client) = setup();
        // Lower boundary
        assert!(client.is_valid_goal(&GOAL_MIN));
        assert!(!client.is_valid_goal(&(GOAL_MIN - 1)));
        // Upper boundary
        assert!(client.is_valid_goal(&GOAL_MAX));
        assert!(!client.is_valid_goal(&(GOAL_MAX + 1)));
        // Mid-range
        assert!(client.is_valid_goal(&50_000_000));
    }

    #[test]
    fn test_is_valid_goal_edge_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_goal(&0));
        assert!(!client.is_valid_goal(&-1));
        assert!(!client.is_valid_goal(&999));
        assert!(!client.is_valid_goal(&i128::MIN));
    }

    // ── Minimum Contribution Validation ───────────────────────────────────────

    #[test]
    fn test_is_valid_min_contribution() {
        let (_env, client) = setup();
        let goal = 1_000_000;
        // Valid cases
        assert!(client.is_valid_min_contribution(&MIN_CONTRIBUTION_FLOOR, &goal));
        assert!(client.is_valid_min_contribution(&500_000, &goal));
        assert!(client.is_valid_min_contribution(&goal, &goal));
        // Invalid cases
        assert!(!client.is_valid_min_contribution(&0, &goal));
        assert!(!client.is_valid_min_contribution(&(goal + 1), &goal));
        assert!(!client.is_valid_min_contribution(&-1, &goal));
    }

    #[test]
    fn test_is_valid_min_contribution_with_min_goal() {
        let (_env, client) = setup();
        assert!(client.is_valid_min_contribution(&MIN_CONTRIBUTION_FLOOR, &GOAL_MIN));
        assert!(!client.is_valid_min_contribution(&(GOAL_MIN + 1), &GOAL_MIN));
    }

    // ── Contribution Amount Validation ────────────────────────────────────────

    #[test]
    fn test_is_valid_contribution_amount() {
        let (_env, client) = setup();
        let min_contribution = 1_000;
        // Valid cases
        assert!(client.is_valid_contribution_amount(&min_contribution, &min_contribution));
        assert!(client.is_valid_contribution_amount(&(min_contribution + 1), &min_contribution));
        assert!(client.is_valid_contribution_amount(&1_000_000, &min_contribution));
        // Invalid cases
        assert!(!client.is_valid_contribution_amount(&(min_contribution - 1), &min_contribution));
        assert!(!client.is_valid_contribution_amount(&0, &min_contribution));
        assert!(!client.is_valid_contribution_amount(&-1, &min_contribution));
    }

    // ── Fee Basis Points Validation ───────────────────────────────────────────

    #[test]
    fn test_is_valid_fee_bps() {
        let (_env, client) = setup();
        // Valid cases
        assert!(client.is_valid_fee_bps(&0));
        assert!(client.is_valid_fee_bps(&5_000));
        assert!(client.is_valid_fee_bps(&FEE_BPS_CAP));
        // Invalid cases
        assert!(!client.is_valid_fee_bps(&(FEE_BPS_CAP + 1)));
        assert!(!client.is_valid_fee_bps(&u32::MAX));
    }

    // ── Generator Batch Size Validation ───────────────────────────────────────

    #[test]
    fn test_is_valid_generator_batch_size() {
        let (_env, client) = setup();
        // Valid cases
        assert!(client.is_valid_generator_batch_size(&1));
        assert!(client.is_valid_generator_batch_size(&256));
        assert!(client.is_valid_generator_batch_size(&GENERATOR_BATCH_MAX));
        // Invalid cases
        assert!(!client.is_valid_generator_batch_size(&0));
        assert!(!client.is_valid_generator_batch_size(&(GENERATOR_BATCH_MAX + 1)));
    }

    // ── Clamping Functions ────────────────────────────────────────────────────

    #[test]
    fn test_is_valid_fee_bps_invalid_cases() {
        let (_env, client) = setup();
        // Below minimum
        assert_eq!(client.clamp_proptest_cases(&0), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&1), PROPTEST_CASES_MIN);
        // Within range
        assert_eq!(client.clamp_proptest_cases(&64), 64);
        assert_eq!(client.clamp_proptest_cases(&128), 128);
        // Above maximum
        assert_eq!(client.clamp_proptest_cases(&1000), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&u32::MAX), PROPTEST_CASES_MAX);
    }

    #[test]
    fn test_clamp_progress_bps() {
        let (_env, client) = setup();
        // Negative values
        assert_eq!(client.clamp_progress_bps(&-1000), 0);
        assert_eq!(client.clamp_progress_bps(&-1), 0);
        // Zero
        assert_eq!(client.clamp_progress_bps(&0), 0);
        // Within range
        assert_eq!(client.clamp_progress_bps(&5000), 5000);
        assert_eq!(client.clamp_progress_bps(&10000), PROGRESS_BPS_CAP);
        // Above cap
        assert_eq!(client.clamp_progress_bps(&10001), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&i128::MAX), PROGRESS_BPS_CAP);
    }

    // ── Derived Calculation Functions ─────────────────────────────────────────

    #[test]
    fn test_compute_progress_bps_basic() {
        let (_env, client) = setup();
        // 50% funded
        assert_eq!(client.compute_progress_bps(&500, &1000), 5000);
        // 100% funded
        assert_eq!(client.compute_progress_bps(&1000, &1000), 10000);
        // 200% funded (capped)
        assert_eq!(client.compute_progress_bps(&2000, &1000), 10000);
    }

    #[test]
    fn test_compute_progress_bps_edge_cases() {
        let (_env, client) = setup();
        // Zero goal
        assert_eq!(client.compute_progress_bps(&500, &0), 0);
        // Negative goal
        assert_eq!(client.compute_progress_bps(&500, &-1000), 0);
        // Negative raised
        assert_eq!(client.compute_progress_bps(&-100, &1000), 0);
        // Very small amounts
        assert_eq!(client.compute_progress_bps(&1, &10000), 1);
    }

    #[test]
    fn test_compute_progress_bps_overflow_safety() {
        let (_env, client) = setup();
        // Large values that could overflow without saturating_mul
        let large_raised = i128::MAX / 2;
        let large_goal = 1_000;
        let result = client.compute_progress_bps(&large_raised, &large_goal);
        assert_eq!(result, PROGRESS_BPS_CAP);
    }

    #[test]
    fn test_compute_fee_amount_basic() {
        let (_env, client) = setup();
        // 10% fee
        assert_eq!(client.compute_fee_amount(&1000, &1000), 100);
        // 50% fee
        assert_eq!(client.compute_fee_amount(&1000, &5000), 500);
        // 100% fee
        assert_eq!(client.compute_fee_amount(&1000, &10000), 1000);
    }

    #[test]
    fn test_compute_fee_amount_edge_cases() {
        let (_env, client) = setup();
        // Zero amount
        assert_eq!(client.compute_fee_amount(&0, &5000), 0);
        // Negative amount
        assert_eq!(client.compute_fee_amount(&-1000, &5000), 0);
        // Zero fee
        assert_eq!(client.compute_fee_amount(&1000, &0), 0);
        // Both zero
        assert_eq!(client.compute_fee_amount(&0, &0), 0);
    }

    #[test]
    fn test_compute_fee_amount_floor_division() {
        let (_env, client) = setup();
        // 1/3 fee (should floor)
        assert_eq!(client.compute_fee_amount(&1000, &3333), 333);
        // 2/3 fee (should floor)
        assert_eq!(client.compute_fee_amount(&1000, &6666), 666);
    }

    #[test]
    fn test_compute_progress_bps_negative_raised() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&-1_000, &1_000), 0);
        assert_eq!(client.compute_progress_bps(&-100_000_000, &1_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_partial_progress() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&500, &1_000), 5_000);
        assert_eq!(client.compute_progress_bps(&250, &1_000), 2_500);
        assert_eq!(client.compute_progress_bps(&1, &1_000), 10);
    }

    #[test]
    fn test_compute_progress_bps_full_progress() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &1_000), 10_000);
        assert_eq!(client.compute_progress_bps(&100_000_000, &100_000_000), 10_000);
    }

    #[test]
    fn test_compute_progress_bps_over_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&2_000, &1_000), 10_000);
        assert_eq!(client.compute_progress_bps(&200_000_000, &100_000_000), 10_000);
    }

    // ── compute_fee_amount Tests ─────────────────────────────────────────────

    #[test]
    fn test_compute_fee_amount_zero_amount() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&0, &1_000), 0);
        assert_eq!(client.compute_fee_amount(&0, &10_000), 0);
    }

    #[test]
    fn test_compute_fee_amount_negative_amount() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&-1_000, &1_000), 0);
        assert_eq!(client.compute_fee_amount(&-100_000_000, &5_000), 0);
    }

    #[test]
    fn test_compute_fee_amount_zero_fee() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &0), 0);
        assert_eq!(client.compute_fee_amount(&100_000_000, &0), 0);
    }

    #[test]
    fn test_compute_fee_amount_valid_calculations() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_000, &5_000), 500);
        assert_eq!(client.compute_fee_amount(&1_000, &10_000), 1_000);
        assert_eq!(client.compute_fee_amount(&10_000, &1_000), 1_000);
    }

    #[test]
    fn test_compute_fee_amount_large_values() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&100_000_000, &1_000), 10_000_000);
        assert_eq!(client.compute_fee_amount(&100_000_000, &5_000), 50_000_000);
    }

    // ── log_tag Tests ────────────────────────────────────────────────────────

    #[test]
    fn test_log_tag() {
        let (env, client) = setup();
        assert_eq!(client.log_tag(), Symbol::new(&env, "boundary"));
    }

    // ── Property-Based Tests ──────────────────────────────────────────────────
    // @notice These tests use proptest to explore the input space systematically.
    //         Each property is tested with 64+ randomly generated cases.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Property: All valid deadline offsets pass validation.
        #[test]
        fn prop_deadline_offset_validity(offset in DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_deadline_offset(&offset));
        }

        /// Property: All invalid deadline offsets fail validation.
        #[test]
        fn prop_deadline_offset_invalidity(offset in 0u64..DEADLINE_OFFSET_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_deadline_offset(&offset));
        }

        /// Property: All valid goals pass validation.
        #[test]
        fn prop_deadline_offset_below_min_invalid(offset in 0u64..DEADLINE_OFFSET_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_deadline_offset(&offset));
        }

        #[test]
        fn prop_deadline_offset_above_max_invalid(offset in (DEADLINE_OFFSET_MAX + 1)..u64::MAX) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_deadline_offset(&offset));
        }

        #[test]
        fn prop_goal_validity(goal in GOAL_MIN..=GOAL_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_goal(&goal));
        }

        /// Property: All invalid goals fail validation.
        #[test]
        fn prop_goal_below_min_invalid(goal in i128::MIN..GOAL_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_goal(&goal));
        }

        #[test]
        fn prop_goal_above_max_invalid(goal in (GOAL_MAX + 1)..i128::MAX) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_goal(&goal));
        }

        #[test]
        fn prop_progress_bps_always_bounded(
            raised in -1_000_000_000i128..=1_000_000_000i128,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let (_env, client) = setup();
            let bps = client.compute_progress_bps(&raised, &goal);
            prop_assert!(bps <= PROGRESS_BPS_CAP);
        }

        #[test]
        fn prop_progress_bps_zero_when_goal_zero(raised in -1_000_000i128..=1_000_000i128) {
            let (_env, client) = setup();
            let bps = client.compute_progress_bps(&raised, &0);
            prop_assert_eq!(bps, 0);
        }

        #[test]
        fn prop_progress_bps_zero_when_raised_negative(goal in GOAL_MIN..=GOAL_MAX) {
            let (_env, client) = setup();
            let bps = client.compute_progress_bps(&-1000, &goal);
            prop_assert_eq!(bps, 0);
        }

        #[test]
        fn prop_fee_amount_always_non_negative(
            amount in -1_000_000i128..=1_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&amount, &fee_bps);
            prop_assert!(fee >= 0);
        }

        #[test]
        fn prop_fee_amount_zero_when_amount_zero(fee_bps in 0u32..=FEE_BPS_CAP) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&0, &fee_bps);
            prop_assert_eq!(fee, 0);
        }

        #[test]
        fn prop_fee_amount_zero_when_fee_zero(amount in -1_000_000i128..=1_000_000i128) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&amount, &0);
            prop_assert_eq!(fee, 0);
        }

        #[test]
        fn prop_clamp_proptest_cases_within_bounds(requested in 0u32..=u32::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_proptest_cases(&requested);
            prop_assert!(clamped >= PROPTEST_CASES_MIN);
            prop_assert!(clamped <= PROPTEST_CASES_MAX);
        }

        #[test]
        fn prop_clamp_progress_bps_within_bounds(raw in i128::MIN..=i128::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_progress_bps(&raw);
            prop_assert!(clamped <= PROGRESS_BPS_CAP);
        }

        #[test]
        fn prop_min_contribution_valid_when_in_range(
            min_contrib in MIN_CONTRIBUTION_FLOOR..=GOAL_MAX,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let (_env, client) = setup();
            if min_contrib <= goal {
                prop_assert!(client.is_valid_min_contribution(&min_contrib, &goal));
            }
        }

        #[test]
        fn prop_contribution_amount_valid_when_meets_minimum(
            amount in MIN_CONTRIBUTION_FLOOR..=1_000_000i128,
            min_contrib in MIN_CONTRIBUTION_FLOOR..=1_000_000i128
        ) {
            let (_env, client) = setup();
            if amount >= min_contrib {
                prop_assert!(client.is_valid_contribution_amount(&amount, &min_contrib));
            }
        }

        #[test]
        fn prop_fee_bps_valid_when_within_cap(fee_bps in 0u32..=FEE_BPS_CAP) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_fee_bps(&fee_bps));
        }

        #[test]
        fn prop_batch_size_valid_when_in_range(batch_size in 1u32..=GENERATOR_BATCH_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_generator_batch_size(&batch_size));
        }
    }

    // ── Regression Tests ──────────────────────────────────────────────────────
    // @notice These tests capture known problematic values from CI failures.

    #[test]
    fn regression_deadline_offset_100_seconds_now_invalid() {
        let (_env, client) = setup();
        // Previously accepted (caused flaky tests), now rejected
        assert!(!client.is_valid_deadline_offset(&100));
    }

    #[test]
    fn regression_goal_zero_always_invalid() {
        let (_env, client) = setup();
        assert!(!client.is_valid_goal(&0));
    }

    #[test]
    fn regression_progress_bps_never_exceeds_cap() {
        let (_env, client) = setup();
        // Even with extreme values, should cap at 10,000
        assert_eq!(
            client.compute_progress_bps(&i128::MAX, &1),
            PROGRESS_BPS_CAP
        );
    }

    #[test]
    fn regression_fee_amount_never_negative() {
        let (_env, client) = setup();
        // Even with negative inputs, should return 0 or positive
        assert!(client.compute_fee_amount(&-1_000_000, &5000) >= 0);
    }
}
