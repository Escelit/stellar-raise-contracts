//! Tests for `soroban_sdk_minor` helpers.
//!
//! Covers every public function with normal, boundary, and edge-case inputs
//! to achieve ≥ 95 % line coverage.

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    use crate::soroban_sdk_minor::{
        assess_compatibility, emit_upgrade_audit_event, validate_wasm_hash, CompatibilityStatus,
    };

    // ── assess_compatibility ─────────────────────────────────────────────────

    #[test]
    fn test_same_major_is_compatible() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "22.1.0"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_same_version_is_compatible() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "22.0.0"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_different_major_requires_migration() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "21.0.0", "22.0.0"),
            CompatibilityStatus::RequiresMigration
        );
    }

    #[test]
    fn test_unparseable_version_treated_as_zero_major() {
        let env = Env::default();
        // Both unparseable → both major == 0 → Compatible
        assert_eq!(
            assess_compatibility(&env, "invalid", "also-invalid"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_one_unparseable_version_requires_migration() {
        let env = Env::default();
        // "22.0.0" → major 22; "invalid" → major 0 → RequiresMigration
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "invalid"),
            CompatibilityStatus::RequiresMigration
        );
    }

    // ── validate_wasm_hash ───────────────────────────────────────────────────

    #[test]
    fn test_zero_hash_is_invalid() {
        let env = Env::default();
        let zero: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);
        assert!(!validate_wasm_hash(&zero));
    }

    #[test]
    fn test_nonzero_hash_is_valid() {
        let env = Env::default();
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        let hash: BytesN<32> = BytesN::from_array(&env, &bytes);
        assert!(validate_wasm_hash(&hash));
    }

    #[test]
    fn test_all_ones_hash_is_valid() {
        let env = Env::default();
        let hash: BytesN<32> = BytesN::from_array(&env, &[0xFF; 32]);
        assert!(validate_wasm_hash(&hash));
    }

    // ── emit_upgrade_audit_event ─────────────────────────────────────────────

    #[test]
    fn test_emit_upgrade_audit_event_does_not_panic() {
        let env = Env::default();
        let reviewer = Address::generate(&env);
        // Should emit without panicking; event ledger is observable in tests.
        emit_upgrade_audit_event(
            &env,
            String::from_str(&env, "22.0.0"),
            String::from_str(&env, "22.1.0"),
            reviewer,
        );
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::{
        soroban_sdk_minor::{
            compute_fee, get_contribution, instance_get_or, is_active_window, is_past_deadline,
            persistent_get_or, progress_bps, set_contribution,
        },
        CrowdfundContract, DataKey,
    };

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Minimal env with the crowdfund contract registered.
    fn make_env() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(CrowdfundContract, ());
        (env, contract_id)
    }

    // ── progress_bps ─────────────────────────────────────────────────────────

    #[test]
    fn test_progress_bps_zero_goal() {
        assert_eq!(progress_bps(0, 0), 0);
        assert_eq!(progress_bps(1_000, 0), 0);
    }

    #[test]
    fn test_progress_bps_half() {
        assert_eq!(progress_bps(500_000, 1_000_000), 5_000);
    }

    #[test]
    fn test_progress_bps_exact_goal() {
        assert_eq!(progress_bps(1_000_000, 1_000_000), 10_000);
    }

    #[test]
    fn test_progress_bps_over_goal_capped() {
        // Exceeding the goal must be capped at 10 000.
        assert_eq!(progress_bps(2_000_000, 1_000_000), 10_000);
    }

    #[test]
    fn test_progress_bps_zero_raised() {
        assert_eq!(progress_bps(0, 1_000_000), 0);
    }

    #[test]
    fn test_progress_bps_one_bps() {
        // 1 / 10_000 of the goal → exactly 1 bps.
        assert_eq!(progress_bps(100, 1_000_000), 1);
    }

    // ── compute_fee ──────────────────────────────────────────────────────────

    #[test]
    fn test_compute_fee_zero_bps() {
        assert_eq!(compute_fee(1_000_000, 0), 0);
    }

    #[test]
    fn test_compute_fee_full_bps() {
        // 10 000 bps = 100 % → fee equals total.
        assert_eq!(compute_fee(1_000_000, 10_000), 1_000_000);
    }

    #[test]
    fn test_compute_fee_250_bps() {
        // 2.5 % of 1_000_000 = 25_000.
        assert_eq!(compute_fee(1_000_000, 250), 25_000);
    }

    #[test]
    fn test_compute_fee_rounds_down() {
        // 1 bps of 999 = 0 (integer division floors).
        assert_eq!(compute_fee(999, 1), 0);
    }

    #[test]
    fn test_compute_fee_large_amount() {
        // Verify no overflow for large but realistic token amounts.
        let total: i128 = 1_000_000_000_000; // 1 trillion stroops
        let fee = compute_fee(total, 500); // 5 %
        assert_eq!(fee, 50_000_000_000);
    }

    // ── is_past_deadline / is_active_window ──────────────────────────────────

    #[test]
    fn test_is_past_deadline_no_deadline_set() {
        let (env, contract_id) = make_env();
        // No deadline stored → defaults to 0; current timestamp (also 0) is NOT > 0.
        env.as_contract(&contract_id, || {
            assert!(!is_past_deadline(&env));
        });
    }

    #[test]
    fn test_is_past_deadline_future_deadline() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let future: u64 = env.ledger().timestamp() + 3_600;
            env.storage().instance().set(&DataKey::Deadline, &future);
            assert!(!is_past_deadline(&env));
            assert!(is_active_window(&env));
        });
    }

    #[test]
    fn test_is_past_deadline_past_deadline() {
        use soroban_sdk::testutils::Ledger;
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let deadline: u64 = env.ledger().timestamp() + 100;
            env.storage().instance().set(&DataKey::Deadline, &deadline);
            // Advance ledger past the deadline.
            env.ledger().set_timestamp(deadline + 1);
            assert!(is_past_deadline(&env));
            assert!(!is_active_window(&env));
        });
    }

    // ── get_contribution / set_contribution ──────────────────────────────────

    #[test]
    fn test_get_contribution_absent_returns_zero() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            assert_eq!(get_contribution(&env, &addr), 0);
        });
    }

    #[test]
    fn test_set_and_get_contribution() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            set_contribution(&env, &addr, 500_000, 100);
            assert_eq!(get_contribution(&env, &addr), 500_000);
        });
    }

    #[test]
    fn test_set_contribution_overwrites_previous() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            set_contribution(&env, &addr, 100_000, 100);
            set_contribution(&env, &addr, 250_000, 100);
            assert_eq!(get_contribution(&env, &addr), 250_000);
        });
    }

    #[test]
    fn test_set_contribution_zero() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            set_contribution(&env, &addr, 100_000, 100);
            set_contribution(&env, &addr, 0, 100);
            assert_eq!(get_contribution(&env, &addr), 0);
        });
    }

    // ── instance_get_or / persistent_get_or ──────────────────────────────────

    #[test]
    fn test_instance_get_or_returns_default_when_absent() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let val: i128 = instance_get_or(&env, &DataKey::TotalRaised, 42i128);
            assert_eq!(val, 42);
        });
    }

    #[test]
    fn test_instance_get_or_returns_stored_value() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .set(&DataKey::TotalRaised, &999i128);
            let val: i128 = instance_get_or(&env, &DataKey::TotalRaised, 0i128);
            assert_eq!(val, 999);
        });
    }

    #[test]
    fn test_persistent_get_or_returns_default_when_absent() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            let key = DataKey::Contribution(addr);
            let val: i128 = persistent_get_or(&env, &key, -1i128);
            assert_eq!(val, -1);
        });
    }

    #[test]
    fn test_persistent_get_or_returns_stored_value() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let addr = Address::generate(&env);
            let key = DataKey::Contribution(addr);
            env.storage().persistent().set(&key, &777i128);
            let val: i128 = persistent_get_or(&env, &key, 0i128);
            assert_eq!(val, 777);
        });
    }
}
