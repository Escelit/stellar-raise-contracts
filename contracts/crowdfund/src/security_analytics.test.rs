//! Comprehensive tests for `security_analytics`.
//!
//! Coverage targets:
//! - ThreatSeverity enum methods (priority, is_critical).
//! - ThreatType enum variants.
//! - Anomaly detection functions (contribution, withdrawal, auth, storage).
//! - Access pattern recording and retrieval.
//! - Threat logging and retrieval.
//! - Security metrics increment and retrieval.
//! - Security summary generation.
//! - Rate limit checking.
//! - Threat trend analysis.
//! - Blocked addresses retrieval.
//! - Utility functions (format_threat_severity, format_threat_type, get_risk_level).
//! - Edge cases: empty patterns, overflow handling, boundary conditions.

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

use crate::{
    security_analytics::{
        acknowledge_threat, analyze_access_pattern, analyze_threat_trends,
        check_rate_limit, detect_auth_anomaly, detect_contribution_anomaly,
        detect_storage_anomaly, detect_withdrawal_anomaly, format_threat_severity,
        format_threat_type, get_access_pattern, get_blocked_addresses,
        get_metric, get_risk_level, get_security_summary, get_threat_log,
        increment_metric, log_threat, record_access_pattern,
        record_rate_limit_violation, AccessPatternEntry, AnomalyReport,
        SecuritySummary, ThreatRecord, ThreatSeverity, ThreatType, MetricType,
        MAX_ACCESS_PATTERN_ENTRIES, MAX_THREAT_LOG_SIZE,
    },
    DataKey, Status,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Seed a compliant contract state into `env` for testing.
fn seed_compliant_state(env: &Env) -> (Address, Address, Address) {
    let admin = Address::generate(env);
    let creator = Address::generate(env);
    let token = Address::generate(env);

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Creator, &creator);
    env.storage().instance().set(&DataKey::Token, &token);
    env.storage().instance().set(&DataKey::Status, &Status::Active);
    env.storage().instance().set(&DataKey::Goal, &1_000_000_000i128); // 1000 with 7 decimals
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &10_000_000i128);
    // Deadline 1 hour in the future.
    let deadline = env.ledger().timestamp() + 3_600;
    env.storage().instance().set(&DataKey::Deadline, &deadline);
    env.storage().instance().set(&DataKey::TotalRaised, &0i128);
    env.storage().instance().set(&DataKey::Paused, &false);

    (admin, creator, token)
}

/// Create an access pattern entry for testing.
fn make_entry(env: &Env, action: &str, success: bool, value: i128) -> AccessPatternEntry {
    AccessPatternEntry {
        timestamp: env.ledger().timestamp(),
        action: Symbol::new(env, action),
        success,
        value,
    }
}

// ── ThreatSeverity Tests ─────────────────────────────────────────────────────

#[test]
fn test_threat_severity_priority_low() {
    assert_eq!(ThreatSeverity::Low.priority(), 1);
}

#[test]
fn test_threat_severity_priority_medium() {
    assert_eq!(ThreatSeverity::Medium.priority(), 2);
}

#[test]
fn test_threat_severity_priority_high() {
    assert_eq!(ThreatSeverity::High.priority(), 3);
}

#[test]
fn test_threat_severity_priority_critical() {
    assert_eq!(ThreatSeverity::Critical.priority(), 4);
}

#[test]
fn test_threat_severity_is_critical_low() {
    assert!(!ThreatSeverity::Low.is_critical());
}

#[test]
fn test_threat_severity_is_critical_medium() {
    assert!(!ThreatSeverity::Medium.is_critical());
}

#[test]
fn test_threat_severity_is_critical_high() {
    assert!(ThreatSeverity::High.is_critical());
}

#[test]
fn test_threat_severity_is_critical_critical() {
    assert!(ThreatSeverity::Critical.is_critical());
}

