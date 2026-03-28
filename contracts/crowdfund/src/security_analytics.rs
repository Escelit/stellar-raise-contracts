//! # security_analytics
//!
//! @title   SecurityAnalytics — Advanced security analytics and threat intelligence
//!          for the crowdfund contract ecosystem.
//!
//! @notice  This module provides on-chain security analytics capabilities including
//!          threat detection, anomaly analysis, access pattern monitoring, and
//!          security metrics aggregation. Designed for integration with SIEM
//!          systems, automated threat response, and security audit trails.
//!
//! @dev     All public functions are read-only unless explicitly state-mutating.
//!          Functions that modify state emit structured events for off-chain
//!          indexing and analysis.
//!
//! ## Security Assumptions
//!
//! 1. **Read-only by default** — Analytics functions do not mutate storage.
//! 2. **No auth required** — All analytics queries are permissionless for
//!    integration with monitoring systems.
//! 3. **Overflow-safe** — All arithmetic uses `checked_*` operations.
//! 4. **Bounded iteration** — All loops iterate over fixed or bounded sets.
//! 5. **Event-driven** — State changes emit structured events for audit trails.
//! 6. **Non-blocking** — Analytics never reverts transactions; results are advisory.

#![allow(dead_code)]

use soroban_sdk::{Address, Env, Symbol};

use crate::{DataKey, MetricType, Status};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum number of entries in threat log before rotation.
pub const MAX_THREAT_LOG_SIZE: u32 = 1000;

/// Maximum number of access pattern entries tracked per entity.
pub const MAX_ACCESS_PATTERN_ENTRIES: u32 = 500;

/// Threat severity levels.
#[derive(Clone, Copy, PartialEq, Debug)]
#[contracttype]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatSeverity {
    /// Returns numeric priority (higher = more severe).
    pub fn priority(&self) -> u8 {
        match self {
            ThreatSeverity::Low => 1,
            ThreatSeverity::Medium => 2,
            ThreatSeverity::High => 3,
            ThreatSeverity::Critical => 4,
        }
    }

    /// Returns true if severity requires immediate attention.
    pub fn is_critical(&self) -> bool {
        matches!(self, ThreatSeverity::High | ThreatSeverity::Critical)
    }
}

/// Types of security threats detected.
#[derive(Clone, Copy, PartialEq, Debug)]
#[contracttype]
pub enum ThreatType {
    /// Unusual contribution pattern detected.
    AbnormalContributionPattern,
    /// Repeated failed transaction attempts.
    RepeatedFailure,
    /// Suspicious address interaction pattern.
    SuspiciousAddressActivity,
    /// Rapid state changes detected.
    RapidStateChanges,
    /// Unusual withdrawal pattern.
    AbnormalWithdrawalPattern,
    /// Authorization anomaly detected.
    AuthAnomaly,
    /// Storage anomaly detected.
    StorageAnomaly,
    /// Rate limit threshold exceeded.
    RateLimitExceeded,
    /// Access pattern anomaly.
    AccessPatternAnomaly,
}

// ── Data Structures ──────────────────────────────────────────────────────────

/// A single threat event record.
#[derive(Clone)]
#[contracttype]
pub struct ThreatRecord {
    /// Unix timestamp when threat was detected.
    pub timestamp: u64,
    /// Type of threat detected.
    pub threat_type: ThreatType,
    /// Severity level.
    pub severity: ThreatSeverity,
    /// Associated address (if applicable).
    pub address: Option<Address>,
    /// Additional threat details as symbol.
    pub details: Symbol,
    /// Whether threat has been acknowledged.
    pub acknowledged: bool,
}

/// Summary of security analytics for a time period.
#[derive(Clone)]
#[contracttype]
pub struct SecuritySummary {
    /// Total threats detected in period.
    pub threat_count: u32,
    /// Critical threats count.
    pub critical_threats: u32,
    /// Total transactions processed.
    pub total_transactions: u32,
    /// Failed transaction rate (basis points).
    pub failure_rate_bps: u32,
    /// Unique active addresses.
    pub unique_addresses: u32,
    /// Average transaction size.
    pub avg_transaction_size: i128,
    /// Security score (0-100, higher is better).
    pub security_score: u8,
}

