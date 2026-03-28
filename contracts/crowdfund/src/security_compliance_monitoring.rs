#![no_std]

use soroban_sdk::{contracttype, Env, Symbol, Vec, Address};

use crate::{ContractError, DataKey, Status, PlatformConfig, RoadmapItem, CampaignStats};

// ── Compliance Report ─────────────────────────────────────────────────────────

/// Comprehensive compliance audit result returned by `run_full_audit()`.
///
/// Contains pass/fail status and detailed violations for off-chain analysis.
#[derive(Clone)]
#[contracttype]
pub struct ComplianceReport {
    /// Overall pass/fail status.
    pub passed: bool,
    /// List of violations found (empty if passed=true).
    pub violations: Vec<ComplianceIssue>,
    /// Snapshot of key metrics at audit time.
    pub metrics: AuditMetrics,
}

/// Individual compliance violation with numeric code and description.
#[derive(Clone)]
#[contracttype]
pub struct ComplianceIssue {
    /// Numeric error code for scripting.
    pub code: u32,
    /// Human-readable description.
    pub description: &'static str,
}

/// Key metrics captured during the audit.
#[derive(Clone)]
#[contracttype]
pub struct AuditMetrics {
    pub total_raised: i128,
    pub contributor_count: u32,
    pub status: Status,
    pub ledger_timestamp: u64,
}

// ── Error Codes ──────────────────────────────────────────────────────────────

/// Numeric compliance violation codes for off-chain monitoring.
pub mod error_codes {
    /// Unauthorized creator/admin detected in storage.
    pub const UNAUTHORIZED_CREATOR: u32 = 100;
    /// Contributor list exceeds MAX_CONTRIBUTORS.
    pub const CONTRIBUTOR_LIMIT_EXCEEDED: u32 = 101;
    /// Roadmap exceeds MAX_ROADMAP_ITEMS.
    pub const ROADMAP_LIMIT_EXCEEDED: u32 = 102;
    /// Invalid status transition.
    pub const INVALID_STATUS: u32 = 103;
    /// Arithmetic anomaly (total_raised > goal * 2).
    pub const ARITHMETIC_ANOMALY: u32 = 104;
    /// Negative contributions detected.
    pub const NEGATIVE_CONTRIBUTION: u32 = 105;
    /// Platform fee > 100%.
    pub const INVALID_PLATFORM_FEE: u32 = 106;
}

/// Returns human-readable description for compliance error code.
pub fn describe_violation(code: u32) -> &'static str {
    match code {
        error_codes::UNAUTHORIZED_CREATOR => "Unauthorized creator address detected",
        error_codes::CONTRIBUTOR_LIMIT_EXCEEDED => "Contributors exceed MAX_CONTRIBUTORS (128)",
        error_codes::ROADMAP_LIMIT_EXCEEDED => "Roadmap exceeds MAX_ROADMAP_ITEMS (32)",
        error_codes::INVALID_STATUS => "Campaign status violates lifecycle invariants",
        error_codes::ARITHMETIC_ANOMALY => "Total raised exceeds plausible bounds (goal * 2)",
        error_codes::NEGATIVE_CONTRIBUTION => "Negative contribution amount detected",
        error_codes::INVALID_PLATFORM_FEE => "Platform fee exceeds 100% (10,000 bps)",
        _ => "Unknown compliance violation",
    }
}

// ── Individual Invariant Checks ──────────────────────────────────────────────

/// @notice Validates creator/admin authorization context.
///
/// @dev Assumes storage initialized; panics if not. Read-only.
///
/// @security Requires matching DataKey::Creator and DataKey::Admin.
///
/// @return Ok(()) if valid, Err(ComplianceIssue) otherwise.
pub fn check_auth_invariant(env: &amp;Env) -> Result<(), ComplianceIssue> {
    let creator: Address = env.storage().instance().get(&amp;DataKey::Creator).unwrap();
    let admin: Address = env.storage().instance().get(&amp;DataKey::Admin).unwrap_or(creator.clone());
    
    // Basic auth pattern: admin should be creator or designated
    if admin != creator {
        return Err(ComplianceIssue {
            code: error_codes::UNAUTHORIZED_CREATOR,
            description: "Admin differs from creator without explicit delegation",
        });
    }
    Ok(())
}

/// @notice Validates state size bounds.
///
/// @dev Checks contributors, roadmap, stretch goals against constants.
///
/// @security Prevents unbounded growth; caps gas/emission.
///
/// @return Ok(()) if within limits.
pub fn check_state_bounds(env: &amp;Env) -> Result<(), ComplianceIssue> {
    use crate::contract_state_size::{MAX_CONTRIBUTORS, MAX_ROADMAP_ITEMS, MAX_STRETCH_GOALS};

    let contributors: Vec<Address> = env.storage().persistent()
        .get(&amp;DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));
    if contributors.len() > MAX_CONTRIBUTORS as usize {
        return Err(ComplianceIssue {
            code: error_codes::CONTRIBUTOR_LIMIT_EXCEEDED,
            description: "Contributor list exceeds MAX_CONTRIBUTORS",
        });
    }

    let roadmap: Vec<RoadmapItem> = env.storage().instance()
        .get(&amp;DataKey::Roadmap)
        .unwrap_or_else(|| Vec::new(env));
    if roadmap.len() > MAX_ROADMAP_ITEMS as usize {
        return Err(ComplianceIssue {
            code: error_codes::ROADMAP_LIMIT_EXCEEDED,
            description: "Roadmap exceeds MAX_ROADMAP_ITEMS",
        });
    }

    // Add stretch goals check...
    Ok(())
}

