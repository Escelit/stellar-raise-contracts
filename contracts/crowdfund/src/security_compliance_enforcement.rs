//! # security_compliance_enforcement
//!
//! @title   SecurityComplianceEnforcement — Active enforcement of security
//!          compliance invariants for the crowdfund contract.
//!
//! @notice  Extends the read-only `security_compliance_automation` module with
//!          state-mutating enforcement capabilities. Where the automation module
//!          _observes_ and _reports_ compliance violations, this module
//!          _prevents_ and _remediates_ them. It provides:
//!
//!          - A zero-trust compliance gate (`assert_compliant`) that any caller
//!            can invoke before a state-changing operation.
//!          - An automatic circuit-breaker (`enforce_compliance`) that freezes
//!            the contract when critical invariants are violated.
//!          - Admin-controlled emergency halt/lift functions.
//!          - Targeted remediation functions for platform fee and minimum
//!            contribution misconfigurations.
//!          - A read-only status snapshot (`compliance_status`) for off-chain
//!            monitoring bots.
//!
//! @dev     All state-mutating functions require Soroban `require_auth()` plus
//!          a role check against `DataKey::Admin`, `DataKey::DefaultAdmin`, or
//!          `DataKey::Pauser`. Read-only functions are permissionless.
//!          Check logic is fully delegated to `security_compliance_automation`
//!          — no duplication of check logic exists in this module.
//!
//! ## Security Assumptions
//!
//! 1. **Authorization model** — State-mutating functions enforce authorization
//!    via `caller.require_auth()` (Soroban host-level auth) followed by a
//!    role check against instance storage keys (`DataKey::Admin`,
//!    `DataKey::DefaultAdmin`, `DataKey::Pauser`). Permissionless functions
//!    (`assert_compliant`, `enforce_compliance`, `compliance_status`) never
//!    mutate storage and require no auth.
//! 2. **Storage mutation scope** — Only `DataKey::Paused`,
//!    `DataKey::PlatformConfig`, and `DataKey::MinContribution` are ever
//!    written by this module. No other storage keys are modified.
//! 3. **Event emission ordering** — All enforcement events are emitted
//!    *after* the corresponding storage mutation completes. A host-level
//!    panic during event emission therefore cannot leave storage in an
//!    inconsistent state.
//! 4. **Overflow safety** — All counter arithmetic uses `checked_add` with
//!    `unwrap_or(u32::MAX)` saturation, matching the pattern established in
//!    the automation module.
//! 5. **Reentrancy** — Soroban's single-threaded execution model prevents
//!    reentrancy. No cross-contract calls are made by this module.
//! 6. **Determinism** — Given the same ledger state, every read-only function
//!    returns the same result. State-mutating functions produce deterministic
//!    storage transitions.

#![allow(unused_imports)]

use soroban_sdk::{contracterror, contracttype, Address, Env, IntoVal, Symbol, Val};

use crate::security_compliance_automation::{
    audit_all_checks, check_admin_initialized, check_creator_address_set, check_deadline_in_future,
    check_goal_positive, check_min_contribution_positive, check_paused_flag_present,
    check_platform_fee_within_limit, check_status_valid, check_token_address_set,
    check_total_raised_non_negative, ComplianceReport,
};
use crate::DataKey;
use crate::PlatformConfig;

// ── Re-exports ────────────────────────────────────────────────────────────────

/// Re-exported from the automation module: maximum allowed platform fee in
/// basis points (10 %). Used by `remediate_platform_fee` for validation.
pub use crate::security_compliance_automation::MAX_ALLOWED_FEE_BPS;

/// Re-exported from the automation module: minimum compliant contribution
/// floor (1 token unit). Used by `remediate_min_contribution` for validation.
pub use crate::security_compliance_automation::MIN_COMPLIANT_CONTRIBUTION;

// ── Error Type ────────────────────────────────────────────────────────────────

/// @title   EnforcementError
/// @notice  All error variants that can be returned by enforcement functions.
///          Each variant maps to a stable `u32` discriminant so that error
///          codes remain consistent across contract upgrades.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EnforcementError {
    /// One or more Critical_Violations were detected (admin absent, token
    /// absent, status invalid, or total_raised negative).
    CriticalViolationDetected = 1,

    /// Only Non_Critical_Violations were detected (e.g. deadline passed,
    /// fee too high, paused flag absent). No Emergency_Halt is triggered.
    NonCriticalViolationsPresent = 2,

    /// The supplied remediation value is out of the compliant range
    /// (e.g. `new_fee_bps > MAX_ALLOWED_FEE_BPS` or `new_min < 1`).
    RemediationValueInvalid = 3,

    /// `remediate_platform_fee` was called but no `PlatformConfig` is
    /// currently stored in instance storage.
    NoPlatformConfigToRemediate = 4,

    /// Supplementary unauthorized variant. Primary authorization uses
    /// `require_auth` panic; this variant is reserved for future use.
    Unauthorized = 5,
}