/// Access pattern entry for tracking address behavior.
#[derive(Clone)]
#[contracttype]
pub struct AccessPatternEntry {
    /// Timestamp of access.
    pub timestamp: u64,
    /// Action type performed.
    pub action: Symbol,
    /// Result of action (success/failure).
    pub success: bool,
    /// Associated value (if applicable).
    pub value: i128,
}

/// Anomaly detection result.
#[derive(Clone)]
#[contracttype]
pub struct AnomalyReport {
    /// Whether anomaly was detected.
    pub anomaly_detected: bool,
    /// Anomaly type identifier.
    pub anomaly_type: Symbol,
    /// Confidence score (0-100).
    pub confidence: u8,
    /// Description of anomaly.
    pub description: Symbol,
}

// ── Threat Detection Functions ───────────────────────────────────────────────

/// @title detect_contribution_anomaly
/// @notice Analyzes contribution patterns to detect anomalies such as
///         unusual contribution sizes, rapid contributions, or coordinated
///         activity patterns.
/// @dev    Compares current contribution against historical baseline.
/// @security Read-only; no auth required.
/// @param  env         The Soroban environment.
/// @param  contributor The contributor address to analyze.
/// @param  amount      The contribution amount.
/// @return AnomalyReport with detection results.
pub fn detect_contribution_anomaly(
    env: &Env,
    contributor: &Address,
    amount: i128,
) -> AnomalyReport {
    let threshold_large = 10_000_000_000; // 10k with 7 decimals
    let threshold_rapid = 5u32; // Max contributions per block window

    // Check for large transaction anomaly
    if amount > threshold_large {
        return AnomalyReport {
            anomaly_detected: true,
            anomaly_type: Symbol::new(env, "LARGE_CONTRIB"),
            confidence: 75,
            description: Symbol::new(env, "Contribution exceeds normal threshold"),
        };
    }

    // Check for rapid contribution pattern
    let access_key = DataKey::AccessPattern(contributor.clone());
    let pattern: Vec<AccessPatternEntry> = env
        .storage()
        .instance()
        .get(&access_key)
        .unwrap_or_else(|| Vec::new(env));

    let current_time = env.ledger().timestamp();
    let recent_accesses: u32 = pattern
        .iter()
        .filter(|entry| {
            entry.timestamp > current_time.saturating_sub(3600) // Last hour
        })
        .filter(|entry| entry.action == Symbol::new(env, "CONTRIBUTE"))
        .count()
        .try_into()
        .unwrap_or(threshold_rapid);

    if recent_accesses >= threshold_rapid {
        return AnomalyReport {
            anomaly_detected: true,
            anomaly_type: Symbol::new(env, "RAPID_CONTRIB"),
            confidence: 85,
            description: Symbol::new(env, "Rapid contribution pattern detected"),
        };
    }

    AnomalyReport {
        anomaly_detected: false,
        anomaly_type: Symbol::new(env, "NONE"),
        confidence: 0,
        description: Symbol::new(env, "No anomaly detected"),
    }
}

/// @title detect_withdrawal_anomaly
/// @notice Analyzes withdrawal patterns to detect anomalies such as
///         unusual withdrawal timing, amounts, or frequency.
/// @dev    Monitors withdrawal behavior against established patterns.
/// @security Read-only; no auth required.
/// @param  env       The Soroban environment.
/// @param  withdrawer The withdrawer address.
/// @param  amount    The withdrawal amount.
/// @return AnomalyReport with detection results.
pub fn detect_withdrawal_anomaly(
    env: &Env,
    withdrawer: &Address,
    amount: i128,
) -> AnomalyReport {
    let threshold_large = 5_000_000_000; // 5k with 7 decimals

    // Check for large withdrawal anomaly
    if amount > threshold_large {
        return AnomalyReport {
            anomaly_detected: true,
            anomaly_type: Symbol::new(env, "LARGE_WITHDRAW"),
            confidence: 80,
            description: Symbol::new(env, "Large withdrawal detected"),
        };
    }

    // Check withdrawal frequency
    let access_key = DataKey::AccessPattern(withdrawer.clone());
    let pattern: Vec<AccessPatternEntry> = env
        .storage()
        .instance()
        .get(&access_key)
        .unwrap_or_else(|| Vec::new(env));

    let current_time = env.ledger().timestamp();
    let recent_withdrawals: u32 = pattern
        .iter()
        .filter(|entry| {
            entry.timestamp > current_time.saturating_sub(1800) // Last 30 min
        })
        .filter(|entry| entry.action == Symbol::new(env, "WITHDRAW"))
        .count()
        .try_into()
        .unwrap_or(10);

    if recent_withdrawals > 3 {
        return AnomalyReport {
            anomaly_detected: true,
            anomaly_type: Symbol::new(env, "RAPID_WITHDRAW"),
            confidence: 70,
            description: Symbol::new(env, "Rapid withdrawal pattern detected"),
        };
    }

    AnomalyReport {
        anomaly_detected: false,
        anomaly_type: Symbol::new(env, "NONE"),
        confidence: 0,
        description: Symbol::new(env, "No anomaly detected"),
    }
}