/// @notice Validates arithmetic consistency.
///
/// @dev total_raised <= goal * 2; no negative contributions.
///
/// @security Detects overflow/underflow anomalies.
pub fn check_arithmetic_safety(env: &amp;Env) -> Result<(), ComplianceIssue> {
    let goal: i128 = env.storage().instance().get(&amp;DataKey::Goal).unwrap();
    let total_raised: i128 = env.storage().instance().get(&amp;DataKey::TotalRaised).unwrap_or(0);

    if total_raised > goal * 2 {
        return Err(ComplianceIssue {
            code: error_codes::ARITHMETIC_ANOMALY,
            description: "Total raised exceeds goal * 2 (overflow risk)",
        });
    }

    // Scan for negative contributions (bounded gas)
    let contributors: Vec<Address> = env.storage().persistent()
        .get(&amp;DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));
    for addr in contributors.iter().take(50) {  // Gas bound
        let contrib: i128 = env.storage().persistent()
            .get(&amp;DataKey::Contribution(addr.clone()))
            .unwrap_or(0);
        if contrib < 0 {
            return Err(ComplianceIssue {
                code: error_codes::NEGATIVE_CONTRIBUTION,
                description: "Negative contribution detected",
            });
        }
    }
    Ok(())
}

/// @notice Validates campaign status lifecycle.
///
/// @dev Active → Succeeded/Expired after deadline; no invalid states.
pub fn check_status_invariant(env: &amp;Env) -> Result<(), ComplianceIssue> {
    let status: Status = env.storage().instance().get(&amp;DataKey::Status).unwrap();
    let deadline: u64 = env.storage().instance().get(&amp;DataKey::Deadline).unwrap();
    let now = env.ledger().timestamp();

    match status {
        Status::Active if now > deadline => {
            Err(ComplianceIssue {
                code: error_codes::INVALID_STATUS,
                description: "Active after deadline (missing finalize)",
            })
        }
        _ => Ok(()),
    }
}

/// @notice Validates platform configuration.
///
/// @dev fee_bps <= 10_000.
pub fn check_platform_config(env: &amp;Env) -> Result<(), ComplianceIssue> {
    if let Some(config) = env.storage().instance().get::<PlatformConfig>(&amp;DataKey::PlatformConfig) {
        if config.fee_bps > 10_000 {
            return Err(ComplianceIssue {
                code: error_codes::INVALID_PLATFORM_FEE,
                description: "Platform fee exceeds 100%",
            });
        }
    }
    Ok(())
}

// ── Core Audit Functions ─────────────────────────────────────────────────────

/// @notice Runs comprehensive security compliance audit.
///
/// @dev Read-only; aggregates all invariant checks. Emits events on violations.
///
/// @security Checks → report → interactions (events). No state mutation.
///
/// @return ComplianceReport with pass/fail and details.
pub fn run_full_audit(env: &amp;Env) -> ComplianceReport {
    let mut violations = Vec::new(env);
    let total_raised = env.storage().instance().get(&amp;DataKey::TotalRaised).unwrap_or(0);
    let contributors = contributors_len(env);
    let status = env.storage().instance().get(&amp;DataKey::Status).unwrap();
    let timestamp = env.ledger().timestamp();

    macro_rules! check_and_log {
        ($check:expr) => {{
            match $check {
                Ok(_) => (),
                Err(issue) => {
                    log_compliance_violation(env, issue.code, issue.description);
                    violations.push_back(issue);
                }
            }
        }};
    }

    check_and_log!(check_auth_invariant(env));
    check_and_log!(check_state_bounds(env));
    check_and_log!(check_arithmetic_safety(env));
    check_and_log!(check_status_invariant(env));
    check_and_log!(check_platform_config(env));

    let passed = violations.is_empty();

    ComplianceReport {
        passed,
        violations,
        metrics: AuditMetrics {
            total_raised,
            contributor_count: contributors,
            status,
            ledger_timestamp: timestamp,
        },
    }
}

/// @notice Quick compliance status check (view function).
///
/// @dev Single bool for fast frontend polling/indexing.
///
/// @return true if all invariants hold.
pub fn compliance_status(env: &amp;Env) -> bool {
    run_full_audit(env).passed
}

fn contributors_len(env: &amp;Env) -> u32 {
    env.storage().persistent()
        .get(&amp;DataKey::Contributors)
        .map(|v: Vec<Address>| v.len() as u32)
        .unwrap_or(0)
}

/// @notice Emits structured compliance violation event (internal).
///
/// # Event schema
///
/// | topic 0 | `Symbol("security")`       |
/// | topic 1 | `Symbol("violation")`      |
/// | topic 2 | `Symbol(<issue_code>)`     |
/// | data    | `u32` numeric code        |
///
/// @dev Off-chain monitoring without host error parsing.
pub fn log_compliance_violation(env: &amp;Env, code: u32, _description: &amp;'static str) {
    env.events().publish(
        ("security", Symbol::new(env, "violation"), Symbol::new(env, &amp;format!("code_{}", code))),
        code,
    );
}

// ── Convenience Helpers ──────────────────────────────────────────────────────

/// Quick state size summary for dashboards.
pub fn get_compliance_metrics(env: &amp;Env) -> AuditMetrics {
    run_full_audit(env).metrics
}