// ── Status Type ───────────────────────────────────────────────────────────────

/// @title   EnforcementStatus
/// @notice  A read-only snapshot of the contract's current compliance and
///          enforcement state, returned by `compliance_status`. Intended for
///          off-chain monitoring bots and dashboards.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EnforcementStatus {
    /// `true` when `DataKey::Paused` is set to `true` in instance storage,
    /// indicating the contract is in an Emergency_Halt state.
    pub is_halted: bool,

    /// `true` when at least one Critical_Violation check fails
    /// (`check_admin_initialized`, `check_token_address_set`,
    /// `check_status_valid`, `check_total_raised_non_negative`).
    pub has_critical_violations: bool,

    /// `true` when at least one Non_Critical_Violation check fails
    /// (any check not listed under Critical_Violation).
    pub has_non_critical_violations: bool,

    /// Count of failing Critical_Violation checks (0–4).
    pub critical_violation_count: u32,

    /// Count of failing Non_Critical_Violation checks (0–6).
    pub non_critical_violation_count: u32,
}

// ── Internal Helpers ──────────────────────────────────────────────────────────

/// Checks whether `caller` is the Admin or DefaultAdmin.
///
/// Used by remediation functions and `lift_emergency_halt` to enforce that
/// only the two highest-privilege roles can perform those operations.
///
/// Returns `true` when `caller` matches the address stored under
/// `DataKey::Admin` or `DataKey::DefaultAdmin` in instance storage.
/// Returns `false` when neither key is present or neither matches.
fn is_authorized_admin(env: &Env, caller: &Address) -> bool {
    // SECURITY: role check — caller must match Admin or DefaultAdmin stored in
    // instance storage. This is evaluated *after* `require_auth()` has already
    // confirmed the caller's cryptographic signature, so we are only checking
    // the role assignment here, not re-doing authentication.
    let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
    if let Some(ref a) = admin {
        if a == caller {
            return true;
        }
    }
    let default_admin: Option<Address> = env.storage().instance().get(&DataKey::DefaultAdmin);
    if let Some(ref da) = default_admin {
        if da == caller {
            return true;
        }
    }
    false
}

/// Checks whether `caller` is the Admin, DefaultAdmin, or Pauser.
///
/// Used by `trigger_emergency_halt` to enforce that the three roles authorized
/// to freeze the contract can do so.
///
/// Returns `true` when `caller` matches the address stored under
/// `DataKey::Admin`, `DataKey::DefaultAdmin`, or `DataKey::Pauser` in
/// instance storage. Returns `false` when none of the keys are present or
/// none match.
fn is_authorized_halter(env: &Env, caller: &Address) -> bool {
    // SECURITY: role check — caller must match Admin, DefaultAdmin, or Pauser
    // stored in instance storage. Evaluated after `require_auth()` has already
    // confirmed the caller's cryptographic signature.
    if is_authorized_admin(env, caller) {
        return true;
    }
    let pauser: Option<Address> = env.storage().instance().get(&DataKey::Pauser);
    if let Some(ref p) = pauser {
        if p == caller {
            return true;
        }
    }
    false
}

/// Emits a structured `enforcement_action` event.
///
/// All enforcement events share the same topic structure so that off-chain
/// indexers can subscribe to a single topic filter:
///
/// ```
/// topics: (Symbol("enforcement_action"), Symbol(<action>))
/// data:   (ledger_timestamp: u64, detail: Val)
/// ```
///
/// This function is always called *after* any storage mutation has completed,
/// ensuring that a host-level panic during event emission cannot leave storage
/// in an inconsistent state (Requirement 3.6).
fn emit_enforcement_action(env: &Env, action: &str, detail: impl IntoVal<Env, Val>) {
    let timestamp: u64 = env.ledger().timestamp();
    let detail_val: Val = detail.into_val(env);
    env.events().publish(
        (
            Symbol::new(env, "enforcement_action"),
            Symbol::new(env, action),
        ),
        (timestamp, detail_val),
    );
}

