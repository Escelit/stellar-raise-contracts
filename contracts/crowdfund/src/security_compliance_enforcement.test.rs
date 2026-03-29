//! Unit tests for `security_compliance_enforcement`.
//!
//! Coverage:
//! - `assert_compliant`: happy path, missing Admin/Token/Status, no-auth requirement
//! - `enforce_compliance`: all-passed, critical halt, non-critical-only path
//! - `trigger_emergency_halt` / `lift_emergency_halt`: set/clear Paused, idempotency, Pauser panic
//! - `remediate_platform_fee`: valid update, boundary, invalid values, no config, address preserved
//! - `remediate_min_contribution`: valid update, zero, negative
//! - `compliance_status`: compliant, halted, violation counts
//! - `EnforcementError` discriminant values

use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

use crate::{
    security_compliance_enforcement::{
        assert_compliant, compliance_status, enforce_compliance, lift_emergency_halt,
        remediate_min_contribution, remediate_platform_fee, trigger_emergency_halt,
        EnforcementError, MAX_ALLOWED_FEE_BPS,
    },
    DataKey, PlatformConfig, Status,
};

// ── Minimal contract for storage access ──────────────────────────────────────

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Create a fresh env with a registered test contract.
fn make_env() -> (Env, Address) {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    (env, contract_id)
}

/// Seed a fully-initialized, compliant contract state.
/// Returns the admin address.
fn seed_compliant_state(env: &Env) -> Address {
    let admin = Address::generate(env);
    let creator = Address::generate(env);
    let token = Address::generate(env);

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Creator, &creator);
    env.storage().instance().set(&DataKey::Token, &token);
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env.storage().instance().set(&DataKey::Goal, &1_000i128);
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &10i128);
    let deadline = env.ledger().timestamp() + 3_600;
    env.storage().instance().set(&DataKey::Deadline, &deadline);
    env.storage().instance().set(&DataKey::TotalRaised, &0i128);
    env.storage().instance().set(&DataKey::Paused, &false);

    admin
}

// ── Task 10.1: assert_compliant ───────────────────────────────────────────────

#[test]
fn test_assert_compliant_passes_on_compliant_contract() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        seed_compliant_state(&env);
        let result = assert_compliant(&env);
        assert_eq!(result, Ok(()));
    });
}

#[test]
fn test_assert_compliant_fails_when_admin_absent() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed everything except Admin
        let token = Address::generate(&env);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        let result = assert_compliant(&env);
        assert_eq!(result, Err(EnforcementError::CriticalViolationDetected));
    });
}

#[test]
fn test_assert_compliant_fails_when_token_absent() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed everything except Token
        let admin = Address::generate(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        let result = assert_compliant(&env);
        assert_eq!(result, Err(EnforcementError::CriticalViolationDetected));
    });
}

#[test]
fn test_assert_compliant_fails_when_status_absent() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed Admin and Token but not Status
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        let result = assert_compliant(&env);
        assert_eq!(result, Err(EnforcementError::CriticalViolationDetected));
    });
}

#[test]
fn test_assert_compliant_no_auth_required() {
    // assert_compliant is permissionless — must not panic without mock auth
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        seed_compliant_state(&env);
        // No env.mock_all_auths() — should not panic
        let result = assert_compliant(&env);
        assert_eq!(result, Ok(()));
    });
}

// ── Task 10.2: enforce_compliance ─────────────────────────────────────────────

#[test]
fn test_enforce_compliance_passes_on_compliant_contract() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        seed_compliant_state(&env);
        let result = enforce_compliance(&env);
        assert!(result.is_ok(), "expected Ok(report) on compliant contract");
        let report = result.unwrap();
        assert!(report.all_passed, "expected all_passed == true");
        assert_eq!(report.failed, 0);
        assert_eq!(report.passed, 10);
    });
}

