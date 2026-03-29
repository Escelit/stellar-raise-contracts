#[cfg(test)]
mod state_compression_tests {
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::Env;

    use crate::state_compression::{
        apply_contribution, apply_refund, is_expired, is_goal_reached, load, load_or_init,
        progress_bps, store, CompressedState,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_env() -> Env {
        Env::default()
    }

    fn default_state() -> CompressedState {
        CompressedState {
            goal: 1_000,
            deadline: 9_999_999,
            min_contribution: 10,
            total_raised: 0,
        }
    }

    // ── load / store ──────────────────────────────────────────────────────────

    /// load returns None on a fresh environment.
    #[test]
    fn test_load_returns_none_when_absent() {
        let env = make_env();
        assert!(load(&env).is_none());
    }

    /// load_or_init returns a zero-value struct when absent.
    #[test]
    fn test_load_or_init_returns_zero_struct_when_absent() {
        let env = make_env();
        let state = load_or_init(&env);
        assert_eq!(state.goal, 0);
        assert_eq!(state.deadline, 0);
        assert_eq!(state.min_contribution, 0);
        assert_eq!(state.total_raised, 0);
    }

    /// store then load round-trips all four fields correctly.
    #[test]
    fn test_store_and_load_round_trip() {
        let env = make_env();
        let original = default_state();
        store(&env, &original);
        let loaded = load(&env).expect("state should exist after store");
        assert_eq!(loaded, original);
    }

    /// Overwriting with store replaces the previous value.
    #[test]
    fn test_store_overwrites_previous_value() {
        let env = make_env();
        store(&env, &default_state());
        let updated = CompressedState {
            goal: 5_000,
            deadline: 1_000,
            min_contribution: 50,
            total_raised: 200,
        };
        store(&env, &updated);
        assert_eq!(load(&env).unwrap(), updated);
    }

    // ── apply_contribution ────────────────────────────────────────────────────

    /// apply_contribution increases total_raised and persists.
    #[test]
    fn test_apply_contribution_increases_total_raised() {
        let env = make_env();
        store(&env, &default_state());
        let result = apply_contribution(&env, 100).expect("should succeed");
        assert_eq!(result.total_raised, 100);
        assert_eq!(load(&env).unwrap().total_raised, 100);
    }

    /// Multiple contributions accumulate correctly.
    #[test]
    fn test_apply_contribution_accumulates() {
        let env = make_env();
        store(&env, &default_state());
        apply_contribution(&env, 300).unwrap();
        apply_contribution(&env, 200).unwrap();
        assert_eq!(load(&env).unwrap().total_raised, 500);
    }

    /// apply_contribution returns None on i128 overflow without mutating state.
    #[test]
    fn test_apply_contribution_overflow_returns_none() {
        let env = make_env();
        let state = CompressedState {
            total_raised: i128::MAX,
            ..default_state()
        };
        store(&env, &state);
        let result = apply_contribution(&env, 1);
        assert!(result.is_none());
        // State must be unchanged.
        assert_eq!(load(&env).unwrap().total_raised, i128::MAX);
    }

    /// apply_contribution with zero amount is a no-op (0 + 0 = 0).
    #[test]
    fn test_apply_contribution_zero_amount() {
        let env = make_env();
        store(&env, &default_state());
        let result = apply_contribution(&env, 0).unwrap();
        assert_eq!(result.total_raised, 0);
    }

    // ── apply_refund ──────────────────────────────────────────────────────────

    /// apply_refund decreases total_raised and persists.
    #[test]
    fn test_apply_refund_decreases_total_raised() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 500;
        store(&env, &s);
        let result = apply_refund(&env, 200).expect("should succeed");
        assert_eq!(result.total_raised, 300);
        assert_eq!(load(&env).unwrap().total_raised, 300);
    }