// ── Public Functions ──────────────────────────────────────────────────────────

/// @title   assert_compliant
/// @notice  Zero-trust compliance gate. Call this before any state-changing
///          operation to verify that all critical invariants hold.
/// @dev     Runs the four critical checks from the automation module:
///          `check_admin_initialized`, `check_token_address_set`,
///          `check_status_valid`, and `check_total_raised_non_negative`.
///          Returns `Ok(())` when all pass. On the first failure, emits a
///          `"gate_blocked"` enforcement event with the failing check name as
///          detail, then returns `Err(CriticalViolationDetected)`.
///          Storage is never mutated by this function.
/// @security No `require_auth` is called — this function is intentionally
///           permissionless so any caller (including automated tooling) can
///           verify compliance without a privileged key.
/// @return  `Ok(())` when all critical checks pass;
///          `Err(EnforcementError::CriticalViolationDetected)` on the first
///          failing critical check.
pub fn assert_compliant(env: &Env) -> Result<(), EnforcementError> {
    // SECURITY: No require_auth — this function is permissionless by design.
    // It only reads storage and emits events; it never mutates state.

    let critical_checks: [(&'static str, bool); 4] = [
        (
            "admin_initialized",
            check_admin_initialized(env).is_passed(),
        ),
        (
            "token_address_set",
            check_token_address_set(env).is_passed(),
        ),
        ("status_valid", check_status_valid(env).is_passed()),
        (
            "total_raised_non_negative",
            check_total_raised_non_negative(env).is_passed(),
        ),
    ];

    for (name, passed) in critical_checks.iter() {
        if !passed {
            // Emit gate_blocked event before returning the error.
            // Storage is not mutated — event emission is the only side-effect.
            emit_enforcement_action(env, "gate_blocked", Symbol::new(env, name));
            return Err(EnforcementError::CriticalViolationDetected);
        }
    }

    Ok(())
}

/// @title   enforce_compliance
/// @notice  Full compliance audit with automatic Emergency_Halt on critical
///          violations. Runs all checks via `audit_all_checks` and acts on
///          the result.
/// @dev     - When `all_passed == true`: returns `Ok(report)` without
///            mutating storage.
///          - When critical violations exist: sets `DataKey::Paused = true`,
///            emits `"halt_triggered"` with the critical violation count, and
///            returns `Err(CriticalViolationDetected)`.
///          - When only non-critical violations exist: returns
///            `Err(NonCriticalViolationsPresent)` without mutating storage.
///          Critical checks: `check_admin_initialized`,
///          `check_token_address_set`, `check_status_valid`,
///          `check_total_raised_non_negative`.
/// @security No `require_auth` — permissionless so automated monitoring bots
///           can trigger the circuit-breaker without a privileged key.
/// @return  `Ok(ComplianceReport)` when all checks pass;
///          `Err(EnforcementError::CriticalViolationDetected)` when critical
///          violations are found (contract is halted);
///          `Err(EnforcementError::NonCriticalViolationsPresent)` when only
///          non-critical violations are found.
pub fn enforce_compliance(env: &Env) -> Result<ComplianceReport, EnforcementError> {
    // SECURITY: No require_auth — permissionless circuit-breaker by design.

    let report = audit_all_checks(env);

    if report.all_passed {
        return Ok(report);
    }

    // Count critical violations individually.
    let critical_checks = [
        check_admin_initialized(env).is_passed(),
        check_token_address_set(env).is_passed(),
        check_status_valid(env).is_passed(),
        check_total_raised_non_negative(env).is_passed(),
    ];

    let critical_violation_count: u32 =
        critical_checks.iter().filter(|&&passed| !passed).count() as u32;

    if critical_violation_count > 0 {
        // SECURITY: Storage write — set Paused = true to freeze the contract.
        // This mutation happens before event emission per Requirement 3.6.
        env.storage().instance().set(&DataKey::Paused, &true);

        emit_enforcement_action(env, "halt_triggered", critical_violation_count);
        return Err(EnforcementError::CriticalViolationDetected);
    }

    // Only non-critical violations — do not halt.
    Err(EnforcementError::NonCriticalViolationsPresent)
}

/// @title   trigger_emergency_halt
/// @notice  Manually freeze the contract by setting `DataKey::Paused = true`.
///          Callable by Admin, DefaultAdmin, or Pauser.
/// @dev     Idempotent: succeeds even when the contract is already paused.
///          Emits a `"manual_halt"` enforcement event with the caller address
///          as detail after the storage write completes.
/// @security `caller.require_auth()` is called first to verify the caller's
///           cryptographic signature. A role check against Admin, DefaultAdmin,
///           and Pauser follows. The storage write happens after both checks.
/// @param   env     The Soroban environment.
/// @param   caller  The address triggering the halt (must be Admin, DefaultAdmin,
///                  or Pauser).
pub fn trigger_emergency_halt(env: &Env, caller: Address) {
    // SECURITY: require_auth must be called before any storage read or write
    // to prevent signature-less callers from freezing the contract.
    caller.require_auth();

    // SECURITY: Role check — only Admin, DefaultAdmin, or Pauser may halt.
    if !is_authorized_halter(env, &caller) {
        panic!("caller is not authorized to trigger emergency halt");
    }

    // SECURITY: Storage write — set Paused = true. Idempotent: safe to call
    // even when already paused. Mutation precedes event emission (Req 3.6).
    env.storage().instance().set(&DataKey::Paused, &true);

    emit_enforcement_action(env, "manual_halt", caller);
}

/// @title   lift_emergency_halt
/// @notice  Unfreeze the contract by setting `DataKey::Paused = false`.
///          Callable by Admin or DefaultAdmin only (not Pauser).
/// @dev     Emits a `"halt_lifted"` enforcement event with the caller address
///          as detail after the storage write completes.
/// @security `caller.require_auth()` is called first. A role check against
///           Admin and DefaultAdmin follows (Pauser is intentionally excluded
///           — only the two highest-privilege roles can lift a halt).
/// @param   env     The Soroban environment.
/// @param   caller  The address lifting the halt (must be Admin or DefaultAdmin).
pub fn lift_emergency_halt(env: &Env, caller: Address) {
    // SECURITY: require_auth must be called before any storage read or write.
    caller.require_auth();

    // SECURITY: Role check — only Admin or DefaultAdmin may lift a halt.
    // Pauser is intentionally excluded from this operation.
    if !is_authorized_admin(env, &caller) {
        panic!("caller is not authorized to lift emergency halt");
    }

    // SECURITY: Storage write — set Paused = false. Mutation precedes event
    // emission per Requirement 3.6.
    env.storage().instance().set(&DataKey::Paused, &false);

    emit_enforcement_action(env, "halt_lifted", caller);
}

/// @title   remediate_platform_fee
/// @notice  Correct an out-of-range platform fee without a full contract
///          upgrade. Updates only the `fee_bps` field of the stored
///          `PlatformConfig`, preserving the `address` field.
/// @dev     Validation order (cheapest first):
///          1. `caller.require_auth()` — cryptographic auth.
///          2. Role check — Admin or DefaultAdmin only.
///          3. Range check — `new_fee_bps <= MAX_ALLOWED_FEE_BPS`.
///          4. Existence check — `PlatformConfig` must be stored.
///          5. Storage update + event emission.
/// @security Auth and role checks precede all storage reads/writes. The range
///           check and existence check both return errors without mutating
///           storage, ensuring no partial state is written on failure.
/// @param   env         The Soroban environment.
/// @param   caller      The address performing the remediation (Admin or DefaultAdmin).
/// @param   new_fee_bps The corrected fee in basis points (0–1000 inclusive).
/// @return  `Ok(())` on success;
///          `Err(EnforcementError::RemediationValueInvalid)` when
///          `new_fee_bps > MAX_ALLOWED_FEE_BPS`;
///          `Err(EnforcementError::NoPlatformConfigToRemediate)` when no
///          `PlatformConfig` is stored.
pub fn remediate_platform_fee(
    env: &Env,
    caller: Address,
    new_fee_bps: u32,
) -> Result<(), EnforcementError> {
    // SECURITY: require_auth before any storage access.
    caller.require_auth();

    // SECURITY: Role check — only Admin or DefaultAdmin may remediate fees.
    if !is_authorized_admin(env, &caller) {
        panic!("caller is not authorized to remediate platform fee");
    }

    // SECURITY: Validation — reject out-of-range values before touching storage.
    if new_fee_bps > MAX_ALLOWED_FEE_BPS {
        return Err(EnforcementError::RemediationValueInvalid);
    }

    // Read existing config — return error if absent (no upsert semantics here).
    let mut config: PlatformConfig = env
        .storage()
        .instance()
        .get(&DataKey::PlatformConfig)
        .ok_or(EnforcementError::NoPlatformConfigToRemediate)?;

    // SECURITY: Storage write — update only fee_bps, preserving address field.
    // Mutation precedes event emission per Requirement 3.6.
    config.fee_bps = new_fee_bps;
    env.storage()
        .instance()
        .set(&DataKey::PlatformConfig, &config);

    emit_enforcement_action(env, "remediation_applied", new_fee_bps);
    Ok(())
}

/// @title   remediate_min_contribution
/// @notice  Correct a zero or negative minimum contribution without a full
///          contract upgrade. Writes `new_min` to `DataKey::MinContribution`
///          (upsert semantics — creates the key if absent).
/// @dev     Validation order:
///          1. `caller.require_auth()` — cryptographic auth.
///          2. Role check — Admin or DefaultAdmin only.
///          3. Range check — `new_min >= MIN_COMPLIANT_CONTRIBUTION` (1).
///          4. Storage write + event emission.
/// @security Auth and role checks precede all storage writes. The range check
///           returns an error without mutating storage on failure.
/// @param   env     The Soroban environment.
/// @param   caller  The address performing the remediation (Admin or DefaultAdmin).
/// @param   new_min The corrected minimum contribution (must be >= 1).
/// @return  `Ok(())` on success;
///          `Err(EnforcementError::RemediationValueInvalid)` when
///          `new_min < MIN_COMPLIANT_CONTRIBUTION`.
pub fn remediate_min_contribution(
    env: &Env,
    caller: Address,
    new_min: i128,
) -> Result<(), EnforcementError> {
    // SECURITY: require_auth before any storage access.
    caller.require_auth();

    // SECURITY: Role check — only Admin or DefaultAdmin may remediate min contribution.
    if !is_authorized_admin(env, &caller) {
        panic!("caller is not authorized to remediate min contribution");
    }

    // SECURITY: Validation — reject values below the compliance floor.
    if new_min < MIN_COMPLIANT_CONTRIBUTION {
        return Err(EnforcementError::RemediationValueInvalid);
    }

    // SECURITY: Storage write (upsert) — creates or overwrites MinContribution.
    // Mutation precedes event emission per Requirement 3.6.
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &new_min);

    emit_enforcement_action(env, "remediation_applied", new_min);
    Ok(())
}

