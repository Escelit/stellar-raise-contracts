//! # ProptestGeneratorBoundary — On-Chain Contract Test Suite
//!
//! @title   ProptestGeneratorBoundary Tests
//! @notice  Validates boundary constants, clamping, validation, and the new
//!          frontend UI edge cases introduced in Issue #423.
//! @dev     Uses the Soroban test environment and the generated client.
//!
//! ## Coverage
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
    fn test_clamp_proptest_cases_midpoint() {
        assert_eq!(clamp_proptest_cases(100), 100);
    }

    #[test]
    fn test_clamp_proptest_cases_at_max() {
        assert_eq!(clamp_proptest_cases(PROPTEST_CASES_MAX), PROPTEST_CASES_MAX);
    }

    #[test]
    fn test_clamp_proptest_cases_above_max() {
        assert_eq!(clamp_proptest_cases(1_000), PROPTEST_CASES_MAX);
    }

    // ── 3. On-Chain Contract Tests ────────────────────────────────────────────

    #[test]
    fn test_contract_constants_match_rust_constants() {
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
        fn prop_valid_deadline_offset_always_accepted(
            offset in DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX
        ) {
            prop_assert!(is_valid_deadline_offset(offset));
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
        fn prop_deadline_offset_below_min_always_rejected(
            offset in 0u64..DEADLINE_OFFSET_MIN
        ) {
            prop_assert!(!is_valid_deadline_offset(offset));
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
//! Comprehensive tests for proptest generator boundary conditions.
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
//!          Target coverage: ≥95% line coverage with 256 property test cases.
//! @notice  Validates boundary constants, pure helper functions, and the
//!          on-chain contract methods for correctness and security.
//! @dev     Combines unit tests, edge-case regression tests, and
//!          property-based tests (proptest) for full coverage.
//!
//! ## Test Categories
//!
//! 1. **Constant sanity** — verify exported constant values haven't drifted.
//! 2. **Pure helper unit tests** — deterministic inputs/outputs.
//! 3. **On-chain contract tests** — exercise the Soroban client interface.
//! 4. **Property-based tests** — proptest generators over valid/invalid ranges.
//! 5. **Regression seeds** — inputs that previously caused failures.
//! 6. **Frontend UX edge cases** — inputs that affect UI display correctness.
//!
//! ## Security Notes
//!
//! - Negative `raised` values must never produce a non-zero progress percentage.
//! - `goal == 0` must never cause a division-by-zero panic.
//! - Over-funded campaigns must clamp to exactly 10 000 bps (100 %).
//! - Deadline offsets below 1 000 s must be rejected to prevent timing races.
//! - Constant sanity checks
//! - Deadline offset validation (boundary values + edge cases)
//! - Goal validation (boundary values + edge cases)
//! - Min-contribution validation
//! - Contribution amount validation
//! - Fee bps validation
//! - Generator batch size validation
//! - Clamping functions
//! - Derived calculations (progress, fee, display percent, net payout)
//! - **New (Issue #423)**: UI-displayable progress, contribution UI safety,
//!   deadline UI state, display percent, net payout edge cases
//! - Property-based tests (256 cases each)
//! - Regression seeds

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use soroban_sdk::{Env, Symbol};

    use crate::proptest_generator_boundary::{
        clamp_progress_bps, compute_display_percent, compute_net_payout, compute_progress_bps,
        deadline_ui_state, is_contribution_ui_safe, is_ui_displayable_progress,
        is_valid_contribution_amount, is_valid_deadline_offset, is_valid_goal,
        is_valid_min_contribution, DeadlineUiState, ProptestGeneratorBoundary,
        ProptestGeneratorBoundaryClient, DEADLINE_ENDING_SOON_THRESHOLD, DEADLINE_OFFSET_MAX,
        DEADLINE_OFFSET_MIN, FEE_BPS_CAP, GENERATOR_BATCH_MAX, GOAL_MAX, GOAL_MIN,
        MAX_TOKEN_DECIMALS, MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP, PROPTEST_CASES_MAX,
        PROPTEST_CASES_MIN,
    };

    // ── Setup ─────────────────────────────────────────────────────────────────

    /// Setup a fresh test environment with the boundary contract registered.
        clamp_progress_bps, clamp_proptest_cases, compute_progress_bps,
        is_valid_contribution_amount, is_valid_deadline_offset, is_valid_goal,
        is_valid_min_contribution, ProptestGeneratorBoundary, ProptestGeneratorBoundaryClient,
        DEADLINE_OFFSET_MAX, DEADLINE_OFFSET_MIN, FEE_BPS_CAP, GENERATOR_BATCH_MAX, GOAL_MAX,
        GOAL_MIN, MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP, PROPTEST_CASES_MAX, PROPTEST_CASES_MIN,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Spin up a fresh Soroban test environment with the boundary contract.
    fn setup() -> (Env, ProptestGeneratorBoundaryClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(ProptestGeneratorBoundary, ());
        let client = ProptestGeneratorBoundaryClient::new(&env, &contract_id);
        (env, client)
    }

    // ── Constant Sanity ───────────────────────────────────────────────────────

    // ── 1. Constant Sanity Tests ──────────────────────────────────────────────

    /// @notice Ensures exported constants match the documented specification.
    ///         Any drift here indicates an unreviewed change to platform limits.
    #[test]
    fn test_constants_have_expected_values() {
        assert_eq!(DEADLINE_OFFSET_MIN, 1_000);
        assert_eq!(DEADLINE_OFFSET_MAX, 1_000_000);
        assert_eq!(GOAL_MIN, 1_000);
        assert_eq!(GOAL_MAX, 100_000_000);
        assert_eq!(MIN_CONTRIBUTION_FLOOR, 1);
        assert_eq!(PROGRESS_BPS_CAP, 10_000);
        assert_eq!(FEE_BPS_CAP, 10_000);
        assert_eq!(PROPTEST_CASES_MIN, 32);
        assert_eq!(PROPTEST_CASES_MAX, 256);
        assert_eq!(GENERATOR_BATCH_MAX, 512);
    }

    // ── 2. Pure Helper Unit Tests ─────────────────────────────────────────────

    // --- is_valid_deadline_offset ---

    #[test]
    fn test_deadline_offset_at_min_is_valid() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MIN));
    }

    #[test]
    fn test_deadline_offset_at_max_is_valid() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MAX));
    }

    #[test]
    fn test_deadline_offset_midpoint_is_valid() {
        assert!(is_valid_deadline_offset(500_000));
    }

    /// @security Offset of 100 was the old (buggy) minimum — must be rejected.
    #[test]
    fn test_deadline_offset_100_rejected_regression() {
        assert!(!is_valid_deadline_offset(100));
    }

    #[test]
    fn test_deadline_offset_zero_rejected() {
        assert!(!is_valid_deadline_offset(0));
    }

    #[test]
    fn test_deadline_offset_one_below_min_rejected() {
        assert!(!is_valid_deadline_offset(DEADLINE_OFFSET_MIN - 1));
    }

    #[test]
    fn test_deadline_offset_one_above_max_rejected() {
        assert!(!is_valid_deadline_offset(DEADLINE_OFFSET_MAX + 1));
    }

    // --- is_valid_goal ---

    #[test]
    fn test_goal_at_min_is_valid() {
        assert!(is_valid_goal(GOAL_MIN));
    }

    #[test]
    fn test_goal_at_max_is_valid() {
        assert!(is_valid_goal(GOAL_MAX));
    }

    #[test]
    fn test_goal_midpoint_is_valid() {
        assert!(is_valid_goal(50_000_000));
    }

    /// @security goal == 0 causes division-by-zero in progress calculation.
    #[test]
    fn test_goal_zero_rejected() {
        assert!(!is_valid_goal(0));
    }

    #[test]
    fn test_goal_negative_rejected() {
        assert!(!is_valid_goal(-1));
    }

    #[test]
    fn test_goal_one_below_min_rejected() {
        assert!(!is_valid_goal(GOAL_MIN - 1));
    }

    #[test]
    fn test_goal_one_above_max_rejected() {
        assert!(!is_valid_goal(GOAL_MAX + 1));
    }

    // --- is_valid_min_contribution ---

    #[test]
    fn test_min_contribution_floor_with_goal_min_is_valid() {
        assert!(is_valid_min_contribution(MIN_CONTRIBUTION_FLOOR, GOAL_MIN));
    }

    #[test]
    fn test_min_contribution_equal_to_goal_is_valid() {
        assert!(is_valid_min_contribution(GOAL_MIN, GOAL_MIN));
    }

    #[test]
    fn test_min_contribution_zero_rejected() {
        assert!(!is_valid_min_contribution(0, GOAL_MIN));
    }

    /// @security min_contribution > goal makes the campaign permanently un-fundable.
    #[test]
    fn test_min_contribution_above_goal_rejected() {
        assert!(!is_valid_min_contribution(GOAL_MIN + 1, GOAL_MIN));
    }

    // --- is_valid_contribution_amount ---

    #[test]
    fn test_contribution_at_min_is_valid() {
        assert!(is_valid_contribution_amount(1_000, 1_000));
    }

    #[test]
    fn test_contribution_above_min_is_valid() {
        assert!(is_valid_contribution_amount(100_000, 1_000));
    }

    #[test]
    fn test_contribution_below_min_rejected() {
        assert!(!is_valid_contribution_amount(999, 1_000));
    }

    #[test]
    fn test_contribution_zero_rejected_when_min_is_one() {
        assert!(!is_valid_contribution_amount(0, 1));
    }

    // --- clamp_progress_bps ---

    #[test]
    fn test_clamp_progress_bps_zero() {
        assert_eq!(clamp_progress_bps(0), 0);
    }

    #[test]
    fn test_clamp_progress_bps_negative_clamped_to_zero() {
        assert_eq!(clamp_progress_bps(-500), 0);
    }

    #[test]
    fn test_clamp_progress_bps_midpoint_unchanged() {
        assert_eq!(clamp_progress_bps(5_000), 5_000);
    }

    #[test]
    fn test_clamp_progress_bps_at_cap() {
        assert_eq!(clamp_progress_bps(10_000), 10_000);
    }

    #[test]
    fn test_clamp_progress_bps_above_cap_clamped() {
        assert_eq!(clamp_progress_bps(15_000), 10_000);
    }

    // --- compute_progress_bps ---

    #[test]
    fn test_compute_progress_bps_half_funded() {
        assert_eq!(compute_progress_bps(500, 1_000), 5_000);
    }

    #[test]
    fn test_compute_progress_bps_fully_funded() {
        assert_eq!(compute_progress_bps(1_000, 1_000), 10_000);
    }

    /// @security Over-funded campaigns must cap at 100 %, not overflow.
    #[test]
    fn test_compute_progress_bps_over_funded_capped() {
        assert_eq!(compute_progress_bps(2_000, 1_000), 10_000);
    }

    /// @security goal == 0 must return 0, not panic.
    #[test]
    fn test_compute_progress_bps_zero_goal_returns_zero() {
        assert_eq!(compute_progress_bps(500, 0), 0);
    }

    /// @security Negative raised must return 0, not a wrapped value.
    #[test]
    fn test_compute_progress_bps_negative_raised_returns_zero() {
        assert_eq!(compute_progress_bps(-100, 1_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_zero_raised() {
        assert_eq!(compute_progress_bps(0, 1_000), 0);
    }

    // --- clamp_proptest_cases ---

    #[test]
    fn test_clamp_proptest_cases_below_min() {
        assert_eq!(clamp_proptest_cases(0), PROPTEST_CASES_MIN);
    }

    #[test]
    fn test_clamp_proptest_cases_at_min() {
        assert_eq!(clamp_proptest_cases(PROPTEST_CASES_MIN), PROPTEST_CASES_MIN);
    }

    #[test]
    fn test_constants_are_ordered_correctly() {
        assert!(DEADLINE_OFFSET_MIN < DEADLINE_OFFSET_MAX);
        assert!(GOAL_MIN < GOAL_MAX);
        assert!(PROPTEST_CASES_MIN < PROPTEST_CASES_MAX);
        assert!(PROGRESS_BPS_CAP > 0);
        assert!(FEE_BPS_CAP > 0);
        assert!(GENERATOR_BATCH_MAX > 0);
        assert!(MAX_TOKEN_DECIMALS > 0);
        assert!(DEADLINE_ENDING_SOON_THRESHOLD > 0);
        assert!(DEADLINE_ENDING_SOON_THRESHOLD < DEADLINE_OFFSET_MIN);
    }

    #[test]
    fn test_contract_constants_match_rust_constants() {
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
        assert_eq!(client.max_token_decimals(), MAX_TOKEN_DECIMALS);
        assert_eq!(
            client.deadline_ending_soon_threshold(),
            DEADLINE_ENDING_SOON_THRESHOLD
        );
    }

    // ── Deadline Offset Validation ────────────────────────────────────────────

    #[test]
    fn test_is_valid_deadline_offset_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MIN));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MIN - 1)));
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MAX));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MAX + 1)));
        assert!(client.is_valid_deadline_offset(&500_000));
    }

    #[test]
    fn test_is_valid_deadline_offset_edge_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_deadline_offset(&0));
        assert!(!client.is_valid_deadline_offset(&999));
        assert!(!client.is_valid_deadline_offset(&u64::MAX));
    #[test]
    fn test_constants_have_reasonable_values() {
        // Deadline offsets should be in seconds
        assert!(DEADLINE_OFFSET_MIN >= 60);
        assert!(DEADLINE_OFFSET_MAX <= 100_000_000);
        
        // Goals should be positive
        assert!(GOAL_MIN > 0);
        assert!(GOAL_MAX > 0);
        
        // Basis points should be <= 10,000
        assert!(PROGRESS_BPS_CAP <= 10_000);
        assert!(FEE_BPS_CAP <= 10_000);
    }

    // ── is_valid_deadline_offset Tests ────────────────────────────────────────

    #[test]
    fn test_is_valid_deadline_offset_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MIN));
    }

    #[test]
    fn test_contract_is_valid_deadline_offset() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MIN));
        assert!(client.is_valid_deadline_offset(&500_000u64));
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MAX));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MIN - 1)));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MAX + 1)));
    }

    #[test]
    fn test_is_valid_deadline_offset_midrange() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&500_000));
        assert!(client.is_valid_deadline_offset(&100_000));
        assert!(client.is_valid_deadline_offset(&10_000));
    }

    #[test]
    fn test_is_valid_deadline_offset_zero_and_negative() {
        let (_env, client) = setup();
        assert!(!client.is_valid_deadline_offset(&0));
        // Note: u64 cannot be negative, so we test the lower bound
    }

    // ── is_valid_goal Tests ──────────────────────────────────────────────────

    #[test]
    fn test_is_valid_goal_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&GOAL_MIN));
        assert!(client.is_valid_goal(&GOAL_MAX));
        assert!(!client.is_valid_goal(&(GOAL_MIN - 1)));
        assert!(!client.is_valid_goal(&(GOAL_MAX + 1)));
    }

    #[test]
    fn test_is_valid_goal_midrange() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&50_000_000));
        assert!(client.is_valid_goal(&10_000_000));
        assert!(client.is_valid_goal(&1_000_000));
    }

    // ── Goal Validation ───────────────────────────────────────────────────────

    #[test]
    fn test_is_valid_goal_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&GOAL_MIN));
        assert!(!client.is_valid_goal(&(GOAL_MIN - 1)));
        assert!(client.is_valid_goal(&GOAL_MAX));
        assert!(!client.is_valid_goal(&(GOAL_MAX + 1)));
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

    // ── Min Contribution Validation ───────────────────────────────────────────

    #[test]
    fn test_is_valid_min_contribution() {
        let (_env, client) = setup();
        let goal = 1_000_000i128;
        assert!(client.is_valid_min_contribution(&MIN_CONTRIBUTION_FLOOR, &goal));
        assert!(client.is_valid_min_contribution(&500_000, &goal));
        assert!(client.is_valid_min_contribution(&goal, &goal));
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
        let min = 1_000i128;
        assert!(client.is_valid_contribution_amount(&min, &min));
        assert!(client.is_valid_contribution_amount(&(min + 1), &min));
        assert!(client.is_valid_contribution_amount(&1_000_000, &min));
        assert!(!client.is_valid_contribution_amount(&(min - 1), &min));
        assert!(!client.is_valid_contribution_amount(&0, &min));
        assert!(!client.is_valid_contribution_amount(&-1, &min));
    }

    // ── Fee Bps Validation ────────────────────────────────────────────────────

    #[test]
    fn test_is_valid_fee_bps() {
        let (_env, client) = setup();
        assert!(client.is_valid_fee_bps(&0));
        assert!(client.is_valid_fee_bps(&5_000));
        assert!(client.is_valid_fee_bps(&FEE_BPS_CAP));
        assert!(!client.is_valid_fee_bps(&(FEE_BPS_CAP + 1)));
        assert!(!client.is_valid_fee_bps(&u32::MAX));
    }

    // ── Generator Batch Size Validation ───────────────────────────────────────

    #[test]
    fn test_is_valid_generator_batch_size() {
        let (_env, client) = setup();
        assert!(client.is_valid_generator_batch_size(&1));
        assert!(client.is_valid_generator_batch_size(&256));
        assert!(client.is_valid_generator_batch_size(&GENERATOR_BATCH_MAX));
        assert!(!client.is_valid_generator_batch_size(&0));
        assert!(!client.is_valid_generator_batch_size(&(GENERATOR_BATCH_MAX + 1)));
    }

    // ── Clamping ──────────────────────────────────────────────────────────────

    #[test]
    fn test_is_valid_goal_zero_and_negative() {
    fn test_clamp_proptest_cases() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&0), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&1), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&64), 64);
        assert_eq!(client.clamp_proptest_cases(&128), 128);
        assert_eq!(client.clamp_proptest_cases(&PROPTEST_CASES_MAX), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&1_000), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&u32::MAX), PROPTEST_CASES_MAX);
    }

    #[test]
    fn test_clamp_progress_bps() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&-1_000), 0);
        assert_eq!(client.clamp_progress_bps(&-1), 0);
        assert_eq!(client.clamp_progress_bps(&0), 0);
        assert_eq!(client.clamp_progress_bps(&5_000), 5_000);
        assert_eq!(client.clamp_progress_bps(&10_000), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&10_001), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&i128::MAX), PROGRESS_BPS_CAP);
    }

    // ── compute_progress_bps ──────────────────────────────────────────────────

    #[test]
    fn test_compute_progress_bps_basic() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&500, &1_000), 5_000);
        assert_eq!(client.compute_progress_bps(&1_000, &1_000), 10_000);
        assert_eq!(client.compute_progress_bps(&2_000, &1_000), 10_000);
    }

    #[test]
    fn test_compute_progress_bps_edge_cases() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&500, &0), 0);
        assert_eq!(client.compute_progress_bps(&500, &-1_000), 0);
        assert_eq!(client.compute_progress_bps(&-100, &1_000), 0);
        assert_eq!(client.compute_progress_bps(&1, &10_000), 1);
    }

    #[test]
    fn test_compute_progress_bps_overflow_safety() {
        let (_env, client) = setup();
        let result = client.compute_progress_bps(&(i128::MAX / 2), &1_000);
        assert_eq!(result, PROGRESS_BPS_CAP);
    }

    // ── compute_fee_amount ────────────────────────────────────────────────────

    #[test]
    fn test_compute_fee_amount_basic() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_000, &5_000), 500);
        assert_eq!(client.compute_fee_amount(&1_000, &10_000), 1_000);
    }

    #[test]
    fn test_compute_fee_amount_edge_cases() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&0, &5_000), 0);
        assert_eq!(client.compute_fee_amount(&-1_000, &5_000), 0);
        assert_eq!(client.compute_fee_amount(&1_000, &0), 0);
        assert_eq!(client.compute_fee_amount(&0, &0), 0);
    }

    #[test]
    fn test_compute_fee_amount_floor_division() {
        let (_env, client) = setup();
        // 1/3 fee (should floor)
        assert_eq!(client.compute_fee_amount(&1000, &3333), 333);
        // 2/3 fee (should floor)
        assert_eq!(client.compute_fee_amount(&1000, &6666), 666);
        assert!(!client.is_valid_goal(&0));
        assert!(!client.is_valid_goal(&-1));
        assert!(!client.is_valid_goal(&-1_000_000));
    }

    // ── is_valid_min_contribution Tests ──────────────────────────────────────

    #[test]
    fn test_is_valid_min_contribution_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_min_contribution(&1, &1_000));
        assert!(client.is_valid_min_contribution(&500, &1_000));
        assert!(client.is_valid_min_contribution(&1_000, &1_000));
        assert!(client.is_valid_min_contribution(&1, &100_000_000));
    }

    #[test]
    fn test_is_valid_min_contribution_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_min_contribution(&0, &1_000));
        assert!(!client.is_valid_min_contribution(&-1, &1_000));
        assert!(!client.is_valid_min_contribution(&1_001, &1_000));
        assert!(!client.is_valid_min_contribution(&100_000_001, &100_000_000));
    }

    // ── is_valid_contribution_amount Tests ───────────────────────────────────

    #[test]
    fn test_is_valid_contribution_amount_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_contribution_amount(&1, &1));
        assert!(client.is_valid_contribution_amount(&100, &50));
        assert!(client.is_valid_contribution_amount(&1_000, &1));
        assert!(client.is_valid_contribution_amount(&100_000_000, &1));
    }

    #[test]
    fn test_is_valid_contribution_amount_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_contribution_amount(&0, &1));
        assert!(!client.is_valid_contribution_amount(&-1, &1));
        assert!(!client.is_valid_contribution_amount(&50, &100));
        assert!(!client.is_valid_contribution_amount(&1, &100));
    }

    // ── is_valid_fee_bps Tests ───────────────────────────────────────────────

    #[test]
    fn test_is_valid_fee_bps_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_fee_bps(&0));
        assert!(client.is_valid_fee_bps(&1));
        assert!(client.is_valid_fee_bps(&5_000));
        assert!(client.is_valid_fee_bps(&10_000));
    }

    #[test]
    fn test_is_valid_fee_bps_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_fee_bps(&10_001));
        assert!(!client.is_valid_fee_bps(&20_000));
        assert!(!client.is_valid_fee_bps(&u32::MAX));
    }

    // ── is_valid_generator_batch_size Tests ──────────────────────────────────

    #[test]
    fn test_is_valid_generator_batch_size_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_generator_batch_size(&1));
        assert!(client.is_valid_generator_batch_size(&256));
        assert!(client.is_valid_generator_batch_size(&512));
    }

    #[test]
    fn test_is_valid_generator_batch_size_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_generator_batch_size(&0));
        assert!(!client.is_valid_generator_batch_size(&513));
        assert!(!client.is_valid_generator_batch_size(&1_000));
    }

    // ── clamp_proptest_cases Tests ───────────────────────────────────────────

    #[test]
    fn test_clamp_proptest_cases_below_min() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&0), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&1), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&31), PROPTEST_CASES_MIN);
    }

    #[test]
    fn test_clamp_proptest_cases_within_range() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&32), 32);
        assert_eq!(client.clamp_proptest_cases(&100), 100);
        assert_eq!(client.clamp_proptest_cases(&256), 256);
    }

    #[test]
    fn test_clamp_proptest_cases_above_max() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&257), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&1_000), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&u32::MAX), PROPTEST_CASES_MAX);
    }

    // ── clamp_progress_bps Tests ─────────────────────────────────────────────

    #[test]
    fn test_clamp_progress_bps_negative_values() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&-1_000), 0);
        assert_eq!(client.clamp_progress_bps(&-1), 0);
    }

    #[test]
    fn test_clamp_progress_bps_zero() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&0), 0);
    }

    #[test]
    fn test_clamp_progress_bps_within_range() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&1), 1);
        assert_eq!(client.clamp_progress_bps(&5_000), 5_000);
        assert_eq!(client.clamp_progress_bps(&10_000), 10_000);
    }

    #[test]
    fn test_clamp_progress_bps_above_cap() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&10_001), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&20_000), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&i128::MAX), PROGRESS_BPS_CAP);
    }

    // ── compute_progress_bps Tests ───────────────────────────────────────────

    #[test]
    fn test_compute_progress_bps_zero_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&0, &0), 0);
        assert_eq!(client.compute_progress_bps(&1_000, &0), 0);
        assert_eq!(client.compute_progress_bps(&100_000_000, &0), 0);
    }

    #[test]
    fn test_compute_progress_bps_negative_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &-1), 0);
        assert_eq!(client.compute_progress_bps(&100_000, &-1_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_zero_raised() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&0, &1_000), 0);
        assert_eq!(client.compute_progress_bps(&0, &100_000_000), 0);
        assert_eq!(client.compute_fee_amount(&1_000, &3_333), 333);
        assert_eq!(client.compute_fee_amount(&1_000, &6_666), 666);
    }

    // ── log_tag ───────────────────────────────────────────────────────────────

    #[test]
    fn test_log_tag() {
    fn test_contract_is_valid_goal() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&GOAL_MIN));
        assert!(client.is_valid_goal(&50_000_000i128));
        assert!(client.is_valid_goal(&GOAL_MAX));
        assert!(!client.is_valid_goal(&(GOAL_MIN - 1)));
        assert!(!client.is_valid_goal(&(GOAL_MAX + 1)));
    }

    #[test]
    fn test_contract_clamp_proptest_cases() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&0u32), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&100u32), 100);
        assert_eq!(client.clamp_proptest_cases(&1_000u32), PROPTEST_CASES_MAX);
    }

    #[test]
    fn test_contract_compute_progress_bps() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&500i128, &1_000i128), 5_000);
        assert_eq!(client.compute_progress_bps(&2_000i128, &1_000i128), 10_000);
        assert_eq!(client.compute_progress_bps(&500i128, &0i128), 0);
        assert_eq!(client.compute_progress_bps(&(-100i128), &1_000i128), 0);
    }

    #[test]
    fn test_contract_log_tag() {
        let (env, client) = setup();
        assert_eq!(client.log_tag(), Symbol::new(&env, "boundary"));
    }

    // ── New Edge Cases: is_ui_displayable_progress (Issue #423) ──────────────

    #[test]
    fn test_is_ui_displayable_progress_valid_range() {
        let (_env, client) = setup();
        assert!(client.is_ui_displayable_progress(&0));
        assert!(client.is_ui_displayable_progress(&5_000));
        assert!(client.is_ui_displayable_progress(&PROGRESS_BPS_CAP));
    }

    #[test]
    fn test_is_ui_displayable_progress_above_cap_rejected() {
        let (_env, client) = setup();
        assert!(!client.is_ui_displayable_progress(&(PROGRESS_BPS_CAP + 1)));
        assert!(!client.is_ui_displayable_progress(&u32::MAX));
    }

    // ── New Edge Cases: compute_display_percent (Issue #423) ─────────────────

    #[test]
    fn test_compute_display_percent_basic() {
        let (_env, client) = setup();
        assert_eq!(client.compute_display_percent(&0), 0);
        assert_eq!(client.compute_display_percent(&5_000), 5_000);
        assert_eq!(client.compute_display_percent(&10_000), 10_000);
    }

    #[test]
    fn test_compute_display_percent_clamps_above_cap() {
        let (_env, client) = setup();
        assert_eq!(client.compute_display_percent(&10_001), PROGRESS_BPS_CAP);
        assert_eq!(client.compute_display_percent(&u32::MAX), PROGRESS_BPS_CAP);
    }

    // ── New Edge Cases: is_contribution_ui_safe (Issue #423) ─────────────────

    #[test]
    fn test_is_contribution_ui_safe_valid() {
        let (_env, client) = setup();
        // XLM decimals = 7
        assert!(client.is_contribution_ui_safe(&1_000, &1_000, &7));
        assert!(client.is_contribution_ui_safe(&100_000_000, &1_000, &7));
        // USDC decimals = 6
        assert!(client.is_contribution_ui_safe(&1_000, &1_000, &6));
    }

    #[test]
    fn test_is_contribution_ui_safe_below_minimum_rejected() {
        let (_env, client) = setup();
        assert!(!client.is_contribution_ui_safe(&999, &1_000, &7));
        assert!(!client.is_contribution_ui_safe(&0, &1_000, &7));
        assert!(!client.is_contribution_ui_safe(&-1, &1_000, &7));
    }

    #[test]
    fn test_is_contribution_ui_safe_excessive_decimals_rejected() {
        let (_env, client) = setup();
        assert!(!client.is_contribution_ui_safe(&1_000, &1_000, &(MAX_TOKEN_DECIMALS + 1)));
        assert!(!client.is_contribution_ui_safe(&1_000, &1_000, &255));
    }

    #[test]
    fn test_is_contribution_ui_safe_overflow_rejected() {
        let (_env, client) = setup();
        // i128::MAX * 10^18 overflows
        assert!(!client.is_contribution_ui_safe(&i128::MAX, &1_000, &18));
    }

    #[test]
    fn test_is_contribution_ui_safe_zero_decimals() {
        let (_env, client) = setup();
        // 0 decimals: scale = 1, no overflow possible for valid amounts
        assert!(client.is_contribution_ui_safe(&1_000, &1_000, &0));
    }

    // ── New Edge Cases: deadline_ui_state (Issue #423) ────────────────────────

    #[test]
    fn test_deadline_ui_state_expired() {
        assert_eq!(deadline_ui_state(0), DeadlineUiState::Expired);
    }

    #[test]
    fn test_deadline_ui_state_ending_soon_boundary() {
        assert_eq!(
            deadline_ui_state(DEADLINE_ENDING_SOON_THRESHOLD),
            DeadlineUiState::EndingSoon
        );
        assert_eq!(deadline_ui_state(1), DeadlineUiState::EndingSoon);
        assert_eq!(
            deadline_ui_state(DEADLINE_ENDING_SOON_THRESHOLD - 1),
            DeadlineUiState::EndingSoon
        );
    }

    #[test]
    fn test_deadline_ui_state_active() {
        assert_eq!(
            deadline_ui_state(DEADLINE_ENDING_SOON_THRESHOLD + 1),
            DeadlineUiState::Active
        );
        assert_eq!(deadline_ui_state(DEADLINE_OFFSET_MIN), DeadlineUiState::Active);
        assert_eq!(deadline_ui_state(u64::MAX), DeadlineUiState::Active);
    }

    // ── New Edge Cases: compute_net_payout (Issue #423) ───────────────────────

    #[test]
    fn test_compute_net_payout_basic() {
        let (_env, client) = setup();
        // 10 % fee on 1 000 → net = 900
        assert_eq!(client.compute_net_payout(&1_000, &1_000), 900);
        // 0 % fee → net = total
        assert_eq!(client.compute_net_payout(&1_000, &0), 1_000);
        // 100 % fee → net = 0
        assert_eq!(client.compute_net_payout(&1_000, &10_000), 0);
    }

    #[test]
    fn test_compute_net_payout_zero_total() {
        let (_env, client) = setup();
        assert_eq!(client.compute_net_payout(&0, &5_000), 0);
    }

    #[test]
    fn test_compute_net_payout_invalid_fee_returns_zero() {
        let (_env, client) = setup();
        // fee_bps > FEE_BPS_CAP → None → contract returns 0
        assert_eq!(client.compute_net_payout(&1_000, &(FEE_BPS_CAP + 1)), 0);
        assert_eq!(client.compute_net_payout(&1_000, &u32::MAX), 0);
    }

    #[test]
    fn test_compute_net_payout_negative_total_returns_zero() {
        let (_env, client) = setup();
        assert_eq!(client.compute_net_payout(&-1_000, &1_000), 0);
    }

    // ── Pure function: compute_net_payout returns None on invalid fee ─────────

    #[test]
    fn test_pure_compute_net_payout_none_on_invalid_fee() {
        assert_eq!(compute_net_payout(1_000, FEE_BPS_CAP + 1), None);
    }

    #[test]
    fn test_pure_compute_net_payout_some_on_valid_fee() {
        assert_eq!(compute_net_payout(1_000, 1_000), Some(900));
        assert_eq!(compute_net_payout(1_000, 0), Some(1_000));
        assert_eq!(compute_net_payout(0, 5_000), Some(0));
    }

    // ── Property-Based Tests ──────────────────────────────────────────────────
    // @notice These tests use proptest to explore the input space systematically.
    //         Each property is tested with 64+ randomly generated cases.
    // ── 4. Property-Based Tests ───────────────────────────────────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Property: All valid deadline offsets pass validation.
        /// Any offset in [MIN, MAX] must be accepted.
        #[test]
        fn prop_valid_deadline_offset_always_accepted(
            offset in DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX
        ) {
            prop_assert!(is_valid_deadline_offset(offset));
        }

        #[test]
        fn prop_deadline_offset_below_min_rejected(offset in 0u64..DEADLINE_OFFSET_MIN) {
            prop_assert!(!is_valid_deadline_offset(offset));
        }

        #[test]
        fn prop_deadline_offset_above_max_invalid(offset in (DEADLINE_OFFSET_MAX + 1)..u64::MAX) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_deadline_offset(&offset));
        }

        /// Any offset below MIN must be rejected.
        #[test]
        fn prop_deadline_offset_below_min_always_rejected(
            offset in 0u64..DEADLINE_OFFSET_MIN
        fn prop_deadline_offset_above_max_rejected(
            offset in (DEADLINE_OFFSET_MAX + 1)..=(DEADLINE_OFFSET_MAX + 100_000)
        ) {
            prop_assert!(!is_valid_deadline_offset(offset));
        }

        #[test]
        fn prop_goal_below_min_invalid(goal in i128::MIN..GOAL_MIN) {
        fn prop_goal_invalidity(goal in i128::MIN..GOAL_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_goal(&goal));
        fn prop_valid_goal_always_accepted(goal in GOAL_MIN..=GOAL_MAX) {
            prop_assert!(is_valid_goal(goal));
        }

        #[test]
        fn prop_goal_below_min_rejected(goal in i128::MIN..GOAL_MIN) {
            prop_assert!(!is_valid_goal(goal));
        }

        #[test]
        fn prop_goal_above_max_rejected(goal in (GOAL_MAX + 1)..i128::MAX) {
            prop_assert!(!is_valid_goal(goal));
        }

        #[test]
        fn prop_progress_bps_always_bounded(
            raised in -1_000_000_000i128..=1_000_000_000i128,
        /// Property: Progress BPS is always bounded by PROGRESS_BPS_CAP.
        #[test]
        fn prop_progress_bps_bounds(
            raised in -1_000_000_000i128..=200_000_000i128,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let bps = compute_progress_bps(raised, goal);
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
        /// Property: Clamped progress BPS is always bounded.
        #[test]
        fn prop_clamped_progress_bps_bounds(raw in i128::MIN..=i128::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_progress_bps(&raw);
            prop_assert!(clamped <= PROGRESS_BPS_CAP);
        }

        /// Property: Proptest cases are always within bounds after clamping.
        #[test]
        fn prop_clamped_cases_bounds(requested in 0u32..=u32::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_proptest_cases(&requested);
        /// Any offset above MAX must be rejected.
        #[test]
        fn prop_deadline_offset_above_max_always_rejected(
            offset in (DEADLINE_OFFSET_MAX + 1)..=(DEADLINE_OFFSET_MAX + 100_000)
        ) {
            prop_assert!(!is_valid_deadline_offset(offset));
        }

        /// Any goal in [MIN, MAX] must be accepted.
        #[test]
        fn prop_valid_goal_always_accepted(goal in GOAL_MIN..=GOAL_MAX) {
            prop_assert!(is_valid_goal(goal));
        }

        /// Any goal below MIN must be rejected.
        #[test]
        fn prop_goal_below_min_always_rejected(goal in (-1_000_000i128..GOAL_MIN)) {
            prop_assert!(!is_valid_goal(goal));
        }

        /// Any goal above MAX must be rejected.
        #[test]
        fn prop_goal_above_max_always_rejected(
            goal in (GOAL_MAX + 1)..=(GOAL_MAX + 1_000_000)
        ) {
            prop_assert!(!is_valid_goal(goal));
        }

        /// Progress bps must never exceed PROGRESS_BPS_CAP for any raised/goal combo.
        #[test]
        fn prop_progress_bps_never_exceeds_cap(
            raised in -1_000i128..=200_000_000i128,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let bps = compute_progress_bps(raised, goal);
            prop_assert!(bps <= PROGRESS_BPS_CAP);
        }

        /// clamp_progress_bps must never exceed PROGRESS_BPS_CAP.
        #[test]
        fn prop_clamp_progress_bps_never_exceeds_cap(raw in -100_000i128..=100_000i128) {
            prop_assert!(clamp_progress_bps(raw) <= PROGRESS_BPS_CAP);
        }

        /// clamp_proptest_cases output must always be in [MIN, MAX].
        #[test]
        fn prop_clamp_proptest_cases_always_in_bounds(requested in 0u32..=1_000u32) {
            let clamped = clamp_proptest_cases(requested);
            prop_assert!(clamped >= PROPTEST_CASES_MIN);
            prop_assert!(clamped <= PROPTEST_CASES_MAX);
        }

        /// Property: Fee amounts are always non-negative.
        #[test]
        fn prop_fee_amount_non_negative(
            amount in 0i128..=100_000_000i128,
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
            prop_assert_eq!(compute_progress_bps(raised, 0), 0);
        }

        #[test]
        fn prop_clamp_progress_bps_within_bounds(raw in i128::MIN..=i128::MAX) {
            let clamped = clamp_progress_bps(raw);
            prop_assert!(clamped <= PROGRESS_BPS_CAP);
        }

        /// Property (Issue #423): is_ui_displayable_progress is true iff bps <= cap.
        #[test]
        fn prop_min_contribution_valid_when_in_range(
        /// Property: Fee amount never exceeds the original amount.
        #[test]
        fn prop_fee_amount_not_exceeds_original(
            amount in 1i128..=100_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&amount, &fee_bps);
            prop_assert!(fee <= amount);
        }

        /// Property: Valid min contributions are always >= MIN_CONTRIBUTION_FLOOR.
        #[test]
        fn prop_valid_min_contribution_floor(
            min_contrib in MIN_CONTRIBUTION_FLOOR..=GOAL_MAX,
            goal in GOAL_MIN..=GOAL_MAX
        fn prop_ui_displayable_progress_iff_within_cap(bps in 0u32..=u32::MAX) {
            prop_assert_eq!(is_ui_displayable_progress(bps), bps <= PROGRESS_BPS_CAP);
        }

        /// Property (Issue #423): compute_display_percent never exceeds cap.
        #[test]
        fn prop_display_percent_never_exceeds_cap(bps in 0u32..=u32::MAX) {
            prop_assert!(compute_display_percent(bps) <= PROGRESS_BPS_CAP);
        }

        /// Property (Issue #423): compute_net_payout returns None iff fee_bps > cap.
        #[test]
        fn prop_net_payout_none_iff_fee_above_cap(
            total in 0i128..=100_000_000i128,
            fee_bps in (FEE_BPS_CAP + 1)..=u32::MAX
        ) {
            prop_assert_eq!(compute_net_payout(total, fee_bps), None);
        }

        /// Property (Issue #423): compute_net_payout is Some for valid fee_bps.
        #[test]
        fn prop_net_payout_some_for_valid_fee(
            total in 0i128..=100_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            prop_assert!(compute_net_payout(total, fee_bps).is_some());
        }

        /// Property (Issue #423): net payout never exceeds total.
        #[test]
        fn prop_net_payout_never_exceeds_total(
            total in 0i128..=100_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            if let Some(net) = compute_net_payout(total, fee_bps) {
                prop_assert!(net <= total);
                prop_assert!(net >= 0);
            }
        }

        /// Property (Issue #423): deadline_ui_state(0) is always Expired.
        #[test]
        fn prop_contribution_amount_valid_when_meets_minimum(
            amount in MIN_CONTRIBUTION_FLOOR..=1_000_000i128,
            min_contrib in MIN_CONTRIBUTION_FLOOR..=1_000_000i128
        /// Property: Valid contribution amounts are >= min_contribution.
        #[test]
        fn prop_valid_contribution_amount(
            amount in MIN_CONTRIBUTION_FLOOR..=100_000_000i128,
            min_contrib in MIN_CONTRIBUTION_FLOOR..=100_000_000i128
        fn prop_deadline_expired_when_zero(_x in 0u32..1u32) {
            prop_assert_eq!(deadline_ui_state(0), DeadlineUiState::Expired);
        }

        /// Property (Issue #423): deadline_ui_state is never Expired for > 0.
        #[test]
        fn prop_deadline_not_expired_when_positive(secs in 1u64..=u64::MAX) {
            prop_assert_ne!(deadline_ui_state(secs), DeadlineUiState::Expired);
        }

        /// Property (Issue #423): is_contribution_ui_safe rejects excessive decimals.
        #[test]
        fn prop_contribution_ui_safe_rejects_excess_decimals(
            amount in MIN_CONTRIBUTION_FLOOR..=GOAL_MAX,
            decimals in (MAX_TOKEN_DECIMALS + 1)..=255u32
        ) {
            prop_assert!(!is_contribution_ui_safe(amount, MIN_CONTRIBUTION_FLOOR, decimals));
        }

        /// Property (Issue #423): is_contribution_ui_safe rejects below-minimum amounts.
        #[test]
        fn prop_fee_bps_valid_when_within_cap(fee_bps in 0u32..=FEE_BPS_CAP) {
        /// Property: Valid fee BPS are always <= FEE_BPS_CAP.
        #[test]
        fn prop_valid_fee_bps(fee_bps in 0u32..=FEE_BPS_CAP) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_fee_bps(&fee_bps));
        }

        #[test]
        fn prop_batch_size_valid_when_in_range(batch_size in 1u32..=GENERATOR_BATCH_MAX) {
        /// Property: Valid batch sizes are always > 0 and <= GENERATOR_BATCH_MAX.
        #[test]
        fn prop_valid_batch_size(batch_size in 1u32..=GENERATOR_BATCH_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_generator_batch_size(&batch_size));
        fn prop_contribution_ui_safe_rejects_below_minimum(
            amount in i128::MIN..MIN_CONTRIBUTION_FLOOR
        ) {
            prop_assert!(!is_contribution_ui_safe(amount, MIN_CONTRIBUTION_FLOOR, 7));
        }
    }

    // ── Regression Seeds ──────────────────────────────────────────────────────

    #[test]
    fn regression_deadline_offset_100_rejected() {
        let (_env, client) = setup();
        assert!(!client.is_valid_deadline_offset(&100));
    }

    #[test]
    fn regression_goal_zero_rejected() {
        let (_env, client) = setup();
        assert!(!client.is_valid_goal(&0));

    #[test]
    fn regression_deadline_offset_minimum_1000() {
        // Regression: Deadline offset minimum was previously 100, causing flaky tests.
        // This test ensures it's now 1,000 (17 minutes).
        let (_env, client) = setup();
        assert_eq!(client.deadline_offset_min(), 1_000);
        assert!(!client.is_valid_deadline_offset(&100));
        assert!(client.is_valid_deadline_offset(&1_000));
    }

    #[test]
    fn regression_progress_bps_never_exceeds_cap() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&i128::MAX, &1), PROGRESS_BPS_CAP);
    }

    #[test]
    fn regression_fee_amount_never_negative() {
        let (_env, client) = setup();
        // Even with negative inputs, should return 0 or positive
        assert!(client.compute_fee_amount(&-1_000_000, &5000) >= 0);
        // Regression: Progress BPS should never exceed 10,000 (100%).
        let (_env, client) = setup();
        let bps = client.compute_progress_bps(&1_000_000_000, &1);
        assert_eq!(bps, PROGRESS_BPS_CAP);
    }

    #[test]
    fn regression_fee_calculation_precision() {
        // Regression: Fee calculation should use integer floor division.
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_001, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_009, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_010, &1_000), 101);
    }

    #[test]
    fn regression_zero_goal_division_safety() {
        // Regression: Division by zero should be prevented.
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &0), 0);
        assert_eq!(client.compute_progress_bps(&100_000_000, &0), 0);
        /// min_contribution in [1, goal] must always be valid for that goal.
        #[test]
        fn prop_min_contribution_in_range_always_valid(
            (goal, min) in (GOAL_MIN..=GOAL_MAX)
                .prop_flat_map(|g| (Just(g), MIN_CONTRIBUTION_FLOOR..=g))
        ) {
            prop_assert!(is_valid_min_contribution(min, goal));
        }

        /// Contribution >= min_contribution must always be valid.
        #[test]
        fn prop_contribution_at_or_above_min_always_valid(
            (min_contribution, amount) in (MIN_CONTRIBUTION_FLOOR..=1_000_000i128)
                .prop_flat_map(|m| (Just(m), m..=(m + 10_000_000)))
        ) {
            prop_assert!(is_valid_contribution_amount(amount, min_contribution));
        }
    }

    // ── 5. Regression Seeds ───────────────────────────────────────────────────
    //
    // These inputs previously caused test failures and are pinned here to
    // prevent regressions.

    /// @notice Seed: goal=1M, offset=100 — the old buggy minimum caused flaky CI.
    #[test]
    fn regression_seed_goal_1m_offset_100_rejected() {
        assert!(is_valid_goal(1_000_000));
        assert!(!is_valid_deadline_offset(100)); // 100 is below the fixed MIN of 1_000
    }

    /// @notice Seed: goal=2M, offset=100, contribution=100K.
    #[test]
    fn regression_seed_goal_2m_offset_100_rejected() {
        assert!(is_valid_goal(2_000_000));
        assert!(!is_valid_deadline_offset(100));
        assert!(is_valid_contribution_amount(100_000, 1_000));
    }

    // ── 6. Frontend UX Edge Cases ─────────────────────────────────────────────

    /// @notice A 0 % progress bar must render as exactly 0, not a negative number.
    #[test]
    fn frontend_zero_raised_renders_zero_percent() {
        assert_eq!(compute_progress_bps(0, GOAL_MIN), 0);
    }

    /// @notice A 100 % progress bar must render as exactly 10 000 bps.
    #[test]
    fn frontend_fully_funded_renders_100_percent() {
        assert_eq!(compute_progress_bps(GOAL_MIN, GOAL_MIN), 10_000);
    }

    /// @notice An over-funded campaign must still render as 100 %, not > 100 %.
    #[test]
    fn frontend_over_funded_capped_at_100_percent() {
        assert_eq!(compute_progress_bps(GOAL_MAX * 2, GOAL_MIN), 10_000);
    }

    /// @notice Fee cap must equal progress cap so both display consistently.
    #[test]
    fn frontend_fee_cap_equals_progress_cap() {
        assert_eq!(FEE_BPS_CAP, PROGRESS_BPS_CAP);
    }

    /// @notice Deadline offset of exactly 1 000 s renders a valid countdown.
    #[test]
    fn frontend_minimum_deadline_renders_valid_countdown() {
        assert!(is_valid_deadline_offset(1_000));
        assert!(client.compute_fee_amount(&-1_000_000, &5_000) >= 0);
    }

    /// @security Regression: net payout with fee > cap must not silently return
    ///           a wrong value — the contract must return 0 (None path).
    #[test]
    fn regression_net_payout_invalid_fee_returns_zero() {
        let (_env, client) = setup();
        assert_eq!(client.compute_net_payout(&1_000_000, &(FEE_BPS_CAP + 1)), 0);
    }

    /// @security Regression: progress bar must never show > 100 % for over-funded
    ///           campaigns — critical for frontend UX trust.
    #[test]
    fn regression_overfunded_campaign_capped_at_100_percent() {
        let (_env, client) = setup();
        assert_eq!(
            client.compute_progress_bps(&200_000_000, &100_000_000),
            PROGRESS_BPS_CAP
        );
        assert!(client.is_ui_displayable_progress(&PROGRESS_BPS_CAP));
    }

    /// @security Regression: deadline_ui_state(0) must be Expired, not EndingSoon.
    #[test]
    fn regression_zero_seconds_is_expired_not_ending_soon() {
        assert_eq!(deadline_ui_state(0), DeadlineUiState::Expired);
        assert_ne!(deadline_ui_state(0), DeadlineUiState::EndingSoon);
    }
}