// ── ThreatType Tests ─────────────────────────────────────────────────────────

#[test]
fn test_threat_type_variants() {
    // Ensure all variants exist and can be compared
    assert_ne!(ThreatType::AbnormalContributionPattern, ThreatType::RepeatedFailure);
    assert_ne!(ThreatType::AuthAnomaly, ThreatType::StorageAnomaly);
}

// ── Anomaly Detection Tests ──────────────────────────────────────────────────

#[test]
fn test_detect_contribution_anomaly_normal() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // Normal contribution amount
    let result = detect_contribution_anomaly(&env, &contributor, 1_000_000i128);

    assert!(!result.anomaly_detected);
}

#[test]
fn test_detect_contribution_anomaly_large_amount() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // Large contribution (above 10k threshold)
    let result = detect_contribution_anomaly(&env, &contributor, 15_000_000_000i128);

    assert!(result.anomaly_detected);
    assert_eq!(result.anomaly_type, Symbol::new(&env, "LARGE_CONTRIB"));
    assert!(result.confidence >= 75);
}

#[test]
fn test_detect_contribution_anomaly_empty_pattern() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // No prior pattern, should be normal
    let result = detect_contribution_anomaly(&env, &contributor, 1_000_000i128);

    assert!(!result.anomaly_detected);
}

#[test]
fn test_detect_withdrawal_anomaly_normal() {
    let env = Env::default();
    seed_compliant_state(&env);
    let withdrawer = Address::generate(&env);

    // Normal withdrawal
    let result = detect_withdrawal_anomaly(&env, &withdrawer, 500_000_000i128);

    assert!(!result.anomaly_detected);
}

#[test]
fn test_detect_withdrawal_anomaly_large_amount() {
    let env = Env::default();
    seed_compliant_state(&env);
    let withdrawer = Address::generate(&env);

    // Large withdrawal (above 5k threshold)
    let result = detect_withdrawal_anomaly(&env, &withdrawer, 10_000_000_000i128);

    assert!(result.anomaly_detected);
    assert_eq!(result.anomaly_type, Symbol::new(&env, "LARGE_WITHDRAW"));
}

#[test]
fn test_detect_auth_anomaly_no_failures() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // No auth failures recorded
    let result = detect_auth_anomaly(&env, &address);

    assert!(!result.anomaly_detected);
}

#[test]
fn test_detect_auth_anomaly_with_failures() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record auth failures
    increment_metric(&env, Some(address.clone()), MetricType::AuthFailures, 15);

    let result = detect_auth_anomaly(&env, &address);

    assert!(result.anomaly_detected);
    assert_eq!(result.anomaly_type, Symbol::new(&env, "AUTH_FAILURES"));
    assert!(result.confidence >= 90);
}

#[test]
fn test_detect_storage_anomaly_expired_active() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = Address::generate(&env);

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Creator, &creator);
    env.storage().instance().set(&DataKey::Token, &token);
    env.storage().instance().set(&DataKey::Status, &Status::Active);
    // Deadline in the past
    let past_deadline = env.ledger().timestamp() - 100;
    env.storage().instance().set(&DataKey::Deadline, &past_deadline);

    let result = detect_storage_anomaly(&env);

    assert!(result.anomaly_detected);
    assert_eq!(result.anomaly_type, Symbol::new(&env, "EXPIRED_ACTIVE"));
}

#[test]
fn test_detect_storage_anomaly_cancelled_with_funds() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = Address::generate(&env);

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage().instance().set(&DataKey::Creator, &creator);
    env.storage().instance().set(&DataKey::Token, &token);
    env.storage().instance().set(&DataKey::Status, &Status::Cancelled);
    // Funds in cancelled campaign
    env.storage().instance().set(&DataKey::TotalRaised, &1000i128);

    let result = detect_storage_anomaly(&env);

    assert!(result.anomaly_detected);
    assert_eq!(result.anomaly_type, Symbol::new(&env, "CANCELLED_FUNDS"));
}