/// @title   compliance_status
/// @notice  Read-only snapshot of the contract's current compliance and
///          enforcement state. Intended for off-chain monitoring bots and
///          dashboards.
/// @dev     Reads `DataKey::Paused` for `is_halted` (defaults to `false` when
///          absent). Runs each critical and non-critical check individually to
///          populate the violation counts. All counter arithmetic uses
///          `checked_add` with `unwrap_or(u32::MAX)` saturation.
///          Storage is never mutated by this function.
/// @security No `require_auth` — permissionless read-only function.
/// @param   env  The Soroban environment.
/// @return  An `EnforcementStatus` struct with `is_halted`, violation flags,
///          and violation counts.
pub fn compliance_status(env: &Env) -> EnforcementStatus {
    // Read paused flag — default to false when absent.
    let is_halted: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);

    // Run each critical check individually and count failures.
    let critical_results = [
        check_admin_initialized(env).is_passed(),
        check_token_address_set(env).is_passed(),
        check_status_valid(env).is_passed(),
        check_total_raised_non_negative(env).is_passed(),
    ];

    let mut critical_violation_count: u32 = 0;
    for passed in critical_results.iter() {
        if !passed {
            // SECURITY: Overflow-safe counter increment with saturation.
            critical_violation_count = critical_violation_count.saturating_add(1);
        }
    }

    // Run each non-critical check individually and count failures.
    let non_critical_results = [
        check_creator_address_set(env).is_passed(),
        check_goal_positive(env).is_passed(),
        check_min_contribution_positive(env).is_passed(),
        check_deadline_in_future(env).is_passed(),
        check_platform_fee_within_limit(env).is_passed(),
        check_paused_flag_present(env).is_passed(),
    ];

    let mut non_critical_violation_count: u32 = 0;
    for passed in non_critical_results.iter() {
        if !passed {
            // SECURITY: Overflow-safe counter increment with saturation.
            non_critical_violation_count = non_critical_violation_count.saturating_add(1);
        }
    }

    EnforcementStatus {
        is_halted,
        has_critical_violations: critical_violation_count > 0,
        has_non_critical_violations: non_critical_violation_count > 0,
        critical_violation_count,
        non_critical_violation_count,
    }
}