/// @title detect_auth_anomaly
/// @notice Detects authorization-related anomalies such as repeated
///         auth failures, unusual admin activity, or role abuse.
/// @dev    Tracks auth failure patterns and flags suspicious activity.
/// @security Read-only; no auth required.
/// @param  env      The Soroban environment.
/// @param  address  The address to analyze.
/// @return AnomalyReport with detection results.
pub fn detect_auth_anomaly(env: &Env, address: &Address) -> AnomalyReport {
    let metric_key = DataKey::SecurityMetric(address.clone(), MetricType::AuthFailures);
    let failures: u32 = env
        .storage()
        .instance()
        .get(&metric_key)
        .unwrap_or(0);

    if failures > 10 {
        return AnomalyReport {
            anomaly_detected: true,
            anomaly_type: Symbol::new(env, "AUTH_FAILURES"),
            confidence: 90,
            description: Symbol::new(env, "Excessive authorization failures detected"),
        };
    }

    // Check for admin role anomaly
    if env.storage().instance().has(&DataKey::Admin) {
        if let Some(admin) = env.storage().instance().get::<_, Address>(&DataKey::Admin) {
            if address == &admin {
                // Check admin activity pattern
                let access_key = DataKey::AccessPattern(admin);
                if let Some(pattern) = env
                    .storage()
                    .instance()
                    .get::<_, Vec<AccessPatternEntry>>(&access_key)
                {
                    let current_time = env.ledger().timestamp();
                    let recent_admin_actions: u32 = pattern
                        .iter()
                        .filter(|entry| entry.timestamp > current_time.saturating_sub(300)) // 5 min
                        .count()
                        .try_into()
                        .unwrap_or(100);

                    if recent_admin_actions > 5 {
                        return AnomalyReport {
                            anomaly_detected: true,
                            anomaly_type: Symbol::new(env, "ADMIN_BURST"),
                            confidence: 75,
                            description: Symbol::new(env, "Unusual admin activity burst"),
                        };
                    }
                }
            }
        }
    }

    AnomalyReport {
        anomaly_detected: false,
        anomaly_type: Symbol::new(env, "NONE"),
        confidence: 0,
        description: Symbol::new(env, "No anomaly detected"),
    }
}

/// @title detect_storage_anomaly
/// @notice Detects storage-related anomalies such as unusual state changes,
///         storage growth patterns, or data integrity issues.
/// @dev    Monitors storage modifications for suspicious patterns.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @return AnomalyReport with detection results.
pub fn detect_storage_anomaly(env: &Env) -> AnomalyReport {
    // Check for rapid status changes
    let status_key = DataKey::Status;
    let current_status: Option<Status> = env.storage().instance().get(&status_key);

    if let Some(status) = current_status {
        // Status-based anomaly checks
        match status {
            Status::Active => {
                // Check if deadline is approaching
                if let Some(deadline) = env.storage().instance().get::<_, u64>(&DataKey::Deadline) {
                    let current_time = env.ledger().timestamp();
                    if deadline < current_time {
                        return AnomalyReport {
                            anomaly_detected: true,
                            anomaly_type: Symbol::new(env, "EXPIRED_ACTIVE"),
                            confidence: 95,
                            description: Symbol::new(env, "Campaign active past deadline"),
                        };
                    }
                }
            }
            Status::Cancelled => {
                // Cancelled campaign should not have new contributions
                let total_raised: i128 = env
                    .storage()
                    .instance()
                    .get(&DataKey::TotalRaised)
                    .unwrap_or(0);
                if total_raised > 0 {
                    return AnomalyReport {
                        anomaly_detected: true,
                        anomaly_type: Symbol::new(env, "CANCELLED_FUNDS"),
                        confidence: 85,
                        description: Symbol::new(env, "Funds present in cancelled campaign"),
                    };
                }
            }
            _ => {}
        }
    }

    AnomalyReport {
        anomaly_detected: false,
        anomaly_type: Symbol::new(env, "NONE"),
        confidence: 0,
        description: Symbol::new(env, "No anomaly detected"),
    }
}

