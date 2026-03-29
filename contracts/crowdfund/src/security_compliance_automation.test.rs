//! Comprehensive tests for `security_compliance_automation`.
//!
//! Coverage targets:
//! - Every individual check function (happy path + failure path).
//! - `audit_all_checks` aggregate report.
//! - `audit_initialization` and `audit_financial_integrity` targeted helpers.
//! - `describe_check_result` and `CheckResult` helpers.
//! - Edge cases: zero goal, zero min_contribution, negative total_raised,
//!   fee at boundary, fee above boundary, missing keys.

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    security_compliance_automation::{
        audit_all_checks, audit_financial_integrity, audit_initialization, check_admin_initialized,
        check_creator_address_set, check_deadline_in_future, check_goal_positive,
        check_min_contribution_positive, check_paused_flag_present,
        check_platform_fee_within_limit, check_status_valid, check_token_address_set,
        check_total_raised_non_negative, describe_check_result, CheckResult,
        MAX_ALLOWED_FEE_BPS, MIN_COMPLIANT_CONTRIBUTION, MIN_COMPLIANT_GOAL,
    },
    DataKey, PlatformConfig, Status,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Seed a fully-initialized, compliant contract state into `env`.
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
    // Deadline 1 hour in the future.
    let deadline = env.ledger().timestamp() + 3_600;
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &deadline);
    env.storage()
        .instance()
        .set(&DataKey::TotalRaised, &0i128);
    env.storage().instance().set(&DataKey::Paused, &false);

    admin
}

// ── CheckResult helpers ───────────────────────────────────────────────────────

#[test]
fn test_check_result_passed_is_passed() {
    assert!(CheckResult::Passed.is_passed());
}

#[test]
fn test_check_result_failed_is_not_passed() {
    assert!(!CheckResult::Failed("oops").is_passed());
}

#[test]
fn test_check_result_passed_violation_is_empty() {
    assert_eq!(CheckResult::Passed.violation(), "");
}

#[test]
fn test_check_result_failed_violation_returns_message() {
    assert_eq!(CheckResult::Failed("bad state").violation(), "bad state");
}

#[test]
fn test_describe_check_result_passed() {
    assert_eq!(describe_check_result(&CheckResult::Passed), "PASSED");
}

#[test]
fn test_describe_check_result_failed() {
    let msg = "something wrong";
    assert_eq!(
        describe_check_result(&CheckResult::Failed(msg)),
        "something wrong"
    );
}

// ── check_admin_initialized ───────────────────────────────────────────────────

#[test]
fn test_admin_initialized_passes_when_set() {
    let env = Env::default();
    let admin = Address::generate(&env);
    env.storage().instance().set(&DataKey::Admin, &admin);
    assert!(check_admin_initialized(&env).is_passed());
}

#[test]
fn test_admin_initialized_fails_when_missing() {
    let env = Env::default();
    let result = check_admin_initialized(&env);
    assert!(!result.is_passed());
    assert!(!result.violation().is_empty());
}

// ── check_creator_address_set ─────────────────────────────────────────────────

#[test]
fn test_creator_set_passes_when_stored() {
    let env = Env::default();
    let creator = Address::generate(&env);
    env.storage().instance().set(&DataKey::Creator, &creator);
    assert!(check_creator_address_set(&env).is_passed());
}

#[test]
fn test_creator_set_fails_when_missing() {
    let env = Env::default();
    assert!(!check_creator_address_set(&env).is_passed());
}

// ── check_token_address_set ───────────────────────────────────────────────────

#[test]
fn test_token_set_passes_when_stored() {
    let env = Env::default();
    let token = Address::generate(&env);
    env.storage().instance().set(&DataKey::Token, &token);
    assert!(check_token_address_set(&env).is_passed());
}

#[test]
fn test_token_set_fails_when_missing() {
    let env = Env::default();
    assert!(!check_token_address_set(&env).is_passed());
}

// ── check_status_valid ────────────────────────────────────────────────────────

#[test]
fn test_status_valid_passes_for_active() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    assert!(check_status_valid(&env).is_passed());
}

#[test]
fn test_status_valid_passes_for_succeeded() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Succeeded);
    assert!(check_status_valid(&env).is_passed());
}

#[test]
fn test_status_valid_passes_for_expired() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Expired);
    assert!(check_status_valid(&env).is_passed());
}

#[test]
fn test_status_valid_passes_for_cancelled() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Cancelled);
    assert!(check_status_valid(&env).is_passed());
}

#[test]
fn test_status_valid_fails_when_missing() {
    let env = Env::default();
    assert!(!check_status_valid(&env).is_passed());
}

// ── check_goal_positive ───────────────────────────────────────────────────────

#[test]
fn test_goal_positive_passes_for_one() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Goal, &MIN_COMPLIANT_GOAL);
    assert!(check_goal_positive(&env).is_passed());
}