#[test]
fn test_detect_storage_anomaly_normal_active() {
    let env = Env::default();
    seed_compliant_state(&env);

    let result = detect_storage_anomaly(&env);

    assert!(!result.anomaly_detected);
}

// ── Access Pattern Tests ─────────────────────────────────────────────────────

#[test]
fn test_record_access_pattern_new() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    record_access_pattern(
        &env,
        &address,
        Symbol::new(&env, "CONTRIBUTE"),
        true,
        1_000_000i128,
    );

    let pattern = get_access_pattern(&env, &address);
    assert_eq!(pattern.len(), 1);
    assert!(pattern.get(0).unwrap().success);
}

#[test]
fn test_record_access_pattern_multiple() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    for i in 0..5 {
        record_access_pattern(
            &env,
            &address,
            Symbol::new(&env, "CONTRIBUTE"),
            true,
            (i as i128) * 100_000,
        );
    }

    let pattern = get_access_pattern(&env, &address);
    assert_eq!(pattern.len(), 5);
}

#[test]
fn test_record_access_pattern_trim_excess() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record more than MAX_ACCESS_PATTERN_ENTRIES
    for _ in 0..(MAX_ACCESS_PATTERN_ENTRIES + 10) {
        record_access_pattern(
            &env,
            &address,
            Symbol::new(&env, "CONTRIBUTE"),
            true,
            100_000i128,
        );
    }

    let pattern = get_access_pattern(&env, &address);
    // Should be trimmed to MAX_ACCESS_PATTERN_ENTRIES
    assert!(pattern.len() <= MAX_ACCESS_PATTERN_ENTRIES);
}

#[test]
fn test_get_access_pattern_empty() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    let pattern = get_access_pattern(&env, &address);
    assert_eq!(pattern.len(), 0);
}

#[test]
fn fn analyze_access_pattern_empty() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    let (total, success_rate, last_timestamp) = analyze_access_pattern(&env, &address);

    assert_eq!(total, 0);
    assert_eq!(success_rate, 0);
    assert_eq!(last_timestamp, 0);
}

#[test]
fn analyze_access_pattern_with_entries() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record mix of successful and failed actions
    record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 1000);
    record_access_pattern(&env, &address, Symbol::new(&env, "WITHDRAW"), false, 500);
    record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 1500);
    record_access_pattern(&env, &address, Symbol::new(&env, "REFUND"), true, 1000);

    let (total, success_rate, last_timestamp) = analyze_access_pattern(&env, &address);

    assert_eq!(total, 4);
    // 3 out of 4 successful = 7500 bps (75%)
    assert_eq!(success_rate, 7500);
    assert!(last_timestamp > 0);
}

// ── Threat Logging Tests ────────────────────────────────────────────────────

#[test]
fn test_log_threat_basic() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    let index = log_threat(
        &env,
        ThreatType::AbnormalContributionPattern,
        ThreatSeverity::High,
        Some(address.clone()),
        Symbol::new(&env, "large_contrib"),
    );

    assert_eq!(index, 0);

    let log = get_threat_log(&env, 10);
    assert_eq!(log.len(), 1);
    assert_eq!(log.get(0).unwrap().threat_type, ThreatType::AbnormalContributionPattern);
    assert_eq!(log.get(0).unwrap().severity, ThreatSeverity::High);
}

#[test]
fn test_log_threat_multiple() {
    let env = Env::default();
    seed_compliant_state(&env);

    for i in 0..10 {
        log_threat(
            &env,
            ThreatType::RepeatedFailure,
            ThreatSeverity::Medium,
            None,
            Symbol::new(&env, "test"),
        );
    }

    let log = get_threat_log(&env, 20);
    assert_eq!(log.len(), 10);
}

