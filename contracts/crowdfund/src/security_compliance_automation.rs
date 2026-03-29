//! # security_compliance_automation
//!
//! @title   SecurityComplianceAutomation — Automated security compliance checks
//!          and audit trail for the crowdfund contract.
//!
//! @notice  Provides a suite of on-chain compliance helpers that can be called
//!          by automated tooling (CI pipelines, monitoring bots, governance
//!          scripts) to verify that the contract's security invariants hold at
//!          any point in time.  Every check emits a structured event so
//!          off-chain indexers can build a tamper-evident audit log without
//!          re-reading raw storage.
//!
//! @dev     All public functions are pure or read-only — they never mutate
//!          state.  This makes them safe to call from any context, including
//!          simulation calls that must not alter ledger state.
//!
//! ## Security Assumptions
//!
//! 1. **Read-only** — No function in this module writes to storage.
//! 2. **No auth required** — Compliance checks are intentionally permissionless
//!    so automated tooling does not need a privileged key.
//! 3. **Deterministic** — Given the same ledger state, every function returns
//!    the same result.  There are no side-effects beyond event emission.
//! 4. **Overflow-safe** — All arithmetic uses `checked_*` operations.
//! 5. **Bounded iteration** — `audit_all_checks` iterates over a fixed set of
//!    checks (O(1) with respect to contributor count).
//! 6. **Event integrity** — Events are emitted *after* the check result is
//!    computed, so a panicking host never emits a misleading event.

#![allow(dead_code)]

use soroban_sdk::{Address, Env, Symbol};

use crate::{DataKey, PlatformConfig, Status};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum allowed platform fee in basis points (10 %).
/// Fees above this threshold indicate a misconfiguration.
pub const MAX_ALLOWED_FEE_BPS: u32 = 1_000;

/// Minimum campaign goal considered compliant (1 token unit).
pub const MIN_COMPLIANT_GOAL: i128 = 1;

/// Minimum contribution floor considered compliant (1 token unit).
pub const MIN_COMPLIANT_CONTRIBUTION: i128 = 1;

/// Minimum deadline buffer from "now" that a compliant campaign must have
/// had at initialization (60 seconds).
pub const MIN_DEADLINE_BUFFER_SECS: u64 = 60;

// ── Result type ───────────────────────────────────────────────────────────────

/// The outcome of a single compliance check.
///
/// @notice `Passed` means the invariant holds; `Failed` carries a static
///         description of the violation for off-chain logging.
#[derive(Clone, PartialEq, Debug)]
pub enum CheckResult {
    Passed,
    Failed(&'static str),
}

impl CheckResult {
    /// Returns `true` when the check passed.
    pub fn is_passed(&self) -> bool {
        matches!(self, CheckResult::Passed)
    }

    /// Returns the violation description, or `""` when the check passed.
    pub fn violation(&self) -> &'static str {
        match self {
            CheckResult::Passed => "",
            CheckResult::Failed(msg) => msg,
        }
    }
}

// ── Individual compliance checks ──────────────────────────────────────────────

/// @title check_admin_initialized
/// @notice Verifies that an admin address has been stored in instance storage.
/// @dev    A missing admin means `upgrade()` and role-transfer functions would
///         panic, leaving the contract in an unmanageable state.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when `DataKey::Admin` is present.
pub fn check_admin_initialized(env: &Env) -> CheckResult {
    if env.storage().instance().has(&DataKey::Admin) {
        CheckResult::Passed
    } else {
        CheckResult::Failed("Admin key is not initialized")
    }
}

/// @title check_goal_positive
/// @notice Verifies that the stored campaign goal is >= `MIN_COMPLIANT_GOAL`.
/// @dev    A zero or negative goal would allow the campaign to succeed
///         immediately without any contributions.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when goal >= 1.
pub fn check_goal_positive(env: &Env) -> CheckResult {
    let goal: Option<i128> = env.storage().instance().get(&DataKey::Goal);
    match goal {
        None => CheckResult::Failed("Goal is not set"),
        Some(g) if g < MIN_COMPLIANT_GOAL => CheckResult::Failed("Goal is zero or negative"),
        _ => CheckResult::Passed,
    }
}

/// @title check_deadline_in_future
/// @notice Verifies that the stored deadline is strictly greater than the
///         current ledger timestamp.
/// @dev    An expired deadline on an Active campaign means contributions are
///         still accepted by the storage layer but would be rejected by the
///         deadline guard in `contribute()`.  This check surfaces that
///         inconsistency for monitoring.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when deadline > ledger timestamp.
pub fn check_deadline_in_future(env: &Env) -> CheckResult {
    let deadline: Option<u64> = env.storage().instance().get(&DataKey::Deadline);
    match deadline {
        None => CheckResult::Failed("Deadline is not set"),
        Some(d) if d <= env.ledger().timestamp() => {
            CheckResult::Failed("Deadline has already passed")
        }
        _ => CheckResult::Passed,
    }
}