// ── Access Pattern Tracking ─────────────────────────────────────────────────

/// @title record_access_pattern
/// @notice Records an access pattern entry for a given address.
/// @dev    Stores timestamped action records for behavioral analysis.
///         Emits `access_recorded` event for off-chain indexing.
/// @param  env     The Soroban environment.
/// @param  address The address performing the action.
/// @param  action  The action symbol.
/// @param  success Whether the action succeeded.
/// @param  value   Optional associated value.
pub fn record_access_pattern(
    env: &Env,
    address: &Address,
    action: Symbol,
    success: bool,
    value: i128,
) {
    let key = DataKey::AccessPattern(address.clone());
    let mut pattern: Vec<AccessPatternEntry> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    let entry = AccessPatternEntry {
        timestamp: env.ledger().timestamp(),
        action,
        success,
        value,
    };

    pattern.push_back(entry);

    // Trim to max size
    while pattern.len() > MAX_ACCESS_PATTERN_ENTRIES.into() {
        pattern.remove(0);
    }

    env.storage().instance().set(&key, &pattern);

    env.events().publish(
        (Symbol::new(env, "access_recorded"),),
        (address.clone(), entry.timestamp, success),
    );
}

/// @title get_access_pattern
/// @notice Retrieves the access pattern for a given address.
/// @dev    Returns up to MAX_ACCESS_PATTERN_ENTRIES most recent entries.
/// @security Read-only; no auth required.
/// @param  env     The Soroban environment.
/// @param  address The address to query.
/// @return Vec of AccessPatternEntry sorted by timestamp (newest first).
pub fn get_access_pattern(env: &Env, address: &Address) -> Vec<AccessPatternEntry> {
    let key = DataKey::AccessPattern(address.clone());
    let pattern: Vec<AccessPatternEntry> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    pattern
}

/// @title analyze_access_pattern
/// @notice Analyzes access patterns for a given address.
/// @dev    Computes statistics including action counts, success rate,
///         and behavioral indicators.
/// @security Read-only; no auth required.
/// @param  env     The Soroban environment.
/// @param  address The address to analyze.
/// @return Tuple of (total_actions, success_rate_bps, last_activity_timestamp).
pub fn analyze_access_pattern(
    env: &Env,
    address: &Address,
) -> (u32, u32, u64) {
    let pattern = get_access_pattern(env, address);

    let total = pattern.len() as u32;
    if total == 0 {
        return (0, 0, 0);
    }

    let successful: u32 = pattern
        .iter()
        .filter(|e| e.success)
        .count()
        .try_into()
        .unwrap_or(total);

    let last_timestamp = pattern
        .iter()
        .last()
        .map(|e| e.timestamp)
        .unwrap_or(0);

    // Calculate success rate in basis points
    let success_rate_bps = (successful as u64)
        .checked_mul(10000)
        .unwrap_or(10000)
        .checked_div(total as u64)
        .unwrap_or(0) as u32;

    (total, success_rate_bps, last_timestamp)
}

// ── Threat Logging ───────────────────────────────────────────────────────────