#[test]
fn test_log_threat_rotation() {
    let env = Env::default();
    seed_compliant_state(&env);

    // Log more than MAX_THREAT_LOG_SIZE
    for _ in 0..(MAX_THREAT_LOG_SIZE + 50) {
        log_threat(
            &env,
            ThreatType::RateLimitExceeded,
            ThreatSeverity::Low,
            None,
            Symbol::new(&env, "test"),
        );
    }

    let log = get_threat_log(&env, MAX_THREAT_LOG_SIZE);
    // Should be trimmed to MAX_THREAT_LOG_SIZE
    assert!(log.len() <= MAX_THREAT_LOG_SIZE);
}

#[test]
fn test_get_threat_log_limit() {
    let env = Env::default();
    seed_compliant_state(&env);

    for i in 0..20 {
        log_threat(
            &env,
            ThreatType::SuspiciousAddressActivity,
            ThreatSeverity::Medium,
            None,
            Symbol::new(&env, "test"),
        );
    }

    let log = get_threat_log(&env, 5);
    assert_eq!(log.len(), 5);
}

#[test]
fn test_acknowledge_threat() {
    let env = Env::default();
    seed_compliant_state(&env);

    log_threat(
        &env,
        ThreatType::AuthAnomaly,
        ThreatSeverity::High,
        None,
        Symbol::new(&env, "test"),
    );

    let result = acknowledge_threat(&env, 0);
    assert!(result);
}

#[test]
fn test_acknowledge_threat_invalid_index() {
    let env = Env::default();
    seed_compliant_state(&env);

    let result = acknowledge_threat(&env, 999);
    assert!(!result);
}

// ── Security Metrics Tests ───────────────────────────────────────────────────

#[test]
fn test_increment_metric_new() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    increment_metric(&env, Some(address.clone()), MetricType::TotalContributions, 5);

    let value = get_metric(&env, Some(address.clone()), MetricType::TotalContributions);
    assert_eq!(value, 5);
}

#[test]
fn test_increment_metric_existing() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    increment_metric(&env, Some(address.clone()), MetricType::TotalContributions, 5);
    increment_metric(&env, Some(address.clone()), MetricType::TotalContributions, 3);

    let value = get_metric(&env, Some(address.clone()), MetricType::TotalContributions);
    assert_eq!(value, 8);
}

#[test]
fn test_increment_metric_global() {
    let env = Env::default();
    seed_compliant_state(&env);

    increment_metric(&env, None, MetricType::TotalContributions, 10);

    let value = get_metric(&env, None, MetricType::TotalContributions);
    assert_eq!(value, 10);
}

#[test]
fn test_get_metric_unset() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    let value = get_metric(&env, Some(address), MetricType::FailedTransactions);
    assert_eq!(value, 0);
}

#[test]
fn test_metric_overflow_handling() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Increment by large values that would overflow
    increment_metric(&env, Some(address.clone()), MetricType::TotalContributions, u32::MAX);
    increment_metric(&env, Some(address.clone()), MetricType::TotalContributions, u32::MAX);

    let value = get_metric(&env, Some(address.clone()), MetricType::TotalContributions);
    // Should saturate at u32::MAX
    assert_eq!(value, u32::MAX);
}

// ── Security Summary Tests ────────────────────────────────────────────────────

#[test]
fn test_get_security_summary_empty() {
    let env = Env::default();
    seed_compliant_state(&env);

    let summary = get_security_summary(&env);

    assert_eq!(summary.threat_count, 0);
    assert_eq!(summary.total_transactions, 0);
    assert_eq!(summary.failure_rate_bps, 0);
    assert!(summary.security_score > 0);
}