#[test]
fn test_enforce_compliance_halts_on_critical_violation() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed state with Admin absent (critical violation)
        let token = Address::generate(&env);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        let result = enforce_compliance(&env);
        assert!(
            matches!(result, Err(EnforcementError::CriticalViolationDetected)),
            "expected CriticalViolationDetected"
        );

        // Paused must be set to true
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        assert!(paused, "contract should be paused after critical violation");
    });
}

#[test]
fn test_enforce_compliance_non_critical_only() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed all critical keys but introduce non-critical violations:
        // missing Creator, Goal, MinContribution, Deadline, Paused (all non-critical)
        let admin = Address::generate(&env);
        let token = Address::generate(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        // Intentionally omit: Creator, Goal, MinContribution, Deadline, Paused

        let result = enforce_compliance(&env);
        assert!(
            matches!(result, Err(EnforcementError::NonCriticalViolationsPresent)),
            "expected NonCriticalViolationsPresent, got {:?}",
            result.err()
        );

        // Paused must NOT have been set to true
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        assert!(
            !paused,
            "contract should NOT be paused for non-critical violations"
        );
    });
}

// ── Task 10.3: trigger_emergency_halt / lift_emergency_halt ───────────────────

#[test]
fn test_trigger_halt_sets_paused_true() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        trigger_emergency_halt(&env, admin);
    });

    let paused: bool = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    });
    assert!(paused, "Paused should be true after trigger_emergency_halt");
}

#[test]
fn test_lift_halt_sets_paused_false() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        // Pre-set paused to true
        env.storage().instance().set(&DataKey::Paused, &true);
        a
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        lift_emergency_halt(&env, admin);
    });

    let paused: bool = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(true)
    });
    assert!(!paused, "Paused should be false after lift_emergency_halt");
}

#[test]
fn test_trigger_halt_idempotent() {
    let (env, contract_id) = make_env();

    // Seed state outside as_contract (just need the admin address)
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));

    // First call
    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        trigger_emergency_halt(&env, admin.clone());
    });

    // Second call — idempotent
    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        trigger_emergency_halt(&env, admin);
    });

    let paused: bool = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    });
    assert!(
        paused,
        "Paused should still be true after second trigger_emergency_halt"
    );
}

#[test]
#[should_panic]
fn test_lift_halt_panics_for_pauser() {
    let (env, contract_id) = make_env();

    let (admin, pauser) = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let p = Address::generate(&env);
        env.storage().instance().set(&DataKey::Pauser, &p);
        (a, p)
    });

    // Halt first
    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        trigger_emergency_halt(&env, admin);
    });

    // Pauser tries to lift — should panic
    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        lift_emergency_halt(&env, pauser);
    });
}

// ── Task 10.4: remediate_platform_fee ─────────────────────────────────────────

#[test]
fn test_remediate_fee_updates_fee() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let platform_addr = Address::generate(&env);
        let config = PlatformConfig {
            address: platform_addr,
            fee_bps: 500,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, &config);
        a
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_platform_fee(&env, admin, 200);
        assert_eq!(result, Ok(()));

        let stored: PlatformConfig = env
            .storage()
            .instance()
            .get(&DataKey::PlatformConfig)
            .unwrap();
        assert_eq!(stored.fee_bps, 200);
    });
}

#[test]
fn test_remediate_fee_invalid_1001() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let config = PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 500,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, &config);
        a
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_platform_fee(&env, admin, MAX_ALLOWED_FEE_BPS + 1);
        assert_eq!(result, Err(EnforcementError::RemediationValueInvalid));
    });
}

#[test]
fn test_remediate_fee_invalid_max() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let config = PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 500,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, &config);
        a
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_platform_fee(&env, admin, u32::MAX);
        assert_eq!(result, Err(EnforcementError::RemediationValueInvalid));
    });
}