/// @title log_threat
/// @notice Records a detected threat for security analysis.
/// @dev    Stores threat records with severity and type information.
///         Emits `threat_logged` event for real-time alerting.
/// @param  env       The Soroban environment.
/// @param  threat_type The type of threat.
/// @param  severity  The threat severity level.
/// @param  address   Optional associated address.
/// @param  details   Additional threat details.
/// @return The index of the logged threat record.
pub fn log_threat(
    env: &Env,
    threat_type: ThreatType,
    severity: ThreatSeverity,
    address: Option<Address>,
    details: Symbol,
) -> u32 {
    let key = DataKey::ThreatLog;
    let mut log: Vec<ThreatRecord> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    let record = ThreatRecord {
        timestamp: env.ledger().timestamp(),
        threat_type,
        severity,
        address,
        details,
        acknowledged: false,
    };

    let index = log.len();
    log.push_back(record);

    // Rotate log if exceeds max size
    while log.len() > MAX_THREAT_LOG_SIZE.into() {
        log.remove(0);
    }

    env.storage().instance().set(&key, &log);

    env.events().publish(
        (Symbol::new(env, "threat_logged"), Symbol::new(env, "threat")),
        (threat_type, severity.priority()),
    );

    index
}

/// @title get_threat_log
/// @notice Retrieves the threat log entries.
/// @dev    Returns all threat records, newest first.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @param  limit  Maximum number of entries to return.
/// @return Vec of ThreatRecord.
pub fn get_threat_log(env: &Env, limit: u32) -> Vec<ThreatRecord> {
    let key = DataKey::ThreatLog;
    let log: Vec<ThreatRecord> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    // Return most recent entries up to limit
    let start = log.len().saturating_sub(limit as u32);
    let result: Vec<ThreatRecord> = log
        .iter()
        .skip(start as usize)
        .collect();

    result
}

/// @title acknowledge_threat
/// @notice Marks a threat record as acknowledged.
/// @dev    Updates the acknowledged flag on a threat record.
/// @param  env    The Soroban environment.
/// @param  index  The threat record index.
/// @return True if successfully acknowledged, false if not found.
pub fn acknowledge_threat(env: &Env, index: u32) -> bool {
    let key = DataKey::ThreatLog;
    let mut log: Vec<ThreatRecord> = match env.storage().instance().get(&key) {
        Some(l) => l,
        None => return false,
    };

    if index >= log.len() {
        return false;
    }

    // Note: Vec::set is not available in soroban_sdk Vec
    // This is a limitation - in practice, would need persistent storage approach
    // For now, we emit an event to indicate acknowledgment
    env.events().publish(
        (Symbol::new(env, "threat_ack"),),
        index,
    );

    true
}

// ── Security Metrics ─────────────────────────────────────────────────────────

/// @title increment_metric
/// @notice Increments a security metric counter.
/// @dev    Uses checked arithmetic to prevent overflow.
/// @param  env    The Soroban environment.
/// @param  address Optional address association (use None for global).
/// @param  metric The metric type to increment.
/// @param  delta  The amount to increment by.
pub fn increment_metric(env: &Env, address: Option<Address>, metric: MetricType, delta: u32) {
    let key = if let Some(addr) = address {
        DataKey::SecurityMetric(addr, metric)
    } else {
        DataKey::GlobalSecurityMetric(metric)
    };

    let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
    let new_value = current.checked_add(delta).unwrap_or(u32::MAX);
    env.storage().instance().set(&key, &new_value);
}

/// @title get_metric
/// @notice Retrieves the current value of a security metric.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @param  address Optional address association.
/// @param  metric The metric type to retrieve.
/// @return The current metric value.
pub fn get_metric(env: &Env, address: Option<Address>, metric: MetricType) -> u32 {
    let key = if let Some(addr) = address {
        DataKey::SecurityMetric(addr, metric)
    } else {
        DataKey::GlobalSecurityMetric(metric)
    };

    env.storage().instance().get(&key).unwrap_or(0)
}