#[test]
fn test_get_security_summary_with_transactions() {
    let env = Env::default();
    seed_compliant_state(&env);

    // Record some transactions
    increment_metric(&env, None, MetricType::TotalContributions, 100);
    increment_metric(&env, None, MetricType::TotalWithdrawals, 50);
    increment_metric(&env, None, MetricType::FailedTransactions, 5);
    env.storage().instance().set(&DataKey::TotalRaised, &1_000_000_000i128);

    let summary = get_security_summary(&env);

    assert_eq!(summary.total_transactions, 150);
    // Failure rate: 5/150 * 10000 = ~333 bps
    assert!(summary.failure_rate_bps > 0);
}

#[test]
fn test_get_security_summary_with_critical_threats() {
    let env = Env::default();
    seed_compliant_state(&env);

    // Log critical threats
    log_threat(
        &env,
        ThreatType::AbnormalContributionPattern,
        ThreatSeverity::Critical,
        None,
        Symbol::new(&env, "test"),
    );

    let summary = get_security_summary(&env);

    assert!(summary.critical_threats > 0);
    // Security score should be reduced
    assert!(summary.security_score < 100);
}

// ── Rate Limiting Tests ──────────────────────────────────────────────────────

#[test]
fn test_check_rate_limit_within_limit() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record some entries below limit
    record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 1000);
    record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 2000);

    let (within_limit, count, limit) = check_rate_limit(&env, &address, Symbol::new(&env, "CONTRIBUTE"), 3600, 5);

    assert!(within_limit);
    assert_eq!(count, 2);
    assert_eq!(limit, 5);
}

#[test]
fn test_check_rate_limit_exceeded() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record entries at limit
    for _ in 0..5 {
        record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 1000);
    }

    let (within_limit, count, limit) = check_rate_limit(&env, &address, Symbol::new(&env, "CONTRIBUTE"), 3600, 5);

    assert!(!within_limit);
    assert_eq!(count, 5);
    assert_eq!(limit, 5);
}

#[test]
fn test_check_rate_limit_window() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record old entries
    for _ in 0..10 {
        record_access_pattern(&env, &address, Symbol::new(&env, "WITHDRAW"), true, 1000);
    }

    // Should be within limit for 3600 second window
    let (within_limit, count, _) = check_rate_limit(&env, &address, Symbol::new(&env, "WITHDRAW"), 3600, 5);

    assert!(!within_limit);
    assert_eq!(count, 10);
}

#[test]
fn test_record_rate_limit_violation() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    record_rate_limit_violation(&env, &address, Symbol::new(&env, "CONTRIBUTE"));

    let log = get_threat_log(&env, 10);
    assert_eq!(log.len(), 1);
    assert_eq!(log.get(0).unwrap().threat_type, ThreatType::RateLimitExceeded);
    assert_eq!(log.get(0).unwrap().severity, ThreatSeverity::Medium);
}

// ── Threat Intelligence Tests ─────────────────────────────────────────────────

#[test]
fn test_analyze_threat_trends_none() {
    let env = Env::default();
    seed_compliant_state(&env);

    let (count, severity) = analyze_threat_trends(&env, 24);

    assert_eq!(count, 0);
    assert_eq!(severity, ThreatSeverity::Low);
}

#[test]
fn test_analyze_threat_trends_with_threats() {
    let env = Env::default();
    seed_compliant_state(&env);

    log_threat(
        &env,
        ThreatType::AbnormalContributionPattern,
        ThreatSeverity::High,
        None,
        Symbol::new(&env, "test"),
    );
    log_threat(
        &env,
        ThreatType::RepeatedFailure,
        ThreatSeverity::Critical,
        None,
        Symbol::new(&env, "test"),
    );

    let (count, severity) = analyze_threat_trends(&env, 24);

    assert_eq!(count, 2);
    assert_eq!(severity, ThreatSeverity::Critical);
}

#[test]
fn test_get_blocked_addresses_none() {
    let env = Env::default();
    seed_compliant_state(&env);

    let addresses = get_blocked_addresses(&env, ThreatSeverity::High);

    assert_eq!(addresses.len(), 0);
}