#[test]
fn test_goal_positive_passes_for_large_value() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Goal, &1_000_000i128);
    assert!(check_goal_positive(&env).is_passed());
}

#[test]
fn test_goal_positive_fails_for_zero() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Goal, &0i128);
    let result = check_goal_positive(&env);
    assert!(!result.is_passed());
}

#[test]
fn test_goal_positive_fails_for_negative() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Goal, &(-1i128));
    assert!(!check_goal_positive(&env).is_passed());
}

#[test]
fn test_goal_positive_fails_when_missing() {
    let env = Env::default();
    assert!(!check_goal_positive(&env).is_passed());
}

// ── check_min_contribution_positive ──────────────────────────────────────────

#[test]
fn test_min_contribution_passes_for_one() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &MIN_COMPLIANT_CONTRIBUTION);
    assert!(check_min_contribution_positive(&env).is_passed());
}

#[test]
fn test_min_contribution_fails_for_zero() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &0i128);
    assert!(!check_min_contribution_positive(&env).is_passed());
}

#[test]
fn test_min_contribution_fails_for_negative() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &(-5i128));
    assert!(!check_min_contribution_positive(&env).is_passed());
}

#[test]
fn test_min_contribution_fails_when_missing() {
    let env = Env::default();
    assert!(!check_min_contribution_positive(&env).is_passed());
}

// ── check_deadline_in_future ──────────────────────────────────────────────────

#[test]
fn test_deadline_passes_when_in_future() {
    let env = Env::default();
    let future = env.ledger().timestamp() + 3_600;
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &future);
    assert!(check_deadline_in_future(&env).is_passed());
}

#[test]
fn test_deadline_fails_when_in_past() {
    let env = Env::default();
    // Ledger timestamp starts at 0 in tests; set deadline to 0 (already passed).
    env.storage().instance().set(&DataKey::Deadline, &0u64);
    assert!(!check_deadline_in_future(&env).is_passed());
}

#[test]
fn test_deadline_fails_when_equal_to_now() {
    let env = Env::default();
    let now = env.ledger().timestamp();
    env.storage().instance().set(&DataKey::Deadline, &now);
    assert!(!check_deadline_in_future(&env).is_passed());
}

#[test]
fn test_deadline_fails_when_missing() {
    let env = Env::default();
    assert!(!check_deadline_in_future(&env).is_passed());
}

// ── check_total_raised_non_negative ──────────────────────────────────────────

#[test]
fn test_total_raised_passes_for_zero() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::TotalRaised, &0i128);
    assert!(check_total_raised_non_negative(&env).is_passed());
}

#[test]
fn test_total_raised_passes_for_positive() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::TotalRaised, &500i128);
    assert!(check_total_raised_non_negative(&env).is_passed());
}

#[test]
fn test_total_raised_fails_for_negative() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::TotalRaised, &(-1i128));
    let result = check_total_raised_non_negative(&env);
    assert!(!result.is_passed());
    assert!(!result.violation().is_empty());
}

#[test]
fn test_total_raised_passes_when_missing_defaults_to_zero() {
    // When the key is absent, the check defaults to 0 which is non-negative.
    let env = Env::default();
    assert!(check_total_raised_non_negative(&env).is_passed());
}

// ── check_platform_fee_within_limit ──────────────────────────────────────────

#[test]
fn test_platform_fee_passes_when_no_config() {
    let env = Env::default();
    assert!(check_platform_fee_within_limit(&env).is_passed());
}

#[test]
fn test_platform_fee_passes_at_zero() {
    let env = Env::default();
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 0,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    assert!(check_platform_fee_within_limit(&env).is_passed());
}

#[test]
fn test_platform_fee_passes_at_boundary() {
    let env = Env::default();
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: MAX_ALLOWED_FEE_BPS,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    assert!(check_platform_fee_within_limit(&env).is_passed());
}

#[test]
fn test_platform_fee_fails_above_boundary() {
    let env = Env::default();
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: MAX_ALLOWED_FEE_BPS + 1,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    let result = check_platform_fee_within_limit(&env);
    assert!(!result.is_passed());
    assert!(!result.violation().is_empty());
}

#[test]
fn test_platform_fee_fails_at_max_u32() {
    let env = Env::default();
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: u32::MAX,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    assert!(!check_platform_fee_within_limit(&env).is_passed());
}

// ── check_paused_flag_present ─────────────────────────────────────────────────

#[test]
fn test_paused_flag_passes_when_false() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Paused, &false);
    assert!(check_paused_flag_present(&env).is_passed());
}

#[test]
fn test_paused_flag_passes_when_true() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Paused, &true);
    assert!(check_paused_flag_present(&env).is_passed());
}

#[test]
fn test_paused_flag_fails_when_missing() {
    let env = Env::default();
    let result = check_paused_flag_present(&env);
    assert!(!result.is_passed());
    assert!(!result.violation().is_empty());
}