/// @title check_platform_fee_within_limit
/// @notice Verifies that the configured platform fee does not exceed
///         `MAX_ALLOWED_FEE_BPS` (1 000 bps = 10 %).
/// @dev    Fees above 10 % are considered non-compliant and may indicate a
///         misconfiguration or a malicious platform address substitution.
///         When no platform config is stored the check passes (fee = 0).
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when fee_bps <= MAX_ALLOWED_FEE_BPS.
pub fn check_platform_fee_within_limit(env: &Env) -> CheckResult {
    let config: Option<PlatformConfig> = env.storage().instance().get(&DataKey::PlatformConfig);
    match config {
        None => CheckResult::Passed, // no fee configured — compliant
        Some(c) if c.fee_bps > MAX_ALLOWED_FEE_BPS => {
            CheckResult::Failed("Platform fee exceeds maximum allowed basis points")
        }
        _ => CheckResult::Passed,
    }
}

/// @title check_status_valid
/// @notice Verifies that the stored campaign status is one of the four valid
///         variants: Active, Succeeded, Expired, Cancelled.
/// @dev    An unrecognised status value would cause panics in every function
///         that pattern-matches on `DataKey::Status`.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when a valid status is stored.
pub fn check_status_valid(env: &Env) -> CheckResult {
    let status: Option<Status> = env.storage().instance().get(&DataKey::Status);
    match status {
        None => CheckResult::Failed("Campaign status is not set"),
        Some(_) => CheckResult::Passed, // contracttype deserialization guarantees a valid variant
    }
}

/// @title check_total_raised_non_negative
/// @notice Verifies that `TotalRaised` is >= 0.
/// @dev    A negative total would indicate an arithmetic bug or storage
///         corruption.  Contributions are validated to be positive before
///         being added, so this should never fail in a correctly operating
///         contract.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when total_raised >= 0.
pub fn check_total_raised_non_negative(env: &Env) -> CheckResult {
    let total: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalRaised)
        .unwrap_or(0);
    if total < 0 {
        CheckResult::Failed("TotalRaised is negative — possible arithmetic corruption")
    } else {
        CheckResult::Passed
    }
}

/// @title check_min_contribution_positive
/// @notice Verifies that `MinContribution` is >= `MIN_COMPLIANT_CONTRIBUTION`.
/// @dev    A zero minimum would allow dust contributions that bloat the
///         contributor list and inflate gas costs for `cancel()` and
///         `collect_pledges()`.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when min_contribution >= 1.
pub fn check_min_contribution_positive(env: &Env) -> CheckResult {
    let min: Option<i128> = env.storage().instance().get(&DataKey::MinContribution);
    match min {
        None => CheckResult::Failed("MinContribution is not set"),
        Some(m) if m < MIN_COMPLIANT_CONTRIBUTION => {
            CheckResult::Failed("MinContribution is zero or negative")
        }
        _ => CheckResult::Passed,
    }
}

/// @title check_paused_flag_present
/// @notice Verifies that the `Paused` flag key exists in instance storage.
/// @dev    The access-control module relies on this key being present after
///         initialization.  A missing key causes `assert_not_paused` to
///         default to `false`, which is safe, but its absence may indicate
///         that the access-control module was not properly initialized.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when `DataKey::Paused` is present.
pub fn check_paused_flag_present(env: &Env) -> CheckResult {
    if env.storage().instance().has(&DataKey::Paused) {
        CheckResult::Passed
    } else {
        CheckResult::Failed("Paused flag is not initialized in instance storage")
    }
}

/// @title check_token_address_set
/// @notice Verifies that a token contract address has been stored.
/// @dev    Without a token address, `contribute()`, `withdraw()`, and
///         `refund_single()` would all panic.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when `DataKey::Token` is present.
pub fn check_token_address_set(env: &Env) -> CheckResult {
    if env.storage().instance().has(&DataKey::Token) {
        CheckResult::Passed
    } else {
        CheckResult::Failed("Token address is not set")
    }
}

/// @title check_creator_address_set
/// @notice Verifies that a creator address has been stored.
/// @dev    Without a creator address, `withdraw()`, `cancel()`, and
///         `update_metadata()` would all panic.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `CheckResult::Passed` when `DataKey::Creator` is present.
pub fn check_creator_address_set(env: &Env) -> CheckResult {
    if env.storage().instance().has(&DataKey::Creator) {
        CheckResult::Passed
    } else {
        CheckResult::Failed("Creator address is not set")
    }
}