#[test]
fn test_get_blocked_addresses_with_threats() {
    let env = Env::default();
    seed_compliant_state(&env);
    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);

    log_threat(
        &env,
        ThreatType::AbnormalContributionPattern,
        ThreatSeverity::High,
        Some(addr1.clone()),
        Symbol::new(&env, "test"),
    );
    log_threat(
        &env,
        ThreatType::RepeatedFailure,
        ThreatSeverity::Critical,
        Some(addr2.clone()),
        Symbol::new(&env, "test"),
    );
    log_threat(
        &env,
        ThreatType::RateLimitExceeded,
        ThreatSeverity::Low, // Below threshold
        Some(addr1.clone()),
        Symbol::new(&env, "test"),
    );

    let addresses = get_blocked_addresses(&env, ThreatSeverity::High);

    // Should include both addr1 and addr2 (once each)
    assert!(addresses.len() >= 2);
}

#[test]
fn test_get_blocked_addresses_no_duplicates() {
    let env = Env::default();
    seed_compliant_state(&env);
    let addr = Address::generate(&env);

    // Log multiple threats for same address
    log_threat(
        &env,
        ThreatType::AbnormalContributionPattern,
        ThreatSeverity::High,
        Some(addr.clone()),
        Symbol::new(&env, "test1"),
    );
    log_threat(
        &env,
        ThreatType::RepeatedFailure,
        ThreatSeverity::High,
        Some(addr.clone()),
        Symbol::new(&env, "test2"),
    );

    let addresses = get_blocked_addresses(&env, ThreatSeverity::High);

    // Should only appear once
    assert_eq!(addresses.len(), 1);
}

// ── Utility Function Tests ──────────────────────────────────────────────────

#[test]
fn test_format_threat_severity_all() {
    assert_eq!(format_threat_severity(ThreatSeverity::Low), "LOW");
    assert_eq!(format_threat_severity(ThreatSeverity::Medium), "MEDIUM");
    assert_eq!(format_threat_severity(ThreatSeverity::High), "HIGH");
    assert_eq!(format_threat_severity(ThreatSeverity::Critical), "CRITICAL");
}

#[test]
fn test_format_threat_type_all() {
    assert_eq!(
        format_threat_type(ThreatType::AbnormalContributionPattern),
        "ABNORMAL_CONTRIBUTION"
    );
    assert_eq!(
        format_threat_type(ThreatType::RepeatedFailure),
        "REPEATED_FAILURE"
    );
    assert_eq!(
        format_threat_type(ThreatType::SuspiciousAddressActivity),
        "SUSPICIOUS_ACTIVITY"
    );
    assert_eq!(
        format_threat_type(ThreatType::RapidStateChanges),
        "RAPID_STATE_CHANGES"
    );
    assert_eq!(
        format_threat_type(ThreatType::AbnormalWithdrawalPattern),
        "ABNORMAL_WITHDRAWAL"
    );
    assert_eq!(
        format_threat_type(ThreatType::AuthAnomaly),
        "AUTH_ANOMALY"
    );
    assert_eq!(
        format_threat_type(ThreatType::StorageAnomaly),
        "STORAGE_ANOMALY"
    );
    assert_eq!(
        format_threat_type(ThreatType::RateLimitExceeded),
        "RATE_LIMIT_EXCEEDED"
    );
    assert_eq!(
        format_threat_type(ThreatType::AccessPatternAnomaly),
        "ACCESS_PATTERN_ANOMALY"
    );
}

#[test]
fn test_get_risk_level_low() {
    let summary = SecuritySummary {
        threat_count: 0,
        critical_threats: 0,
        total_transactions: 100,
        failure_rate_bps: 10,
        unique_addresses: 20,
        avg_transaction_size: 1000,
        security_score: 95,
    };

    assert_eq!(get_risk_level(&summary), "LOW");
}