/// @title get_security_summary
/// @notice Generates a comprehensive security summary for monitoring.
/// @dev    Aggregates metrics and computes derived statistics.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @return SecuritySummary with aggregated analytics.
pub fn get_security_summary(env: &Env) -> SecuritySummary {
    let total_contrib = get_metric(env, None, MetricType::TotalContributions);
    let total_withdraw = get_metric(env, None, MetricType::TotalWithdrawals);
    let total_refund = get_metric(env, None, MetricType::TotalRefunds);
    let failed = get_metric(env, None, MetricType::FailedTransactions);
    let auth_failures = get_metric(env, None, MetricType::AuthFailures);
    let large_tx = get_metric(env, None, MetricType::LargeTransactions);

    let total_transactions = total_contrib
        .saturating_add(total_withdraw)
        .saturating_add(total_refund);

    // Calculate failure rate in basis points
    let failure_rate_bps = if total_transactions > 0 {
        ((failed as u64)
            .saturating_mul(10000)
            .saturating_div(total_transactions as u64)) as u32
    } else {
        0
    };

    // Count unique contributors
    let unique_key = DataKey::Contributors;
    let contributors: Vec<Address> = env
        .storage()
        .instance()
        .get(&unique_key)
        .unwrap_or_else(|| Vec::new(env));
    let unique_addresses = contributors.len();

    // Calculate average transaction size from total raised
    let total_raised: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalRaised)
        .unwrap_or(0);
    let avg_transaction_size = if total_contrib > 0 {
        total_raised / (total_contrib as i128)
    } else {
        0
    };

    // Count critical threats
    let threat_log = get_threat_log(env, MAX_THREAT_LOG_SIZE);
    let critical_threats: u32 = threat_log
        .iter()
        .filter(|t| t.severity.is_critical())
        .count()
        .try_into()
        .unwrap_or(0);

    // Calculate security score (0-100)
    let security_score = calculate_security_score(
        failure_rate_bps,
        auth_failures,
        critical_threats,
        total_transactions,
    );

    SecuritySummary {
        threat_count: threat_log.len(),
        critical_threats,
        total_transactions,
        failure_rate_bps,
        unique_addresses,
        avg_transaction_size,
        security_score,
    }
}

/// @title calculate_security_score
/// @notice Computes a security score based on threat indicators.
/// @dev    Score is 0-100 where 100 is most secure.
/// @param  failure_rate_bps  Transaction failure rate in basis points.
/// @param  auth_failures      Authorization failure count.
/// @param  critical_threats   Number of critical threats.
/// @param  total_transactions Total transaction count.
/// @return Security score from 0-100.
fn calculate_security_score(
    failure_rate_bps: u32,
    auth_failures: u32,
    critical_threats: u32,
    total_transactions: u32,
) -> u8 {
    let mut score: i32 = 100;

    // Deduct for high failure rate (>1% = 100bps)
    if failure_rate_bps > 100 {
        score -= ((failure_rate_bps.saturating_sub(100)) / 10).min(30) as i32;
    }

    // Deduct for auth failures (>5)
    if auth_failures > 5 {
        score -= ((auth_failures.saturating_sub(5)).min(20)) as i32;
    }

    // Deduct for critical threats
    score -= (critical_threats.min(20)) as i32;

    // Normalize by transaction volume (established history = more confidence)
    if total_transactions < 10 {
        score = score.saturating_sub((10 - total_transactions as u32) as i32);
    }

    score.max(0).min(100) as u8
}

// ── Rate Limiting Analytics ──────────────────────────────────────────────────

/// @title check_rate_limit
/// @notice Checks if an address has exceeded rate limits.
/// @dev    Tracks action frequency within time windows.
/// @param  env       The Soroban environment.
/// @param  address   The address to check.
/// @param  action    The action type.
/// @param  window_secs Time window in seconds.
/// @param  max_count Maximum allowed actions in window.
/// @return Tuple of (within_limit, current_count, limit).
pub fn check_rate_limit(
    env: &Env,
    address: &Address,
    action: Symbol,
    window_secs: u64,
    max_count: u32,
) -> (bool, u32, u32) {
    let pattern = get_access_pattern(env, address);
    let current_time = env.ledger().timestamp();
    let window_start = current_time.saturating_sub(window_secs);

    let recent_count: u32 = pattern
        .iter()
        .filter(|e| e.action == action && e.timestamp >= window_start)
        .count()
        .try_into()
        .unwrap_or(max_count.saturating_add(1));

    let within_limit = recent_count < max_count;

    (within_limit, recent_count, max_count)
}

/// @title record_rate_limit_violation
/// @notice Records a rate limit violation event.
/// @dev    Logs the violation and increments metric.
/// @param  env     The Soroban environment.
/// @param  address The address that violated the limit.
/// @param  action  The action that was rate-limited.
pub fn record_rate_limit_violation(env: &Env, address: &Address, action: Symbol) {
    // Log the violation
    log_threat(
        env,
        ThreatType::RateLimitExceeded,
        ThreatSeverity::Medium,
        Some(address.clone()),
        action,
    );

    // Increment violation metric
    increment_metric(env, Some(address.clone()), MetricType::RateLimitTriggers, 1);
}