    /// apply_refund to exactly zero is allowed.
    #[test]
    fn test_apply_refund_to_zero_is_allowed() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 100;
        store(&env, &s);
        let result = apply_refund(&env, 100).unwrap();
        assert_eq!(result.total_raised, 0);
    }

    /// apply_refund returns None when it would go negative.
    #[test]
    fn test_apply_refund_below_zero_returns_none() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 50;
        store(&env, &s);
        let result = apply_refund(&env, 100);
        assert!(result.is_none());
        // State must be unchanged.
        assert_eq!(load(&env).unwrap().total_raised, 50);
    }

    /// apply_refund returns None on i128 underflow (negative amount edge case).
    #[test]
    fn test_apply_refund_underflow_returns_none() {
        let env = make_env();
        let state = CompressedState {
            total_raised: i128::MIN,
            ..default_state()
        };
        store(&env, &state);
        // Subtracting a positive value from MIN overflows checked_sub.
        let result = apply_refund(&env, 1);
        assert!(result.is_none());
    }

    // ── is_goal_reached ───────────────────────────────────────────────────────

    /// is_goal_reached returns false when no state exists.
    #[test]
    fn test_is_goal_reached_false_when_absent() {
        let env = make_env();
        assert!(!is_goal_reached(&env));
    }

    /// is_goal_reached returns false when total_raised < goal.
    #[test]
    fn test_is_goal_reached_false_below_goal() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 999;
        store(&env, &s);
        assert!(!is_goal_reached(&env));
    }

    /// is_goal_reached returns true when total_raised == goal.
    #[test]
    fn test_is_goal_reached_true_at_goal() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 1_000;
        store(&env, &s);
        assert!(is_goal_reached(&env));
    }

    /// is_goal_reached returns true when total_raised > goal (over-funded).
    #[test]
    fn test_is_goal_reached_true_over_funded() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 2_000;
        store(&env, &s);
        assert!(is_goal_reached(&env));
    }

    // ── is_expired ────────────────────────────────────────────────────────────

    /// is_expired returns false when no state exists.
    #[test]
    fn test_is_expired_false_when_absent() {
        let env = make_env();
        assert!(!is_expired(&env));
    }

    /// is_expired returns false when ledger timestamp <= deadline.
    #[test]
    fn test_is_expired_false_before_deadline() {
        let env = make_env();
        env.ledger().set_timestamp(100);
        let mut s = default_state();
        s.deadline = 200;
        store(&env, &s);
        assert!(!is_expired(&env));
    }

    /// is_expired returns false when ledger timestamp == deadline (boundary).
    #[test]
    fn test_is_expired_false_at_deadline() {
        let env = make_env();
        env.ledger().set_timestamp(200);
        let mut s = default_state();
        s.deadline = 200;
        store(&env, &s);
        assert!(!is_expired(&env));
    }

    /// is_expired returns true when ledger timestamp > deadline.
    #[test]
    fn test_is_expired_true_after_deadline() {
        let env = make_env();
        env.ledger().set_timestamp(201);
        let mut s = default_state();
        s.deadline = 200;
        store(&env, &s);
        assert!(is_expired(&env));
    }

    // ── progress_bps ──────────────────────────────────────────────────────────

    /// progress_bps returns 0 when no state exists.
    #[test]
    fn test_progress_bps_zero_when_absent() {
        let env = make_env();
        assert_eq!(progress_bps(&env), 0);
    }

    /// progress_bps returns 0 when goal is zero.
    #[test]
    fn test_progress_bps_zero_when_goal_is_zero() {
        let env = make_env();
        let s = CompressedState {
            goal: 0,
            deadline: 9_999_999,
            min_contribution: 1,
            total_raised: 500,
        };
        store(&env, &s);
        assert_eq!(progress_bps(&env), 0);
    }

    /// progress_bps returns 0 when nothing raised.
    #[test]
    fn test_progress_bps_zero_when_nothing_raised() {
        let env = make_env();
        store(&env, &default_state());
        assert_eq!(progress_bps(&env), 0);
    }

    /// progress_bps returns 5_000 at 50 % funded.
    #[test]
    fn test_progress_bps_half_funded() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 500; // 500 / 1000 = 50 %
        store(&env, &s);
        assert_eq!(progress_bps(&env), 5_000);
    }

    /// progress_bps returns 10_000 at exactly 100 % funded.
    #[test]
    fn test_progress_bps_fully_funded() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 1_000;
        store(&env, &s);
        assert_eq!(progress_bps(&env), 10_000);
    }

    /// progress_bps saturates at 10_000 when over-funded.
    #[test]
    fn test_progress_bps_saturates_at_10000_when_over_funded() {
        let env = make_env();
        let mut s = default_state();
        s.total_raised = 5_000; // 500 %
        store(&env, &s);
        assert_eq!(progress_bps(&env), 10_000);
    }

    // ── Integration: contribution → refund cycle ──────────────────────────────

    /// Full cycle: init → contribute → partial refund → check state.
    #[test]
    fn test_contribution_then_refund_cycle() {
        let env = make_env();
        store(&env, &default_state());

        apply_contribution(&env, 400).unwrap();
        apply_contribution(&env, 300).unwrap();
        assert_eq!(load(&env).unwrap().total_raised, 700);

        apply_refund(&env, 200).unwrap();
        assert_eq!(load(&env).unwrap().total_raised, 500);
        assert_eq!(progress_bps(&env), 5_000);
        assert!(!is_goal_reached(&env));
    }

    /// Contribute to goal, verify reached, then full refund.
    #[test]
    fn test_reach_goal_then_full_refund() {
        let env = make_env();
        store(&env, &default_state());

        apply_contribution(&env, 1_000).unwrap();
        assert!(is_goal_reached(&env));

        apply_refund(&env, 1_000).unwrap();
        assert!(!is_goal_reached(&env));
        assert_eq!(load(&env).unwrap().total_raised, 0);
    }
}