#[test]
fn test_remediate_fee_boundary_1000() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let config = PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 500,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, &config);
        a
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        // Exactly MAX_ALLOWED_FEE_BPS (1000) should be valid
        let result = remediate_platform_fee(&env, admin, MAX_ALLOWED_FEE_BPS);
        assert_eq!(result, Ok(()));

        let stored: PlatformConfig = env
            .storage()
            .instance()
            .get(&DataKey::PlatformConfig)
            .unwrap();
        assert_eq!(stored.fee_bps, MAX_ALLOWED_FEE_BPS);
    });
}

#[test]
fn test_remediate_fee_no_config() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));
    // No PlatformConfig stored

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_platform_fee(&env, admin, 100);
        assert_eq!(result, Err(EnforcementError::NoPlatformConfigToRemediate));
    });
}

#[test]
fn test_remediate_fee_preserves_address() {
    let (env, contract_id) = make_env();
    let (admin, platform_addr) = env.as_contract(&contract_id, || {
        let a = seed_compliant_state(&env);
        let platform_addr = Address::generate(&env);
        let config = PlatformConfig {
            address: platform_addr.clone(),
            fee_bps: 500,
        };
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, &config);
        (a, platform_addr)
    });

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        remediate_platform_fee(&env, admin, 300).unwrap();

        let stored: PlatformConfig = env
            .storage()
            .instance()
            .get(&DataKey::PlatformConfig)
            .unwrap();
        assert_eq!(
            stored.address, platform_addr,
            "address field must be preserved"
        );
        assert_eq!(stored.fee_bps, 300);
    });
}

// ── Task 10.5: remediate_min_contribution ─────────────────────────────────────

#[test]
fn test_remediate_min_updates() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_min_contribution(&env, admin, 50);
        assert_eq!(result, Ok(()));

        let stored: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        assert_eq!(stored, 50);
    });
}

#[test]
fn test_remediate_min_zero() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_min_contribution(&env, admin, 0);
        assert_eq!(result, Err(EnforcementError::RemediationValueInvalid));
    });
}

#[test]
fn test_remediate_min_negative() {
    let (env, contract_id) = make_env();
    let admin = env.as_contract(&contract_id, || seed_compliant_state(&env));

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        let result = remediate_min_contribution(&env, admin, -1);
        assert_eq!(result, Err(EnforcementError::RemediationValueInvalid));
    });
}

// ── Task 10.6: compliance_status ──────────────────────────────────────────────

#[test]
fn test_compliance_status_compliant() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        seed_compliant_state(&env);

        let status = compliance_status(&env);
        assert!(!status.is_halted);
        assert!(!status.has_critical_violations);
        assert!(!status.has_non_critical_violations);
        assert_eq!(status.critical_violation_count, 0);
        assert_eq!(status.non_critical_violation_count, 0);
    });
}

#[test]
fn test_compliance_status_halted() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        seed_compliant_state(&env);
        env.storage().instance().set(&DataKey::Paused, &true);

        let status = compliance_status(&env);
        assert!(
            status.is_halted,
            "is_halted should be true when Paused == true"
        );
    });
}

#[test]
fn test_compliance_status_violation_counts() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        // Seed Token, Status, TotalRaised but omit Admin → 1 critical violation
        let token = Address::generate(&env);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        // Admin is absent → 1 critical violation
        // Creator, Goal, MinContribution, Deadline, Paused absent → non-critical violations

        let status = compliance_status(&env);
        assert!(status.has_critical_violations);
        assert!(status.critical_violation_count >= 1);
        assert!(status.has_non_critical_violations);
        assert!(status.non_critical_violation_count >= 1);
    });
}

// ── Task 10.7: EnforcementError discriminant values ───────────────────────────

#[test]
fn test_enforcement_error_discriminants() {
    assert_eq!(EnforcementError::CriticalViolationDetected as u32, 1);
    assert_eq!(EnforcementError::NonCriticalViolationsPresent as u32, 2);
    assert_eq!(EnforcementError::RemediationValueInvalid as u32, 3);
    assert_eq!(EnforcementError::NoPlatformConfigToRemediate as u32, 4);
    assert_eq!(EnforcementError::Unauthorized as u32, 5);
}