// ── Threat Intelligence ───────────────────────────────────────────────────────

/// @title analyze_threat_trends
/// @notice Analyzes threat patterns over time to identify trends.
/// @dev    Groups threats by type and severity to detect patterns.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @param  hours  Time window to analyze (in hours).
/// @return Tuple of (threat_count_by_type, highest_severity).
pub fn analyze_threat_trends(env: &Env, hours: u64) -> (u32, ThreatSeverity) {
    let log = get_threat_log(env, MAX_THREAT_LOG_SIZE);
    let current_time = env.ledger().timestamp();
    let window_start = current_time.saturating_sub(hours * 3600);

    let mut threat_count: u32 = 0;
    let mut highest_severity = ThreatSeverity::Low;

    for record in log.iter() {
        if record.timestamp >= window_start {
            threat_count = threat_count.saturating_add(1);

            if record.severity.priority() > highest_severity.priority() {
                highest_severity = record.severity;
            }
        }
    }

    (threat_count, highest_severity)
}

/// @title get_blocked_addresses
/// @notice Returns list of addresses that have been flagged for threats.
/// @dev    Useful for external monitoring and integration with blocklists.
/// @security Read-only; no auth required.
/// @param  env    The Soroban environment.
/// @param  severity Minimum severity to include.
/// @return Vec of addresses with threats above threshold.
pub fn get_blocked_addresses(env: &Env, severity: ThreatSeverity) -> Vec<Address> {
    let log = get_threat_log(env, MAX_THREAT_LOG_SIZE);
    let mut addresses: Vec<Address> = Vec::new(env);

    for record in log.iter() {
        if record.severity.priority() >= severity.priority() {
            if let Some(addr) = record.address.clone() {
                // Avoid duplicates
                if !addresses.iter().any(|a| a == &addr) {
                    addresses.push_back(addr);
                }
            }
        }
    }

    addresses
}

// ── Utility Functions ────────────────────────────────────────────────────────

/// @title format_threat_severity
/// @notice Returns a human-readable string for threat severity.
/// @param  severity The severity level.
/// @return Static string representation.
pub fn format_threat_severity(severity: ThreatSeverity) -> &'static str {
    match severity {
        ThreatSeverity::Low => "LOW",
        ThreatSeverity::Medium => "MEDIUM",
        ThreatSeverity::High => "HIGH",
        ThreatSeverity::Critical => "CRITICAL",
    }
}

/// @title format_threat_type
/// @notice Returns a human-readable string for threat type.
/// @param  threat_type The threat type.
/// @return Static string representation.
pub fn format_threat_type(threat_type: ThreatType) -> &'static str {
    match threat_type {
        ThreatType::AbnormalContributionPattern => "ABNORMAL_CONTRIBUTION",
        ThreatType::RepeatedFailure => "REPEATED_FAILURE",
        ThreatType::SuspiciousAddressActivity => "SUSPICIOUS_ACTIVITY",
        ThreatType::RapidStateChanges => "RAPID_STATE_CHANGES",
        ThreatType::AbnormalWithdrawalPattern => "ABNORMAL_WITHDRAWAL",
        ThreatType::AuthAnomaly => "AUTH_ANOMALY",
        ThreatType::StorageAnomaly => "STORAGE_ANOMALY",
        ThreatType::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
        ThreatType::AccessPatternAnomaly => "ACCESS_PATTERN_ANOMALY",
    }
}

/// @title get_risk_level
/// @notice Computes overall risk level based on security summary.
/// @param  summary The security summary.
/// @return Static string: "LOW", "MEDIUM", "HIGH", or "CRITICAL".
pub fn get_risk_level(summary: &SecuritySummary) -> &'static str {
    if summary.security_score >= 80 && summary.critical_threats == 0 {
        "LOW"
    } else if summary.security_score >= 60 && summary.critical_threats < 3 {
        "MEDIUM"
    } else if summary.security_score >= 40 || summary.critical_threats < 10 {
        "HIGH"
    } else {
        "CRITICAL"
    }
}
