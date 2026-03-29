//! # security_compliance_validation
//!
//! @title   SecurityComplianceValidation — on-chain validation helpers for
//!          automated testing and compliance verification.
//!
//! @notice  Exposes read-only compliance assertions that CI and test
//!          automation can call to verify contract configuration and state
//!          before executing scenario tests or contract upgrades.
//!
//! @dev     All exported helpers are permissionless and read-only. They do not
//!          mutate storage, require auth, or depend on unbounded iteration.
//!
//! # Security Assumptions
//!
//! 1. Read-only — no storage writes.
//! 2. Permissionless — no auth required.
//! 3. Deterministic — same ledger state yields same result.
//! 4. Bounded execution — only fixed checks are executed.

#![allow(dead_code)]

use soroban_sdk::Env;

use crate::{DataKey, PlatformConfig, Status};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum compliant platform fee in basis points (10 %).
pub const MAX_ALLOWED_FEE_BPS: u32 = 1_000;

/// Minimum compliant campaign goal in token units.
pub const MIN_COMPLIANT_GOAL: i128 = 1;

/// Minimum compliant contribution amount in token units.
pub const MIN_COMPLIANT_CONTRIBUTION: i128 = 1;

// ── Result types ──────────────────────────────────────────────────────────────

/// Validation outcome for a single contract invariant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ValidationResult {
    /// The invariant held.
    Valid,
    /// The invariant failed with a static message.
    Invalid(&'static str),
}

impl ValidationResult {
    /// Returns true when the invariant is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    /// Returns the static failure message or an empty string for valid results.
    pub fn message(&self) -> &'static str {
        match self {
            ValidationResult::Valid => "",
            ValidationResult::Invalid(msg) => msg,
        }
    }
}

/// Aggregated validation report for multiple checks.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ValidationReport {
    pub passed: u32,
    pub failed: u32,
    pub all_valid: bool,
}

impl ValidationReport {
    /// Builds a report from an array of validation results.
    pub fn from_results(results: &[ValidationResult]) -> ValidationReport {
        let mut passed = 0u32;
        for result in results {
            if result.is_valid() {
                passed += 1;
            }
        }
        let failed = (results.len() as u32).saturating_sub(passed);
        ValidationReport {
            passed,
            failed,
            all_valid: failed == 0,
        }
    }
}

// ── Validation checks ─────────────────────────────────────────────────────────

/// @title validate_admin_present
/// @notice Verifies that the admin address is initialized in contract storage.
/// @dev    Automated testing should not run against a contract without an admin.
/// @security Read-only; no auth required.
pub fn validate_admin_present(env: &Env) -> ValidationResult {
    if env.storage().instance().has(&DataKey::Admin) {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid("admin is not initialized")
    }
}

/// @title validate_status_valid
/// @notice Verifies that the stored campaign status is present and valid.
/// @dev    A missing status indicates an incomplete or corrupted contract state.
/// @security Read-only; no auth required.
pub fn validate_status_valid(env: &Env) -> ValidationResult {
    let status: Option<Status> = env.storage().instance().get(&DataKey::Status);
    match status {
        Some(_) => ValidationResult::Valid,
        None => ValidationResult::Invalid("campaign status is missing"),
    }
}

/// @title validate_goal_compliance
/// @notice Verifies that the stored campaign goal is at least the minimum.
/// @dev    A zero or negative goal invalidates most campaign invariants.
/// @security Read-only; no auth required.
pub fn validate_goal_compliance(env: &Env) -> ValidationResult {
    let goal: Option<i128> = env.storage().instance().get(&DataKey::Goal);
    match goal {
        Some(g) if g >= MIN_COMPLIANT_GOAL => ValidationResult::Valid,
        Some(_) => ValidationResult::Invalid("campaign goal is zero or negative"),
        None => ValidationResult::Invalid("campaign goal is missing"),
    }
}

/// @title validate_minimum_contribution
/// @notice Verifies that the minimum contribution floor is compliant.
/// @dev    Zero or negative floors allow dust contributions and unnecessary gas.
/// @security Read-only; no auth required.
pub fn validate_minimum_contribution(env: &Env) -> ValidationResult {
    let min_contribution: Option<i128> = env.storage().instance().get(&DataKey::MinContribution);
    match min_contribution {
        Some(value) if value >= MIN_COMPLIANT_CONTRIBUTION => ValidationResult::Valid,
        Some(_) => ValidationResult::Invalid("minimum contribution is below the compliant floor"),
        None => ValidationResult::Invalid("minimum contribution is missing"),
    }
}

/// @title validate_deadline_in_future
/// @notice Verifies that an active campaign deadline is in the future.
/// @dev    Expired deadlines on active campaigns may indicate stale test fixtures.
/// @security Read-only; no auth required.
pub fn validate_deadline_in_future(env: &Env) -> ValidationResult {
    let status: Option<Status> = env.storage().instance().get(&DataKey::Status);
    if let Some(Status::Active) = status {
        let deadline: Option<u64> = env.storage().instance().get(&DataKey::Deadline);
        match deadline {
            Some(value) if value > env.ledger().timestamp() => ValidationResult::Valid,
            Some(_) => ValidationResult::Invalid("active campaign deadline has passed"),
            None => ValidationResult::Invalid("active campaign deadline is missing"),
        }
    } else {
        ValidationResult::Valid
    }
}

/// @title validate_platform_fee_cap
/// @notice Verifies that the platform fee does not exceed the allowed cap.
/// @dev    Missing platform config is treated as compliant.
/// @security Read-only; no auth required.
pub fn validate_platform_fee_cap(env: &Env) -> ValidationResult {
    let config: Option<PlatformConfig> = env.storage().instance().get(&DataKey::PlatformConfig);
    match config {
        Some(cfg) if cfg.fee_bps <= MAX_ALLOWED_FEE_BPS => ValidationResult::Valid,
        Some(_) => ValidationResult::Invalid("platform fee exceeds the allowed maximum"),
        None => ValidationResult::Valid,
    }
}

/// @title describe_validation_result
/// @notice Returns the failure message or "VALID" for a successful validation.
pub fn describe_validation_result(result: &ValidationResult) -> &'static str {
    match result {
        ValidationResult::Valid => "VALID",
        ValidationResult::Invalid(msg) => msg,
    }
}

/// @title audit_all_validations
/// @notice Runs all validation checks and returns an aggregated report.
/// @dev    The report is safe for CI automation and test harnesses.
pub fn audit_all_validations(env: &Env) -> ValidationReport {
    let results = [
        validate_admin_present(env),
        validate_status_valid(env),
        validate_goal_compliance(env),
        validate_minimum_contribution(env),
        validate_deadline_in_future(env),
        validate_platform_fee_cap(env),
    ];
    ValidationReport::from_results(&results)
}
