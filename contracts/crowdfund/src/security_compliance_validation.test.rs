//! Comprehensive tests for `security_compliance_validation`.
//!
//! Coverage targets:
//! - Every individual validation helper.
//! - `audit_all_validations` aggregate report.
//! - Validation failure semantics for missing or invalid storage keys.
//! - Edge cases for active deadlines and platform fee limits.

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    security_compliance_validation::{
        audit_all_validations, describe_validation_result, validate_admin_present,
        validate_deadline_in_future, validate_goal_compliance,
        validate_minimum_contribution, validate_platform_fee_cap, validate_status_valid,
        ValidationReport, ValidationResult, MAX_ALLOWED_FEE_BPS,
    },
    DataKey, PlatformConfig, Status,
};

fn seed_valid_state(env: &Env) {
    let admin = Address::generate(env);
    let token = Address::generate(env);

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env.storage().instance().set(&DataKey::Goal, &1_000i128);
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &10i128);
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &(env.ledger().timestamp() + 86_400));
    env.storage().instance().set(
        &DataKey::PlatformConfig,
        &PlatformConfig {
            address: Address::generate(env),
            fee_bps: 500,
        },
    );
}

#[test]
fn test_validate_admin_present_passes_when_set() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Admin, &Address::generate(&env));
    assert_eq!(validate_admin_present(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_admin_present_fails_when_missing() {
    let env = Env::default();
    assert_eq!(
        validate_admin_present(&env),
        ValidationResult::Invalid("admin is not initialized")
    );
}

#[test]
fn test_validate_status_valid_passes_when_status_present() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    assert_eq!(validate_status_valid(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_status_valid_fails_when_missing() {
    let env = Env::default();
    assert_eq!(
        validate_status_valid(&env),
        ValidationResult::Invalid("campaign status is missing")
    );
}

#[test]
fn test_validate_goal_compliance_fails_for_zero_goal() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Goal, &0i128);
    assert_eq!(
        validate_goal_compliance(&env),
        ValidationResult::Invalid("campaign goal is zero or negative")
    );
}

#[test]
fn test_validate_goal_compliance_passes_for_positive_goal() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::Goal, &1_000i128);
    assert_eq!(validate_goal_compliance(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_minimum_contribution_fails_for_zero() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::MinContribution, &0i128);
    assert_eq!(
        validate_minimum_contribution(&env),
        ValidationResult::Invalid("minimum contribution is below the compliant floor")
    );
}

#[test]
fn test_validate_minimum_contribution_passes_for_positive_value() {
    let env = Env::default();
    env.storage().instance().set(&DataKey::MinContribution, &5i128);
    assert_eq!(validate_minimum_contribution(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_deadline_in_future_passes_for_active_future_deadline() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &(env.ledger().timestamp() + 1_000));
    assert_eq!(validate_deadline_in_future(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_deadline_in_future_fails_for_expired_active_campaign() {
    let env = Env::default();
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &(env.ledger().timestamp() - 1));
    assert_eq!(
        validate_deadline_in_future(&env),
        ValidationResult::Invalid("active campaign deadline has passed")
    );
}

#[test]
fn test_validate_platform_fee_cap_passes_when_missing_config() {
    let env = Env::default();
    assert_eq!(validate_platform_fee_cap(&env), ValidationResult::Valid);
}

#[test]
fn test_validate_platform_fee_cap_fails_when_above_limit() {
    let env = Env::default();
    env.storage().instance().set(
        &DataKey::PlatformConfig,
        &PlatformConfig {
            address: Address::generate(&env),
            fee_bps: MAX_ALLOWED_FEE_BPS + 1,
        },
    );
    assert_eq!(
        validate_platform_fee_cap(&env),
        ValidationResult::Invalid("platform fee exceeds the allowed maximum")
    );
}

#[test]
fn test_describe_validation_result_returns_valid_label() {
    assert_eq!(
        describe_validation_result(&ValidationResult::Valid),
        "VALID"
    );
}

#[test]
fn test_describe_validation_result_returns_message_for_invalid() {
    assert_eq!(
        describe_validation_result(&ValidationResult::Invalid("bad config")),
        "bad config"
    );
}

#[test]
fn test_audit_all_validations_reports_all_valid_for_compliant_state() {
    let env = Env::default();
    seed_valid_state(&env);
    let report = audit_all_validations(&env);
    assert_eq!(report.all_valid, true);
    assert_eq!(report.passed, 6);
    assert_eq!(report.failed, 0);
}

#[test]
fn test_audit_all_validations_reports_failure_when_any_validation_fails() {
    let env = Env::default();
    seed_valid_state(&env);
    env.storage().instance().set(&DataKey::Goal, &0i128);
    let report = audit_all_validations(&env);
    assert_eq!(report.all_valid, false);
    assert_eq!(report.failed, 1);
    assert_eq!(report.passed, 5);
}