// ── audit_all_checks ──────────────────────────────────────────────────────────

#[test]
fn test_audit_all_checks_passes_on_compliant_state() {
    let env = Env::default();
    seed_compliant_state(&env);
    let report = audit_all_checks(&env);
    assert!(report.all_passed, "expected all checks to pass");
    assert_eq!(report.failed, 0);
    assert_eq!(report.passed, 10);
}

#[test]
fn test_audit_all_checks_fails_on_empty_state() {
    let env = Env::default();
    let report = audit_all_checks(&env);
    assert!(!report.all_passed);
    assert!(report.failed > 0);
}

#[test]
fn test_audit_all_checks_counts_are_consistent() {
    let env = Env::default();
    seed_compliant_state(&env);
    // Corrupt one field to introduce exactly one failure.
    env.storage().instance().set(&DataKey::Goal, &0i128);
    let report = audit_all_checks(&env);
    assert_eq!(report.passed + report.failed, 10);
    assert_eq!(report.failed, 1);
}

#[test]
fn test_audit_all_checks_multiple_failures() {
    let env = Env::default();
    // Only seed admin — everything else is missing.
    let admin = Address::generate(&env);
    env.storage().instance().set(&DataKey::Admin, &admin);
    let report = audit_all_checks(&env);
    assert!(report.failed >= 2);
}

// ── audit_initialization ──────────────────────────────────────────────────────

#[test]
fn test_audit_initialization_passes_on_compliant_state() {
    let env = Env::default();
    seed_compliant_state(&env);
    assert!(audit_initialization(&env));
}

#[test]
fn test_audit_initialization_fails_when_admin_missing() {
    let env = Env::default();
    seed_compliant_state(&env);
    // Remove admin key to simulate missing initialization.
    // We can't remove a key directly, so we test with a fresh env.
    let env2 = Env::default();
    let creator = Address::generate(&env2);
    let token = Address::generate(&env2);
    env2.storage().instance().set(&DataKey::Creator, &creator);
    env2.storage().instance().set(&DataKey::Token, &token);
    env2.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env2.storage().instance().set(&DataKey::Goal, &1_000i128);
    env2.storage()
        .instance()
        .set(&DataKey::MinContribution, &10i128);
    // Admin is NOT set.
    assert!(!audit_initialization(&env2));
}

#[test]
fn test_audit_initialization_fails_when_goal_zero() {
    let env = Env::default();
    seed_compliant_state(&env);
    // Override goal with zero in a fresh env.
    let env2 = Env::default();
    let admin = Address::generate(&env2);
    let creator = Address::generate(&env2);
    let token = Address::generate(&env2);
    env2.storage().instance().set(&DataKey::Admin, &admin);
    env2.storage().instance().set(&DataKey::Creator, &creator);
    env2.storage().instance().set(&DataKey::Token, &token);
    env2.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env2.storage().instance().set(&DataKey::Goal, &0i128); // invalid
    env2.storage()
        .instance()
        .set(&DataKey::MinContribution, &10i128);
    assert!(!audit_initialization(&env2));
}

// ── audit_financial_integrity ─────────────────────────────────────────────────

#[test]
fn test_audit_financial_integrity_passes_on_compliant_state() {
    let env = Env::default();
    seed_compliant_state(&env);
    assert!(audit_financial_integrity(&env));
}

#[test]
fn test_audit_financial_integrity_fails_on_negative_total() {
    let env = Env::default();
    seed_compliant_state(&env);
    // Corrupt total_raised.
    let env2 = Env::default();
    let admin = Address::generate(&env2);
    env2.storage().instance().set(&DataKey::Admin, &admin);
    env2.storage().instance().set(&DataKey::Goal, &1_000i128);
    env2.storage()
        .instance()
        .set(&DataKey::TotalRaised, &(-100i128));
    assert!(!audit_financial_integrity(&env2));
}

#[test]
fn test_audit_financial_integrity_fails_on_excessive_fee() {
    let env = Env::default();
    seed_compliant_state(&env);
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: MAX_ALLOWED_FEE_BPS + 1,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    assert!(!audit_financial_integrity(&env));
}

#[test]
fn test_audit_financial_integrity_passes_with_fee_at_boundary() {
    let env = Env::default();
    seed_compliant_state(&env);
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: MAX_ALLOWED_FEE_BPS,
    };
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);
    assert!(audit_financial_integrity(&env));
}

// ── Constants sanity ──────────────────────────────────────────────────────────

#[test]
fn test_constants_have_expected_values() {
    assert_eq!(MAX_ALLOWED_FEE_BPS, 1_000);
    assert_eq!(MIN_COMPLIANT_GOAL, 1);
    assert_eq!(MIN_COMPLIANT_CONTRIBUTION, 1);
}