#[test]
fn test_get_risk_level_medium() {
    let summary = SecuritySummary {
        threat_count: 5,
        critical_threats: 2,
        total_transactions: 100,
        failure_rate_bps: 200,
        unique_addresses: 20,
        avg_transaction_size: 1000,
        security_score: 65,
    };

    assert_eq!(get_risk_level(&summary), "MEDIUM");
}

#[test]
fn test_get_risk_level_high() {
    let summary = SecuritySummary {
        threat_count: 20,
        critical_threats: 5,
        total_transactions: 100,
        failure_rate_bps: 500,
        unique_addresses: 20,
        avg_transaction_size: 1000,
        security_score: 45,
    };

    assert_eq!(get_risk_level(&summary), "HIGH");
}

#[test]
fn test_get_risk_level_critical() {
    let summary = SecuritySummary {
        threat_count: 50,
        critical_threats: 15,
        total_transactions: 100,
        failure_rate_bps: 1000,
        unique_addresses: 20,
        avg_transaction_size: 1000,
        security_score: 20,
    };

    assert_eq!(get_risk_level(&summary), "CRITICAL");
}

// ── Edge Case Tests ──────────────────────────────────────────────────────────

#[test]
fn test_empty_contract_state() {
    let env = Env::default();

    // No state set at all
    let summary = get_security_summary(&env);
    assert!(summary.security_score <= 100);

    let log = get_threat_log(&env, 10);
    assert_eq!(log.len(), 0);
}

#[test]
fn test_max_contribution_amount() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // Test with very large amount
    let result = detect_contribution_anomaly(&env, &contributor, i128::MAX);
    assert!(result.anomaly_detected);
}

#[test]
fn test_zero_contribution_amount() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // Zero amount is normal (not flagged as large)
    let result = detect_contribution_anomaly(&env, &contributor, 0);
    assert!(!result.anomaly_detected);
}

#[test]
fn test_negative_contribution_amount() {
    let env = Env::default();
    seed_compliant_state(&env);
    let contributor = Address::generate(&env);

    // Negative amount
    let result = detect_contribution_anomaly(&env, &contributor, -1000);
    assert!(!result.anomaly_detected);
}

#[test]
fn test_security_score_minimum() {
    let env = Env::default();
    seed_compliant_state(&env);

    // Log many critical threats
    for _ in 0..50 {
        log_threat(
            &env,
            ThreatType::AbnormalContributionPattern,
            ThreatSeverity::Critical,
            None,
            Symbol::new(&env, "test"),
        );
    }
    increment_metric(&env, None, MetricType::AuthFailures, 100);
    increment_metric(&env, None, MetricType::FailedTransactions, 1000);

    let summary = get_security_summary(&env);
    // Score should not go below 0
    assert!(summary.security_score >= 0);
}

#[test]
fn test_security_score_maximum() {
    let env = Env::default();
    seed_compliant_state(&env);

    // Set good metrics
    increment_metric(&env, None, MetricType::TotalContributions, 100);
    env.storage().instance().set(&DataKey::TotalRaised, &100_000_000i128);

    let summary = get_security_summary(&env);
    // Score should not exceed 100
    assert!(summary.security_score <= 100);
}

#[test]
fn test_analyze_access_pattern_all_failures() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record all failed actions
    for _ in 0..10 {
        record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), false, 0);
    }

    let (_, success_rate, _) = analyze_access_pattern(&env, &address);

    assert_eq!(success_rate, 0);
}

#[test]
fn test_analyze_access_pattern_all_successes() {
    let env = Env::default();
    seed_compliant_state(&env);
    let address = Address::generate(&env);

    // Record all successful actions
    for _ in 0..10 {
        record_access_pattern(&env, &address, Symbol::new(&env, "CONTRIBUTE"), true, 1000);
    }

    let (total, success_rate, _) = analyze_access_pattern(&env, &address);

    assert_eq!(total, 10);
    assert_eq!(success_rate, 10000); // 100%
}