// ── Aggregate audit ───────────────────────────────────────────────────────────

/// @title ComplianceReport
/// @notice Aggregated result of all compliance checks.
/// @dev    `passed` is the count of checks that returned `CheckResult::Passed`.
///         `failed` is the count of checks that returned `CheckResult::Failed`.
///         `all_passed` is `true` iff `failed == 0`.
#[derive(Clone, Debug)]
pub struct ComplianceReport {
    pub passed: u32,
    pub failed: u32,
    pub all_passed: bool,
}

/// @title audit_all_checks
/// @notice Runs every compliance check and returns an aggregated report.
///         Emits one `compliance_audit` event per check and a final
///         `compliance_summary` event with the aggregate counts.
///
/// @dev    Checks are run in order of increasing storage cost:
///         1. Pure / flag checks (no deserialization).
///         2. Scalar value checks (single key reads).
///         3. Struct checks (PlatformConfig deserialization).
///
///         The function is O(1) with respect to contributor count — it never
///         iterates over the contributor list.
///
/// @security Read-only; no auth required.  Safe to call from simulation.
/// @param  env  The Soroban environment.
/// @return A `ComplianceReport` with aggregate pass/fail counts.
pub fn audit_all_checks(env: &Env) -> ComplianceReport {
    let checks: [(&'static str, CheckResult); 10] = [
        ("admin_initialized", check_admin_initialized(env)),
        ("creator_set", check_creator_address_set(env)),
        ("token_set", check_token_address_set(env)),
        ("status_valid", check_status_valid(env)),
        ("goal_positive", check_goal_positive(env)),
        ("min_contribution_positive", check_min_contribution_positive(env)),
        ("deadline_in_future", check_deadline_in_future(env)),
        ("total_raised_non_negative", check_total_raised_non_negative(env)),
        ("platform_fee_within_limit", check_platform_fee_within_limit(env)),
        ("paused_flag_present", check_paused_flag_present(env)),
    ];

    let mut passed: u32 = 0;
    let mut failed: u32 = 0;

    for (name, result) in checks.iter() {
        let ok = result.is_passed();
        // Emit per-check event — topic: (compliance_audit, <check_name>), data: bool
        env.events().publish(
            (
                Symbol::new(env, "compliance_audit"),
                Symbol::new(env, name),
            ),
            ok,
        );
        if ok {
            passed = passed.checked_add(1).unwrap_or(u32::MAX);
        } else {
            failed = failed.checked_add(1).unwrap_or(u32::MAX);
        }
    }

    let all_passed = failed == 0;

    // Emit summary event — data: (passed_count, failed_count)
    env.events().publish(
        (Symbol::new(env, "compliance_summary"),),
        (passed, failed),
    );

    ComplianceReport {
        passed,
        failed,
        all_passed,
    }
}

// ── Targeted audit helpers ────────────────────────────────────────────────────

/// @title audit_initialization
/// @notice Runs only the checks that verify the contract was correctly
///         initialized: admin, creator, token, status, goal, min_contribution.
/// @dev    Useful in post-deployment CI scripts that want a fast smoke-test
///         without running the full audit.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `true` when all initialization checks pass.
pub fn audit_initialization(env: &Env) -> bool {
    check_admin_initialized(env).is_passed()
        && check_creator_address_set(env).is_passed()
        && check_token_address_set(env).is_passed()
        && check_status_valid(env).is_passed()
        && check_goal_positive(env).is_passed()
        && check_min_contribution_positive(env).is_passed()
}

/// @title audit_financial_integrity
/// @notice Runs only the checks that verify financial invariants:
///         goal, total_raised, and platform fee.
/// @dev    Useful for periodic monitoring bots that focus on fund safety.
/// @security Read-only; no auth required.
/// @param  env  The Soroban environment.
/// @return `true` when all financial integrity checks pass.
pub fn audit_financial_integrity(env: &Env) -> bool {
    check_goal_positive(env).is_passed()
        && check_total_raised_non_negative(env).is_passed()
        && check_platform_fee_within_limit(env).is_passed()
}

/// @title describe_check_result
/// @notice Returns a human-readable string for a `CheckResult`.
/// @dev    Intended for off-chain tooling that logs compliance results.
/// @param  result  The `CheckResult` to describe.
/// @return A static string: `"PASSED"` or the violation message.
pub fn describe_check_result(result: &CheckResult) -> &'static str {
    match result {
        CheckResult::Passed => "PASSED",
        CheckResult::Failed(msg) => msg,
    }
}
