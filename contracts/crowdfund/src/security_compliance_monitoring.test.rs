//! Tests for security_compliance_monitoring.rs — comprehensive unit + proptest coverage.

#![allow(unused_imports)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, Address, Env, Symbol, Vec,
};

use crate::security_compliance_monitoring::{
    AuditMetrics, ComplianceIssue, ComplianceReport, check_arithmetic_safety,
    check_auth_invariant, check_platform_config, check_state_bounds, check_status_invariant,
    compliance_status, describe_violation, error_codes, run_full_audit,
};
use crate::{
    CrowdfundContract, CrowdfundContractClient, DataKey, PlatformConfig, Status,
    contract_state_size::{self, MAX_CONTRIBUTORS, MAX_ROADMAP_ITEMS},
};

// ── Setup Helpers ─────────────────────────────────────────────────────────────

fn setup_happy_path() -> (Env, CrowdfundContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let admin = creator.clone();
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();

    let now = env.ledger().timestamp();
    client.initialize(
        &amp;admin,
        &amp;creator,
        &amp;token_addr,
        &amp;1_000,
        &amp;(now + 3600),
        &amp;10,
        &amp;None,
        &amp;None,
        &amp;None,
    );

    (env, client)
}

// ── Unit Tests: Happy Path ────────────────────────────────────────────────────

#[test]
fn run_full_audit_happy_path_returns_passed_true() {
    let (env, client) = setup_happy_path();
    let report = run_full_audit(&amp;env);
    assert!(report.passed);
    assert!(report.violations.is_empty());
}

#[test]
fn compliance_status_happy_path_returns_true() {
    let (env, _) = setup_happy_path();
    assert!(compliance_status(&amp;env));
}

#[test]
fn individual_invariants_all_pass_happy_path() {
    let (env, _) = setup_happy_path();
    assert!(check_auth_invariant(&amp;env).is_ok());
    assert!(check_state_bounds(&amp;env).is_ok());
    assert!(check_arithmetic_safety(&amp;env).is_ok());
    assert!(check_status_invariant(&amp;env).is_ok());
    assert!(check_platform_config(&amp;env).is_ok());
}

// ── Unit Tests: Individual Violations ────────────────────────────────────────

#[test]
fn check_auth_invariant_fails_different_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&amp;env, &amp;contract_id);

    let creator = Address::generate(&amp;env);
    let admin = Address::generate(&amp;env); // Different!
    let token_addr = env.register_stellar_asset_contract_v2(creator.clone()).address();

    client.initialize(
        &amp;admin,
        &amp;creator,
        &amp;token_addr,
        &amp;1_000,
        &amp;3600,
        &amp;10,
        &amp;None,
        &amp;None,
        &amp;None,
    );

    let issue = check_auth_invariant(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::UNAUTHORIZED_CREATOR);
    assert_eq!(issue.description, "Admin differs from creator without explicit delegation");
}

#[test]
fn check_state_bounds_fails_contributor_overflow() {
    let (mut env, client) = setup_happy_path();
    env.mock_all_auths();

    // Fill contributors to MAX + 1
    let contributor = Address::generate(&amp;env);
    for _ in 0..MAX_CONTRIBUTORS + 1 {
        client.contribute(&amp;contributor, &amp;10);
    }

    let issue = check_state_bounds(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::CONTRIBUTOR_LIMIT_EXCEEDED);
}

#[test]
fn check_arithmetic_safety_fails_massive_overflow() {
    let (mut env, client) = setup_happy_path();
    env.mock_all_auths();

    // Contribute way over goal * 2
    let contributor = Address::generate(&amp;env);
    client.contribute(&amp;contributor, &amp;10_000_000);

    let issue = check_arithmetic_safety(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::ARITHMETIC_ANOMALY);
}

#[test]
fn check_status_invariant_fails_active_post_deadline() {
    let (mut env, client) = setup_happy_path();
    let deadline = client.deadline();
    env.ledger().set_timestamp(deadline + 1); // Past deadline

    let issue = check_status_invariant(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::INVALID_STATUS);
}

#[test]
fn check_platform_config_fails_over_100_percent_fee() {
    let (env, client) = setup_happy_path();
    
    // Manually set invalid platform config
    let bad_config = PlatformConfig {
        address: Address::generate(&amp;env),
        fee_bps: 10_001, // > 100%
    };
    env.storage().instance().set(&amp;DataKey::PlatformConfig, &amp;bad_config);

    let issue = check_platform_config(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::INVALID_PLATFORM_FEE);
}

// ── describe_violation ───────────────────────────────────────────────────────

#[test]
fn describe_violation_returns_correct_strings() {
    assert_eq!(describe_violation(error_codes::CONTRIBUTOR_LIMIT_EXCEEDED),
               "Contributors exceed MAX_CONTRIBUTORS (128)");
    assert_eq!(describe_violation(999), "Unknown compliance violation");
}

// ── Event Emission ───────────────────────────────────────────────────────────

#[test]
fn violation_event_has_correct_schema() {
    let (env, client) = setup_happy_path();
    env.mock_all_auths();

    // Force a violation
    let contributor = Address::generate(&amp;env);
    for _ in 0..MAX_CONTRIBUTORS + 1 {
        client.contribute(&amp;contributor, &amp;10);
    }

    let _ = run_full_audit(&amp;env); // Triggers log_compliance_violation

    let events = env.events().all();
    let violation_event = events.iter()
        .find(|(_, topics, _)| {
            topics.len() == 3 &amp;&amp;
            topics.get(0) == Some(&amp;Symbol::short::new(&amp;env, "security").unwrap().to_val()) &amp;&amp;
            topics.get(1) == Some(&amp;Symbol::short::new(&amp;env, "violation").unwrap().to_val())
        });

    assert!(violation_event.is_some());
}

// ── Integration: Full Audit Report ───────────────────────────────────────────

#[test]
fn full_audit_report_contains_metrics() {
    let (env, client) = setup_happy_path();
    let report = run_full_audit(&amp;env);

    let expected_stats = client.get_stats();
    let metrics = report.metrics;

    assert_eq!(metrics.total_raised, expected_stats.total_raised);
    assert_eq!(metrics.status, Status::Active);
}

// ── Proptest: Boundary Generation ────────────────────────────────────────────

#[cfg(feature = "proptest")]
proptest! {
    #![proptest_config(proptest::prelude::proptest_config::ProptestConfig::with_cases(128))]

    #[test]
    fn prop_compliance_status_deterministic(seed in 0u64..1_000) {
        let env = Env::default();
        env.mock_all_auths();
        // Setup minimal valid state
        let contract_id = env.register(CrowdfundContract, ());
        let _ = CrowdfundContractClient::new(&amp;env, &amp;contract_id).initialize(
            &amp;Address::generate(&amp;env),
            &amp;Address::generate(&amp;env),
            &amp;env.register_stellar_asset_contract_v2(Address::generate(&amp;env)).address(),
            &amp;1_000i128,
            &amp;(env.ledger().timestamp() + 3600),
            &amp;10i128,
            &amp;None,
            &amp;None,
            &amp;None,
        );
        
        // Should always pass for valid minimal state
        prop_assert!(compliance_status(&amp;env));
    }
}

// ── Gas Bounds &amp; Safety ──────────────────────────────────────────────────────

#[test]
fn contributor_scan_is_gas_bounded() {
    let (env, client) = setup_happy_path();
    
    // Fill many contributors
    env.mock_all_auths();
    for i in 0..200 {
        let addr = Address::generate(&amp;env);
        client.contribute(&amp;addr, &amp;10);
    }

    // Should not panic despite >50 contributors (scan caps at 50)
    let report = run_full_audit(&amp;env);
    assert!(!report.violations.is_empty()); // Bounds violation
}

// ── Negative Cases: Storage Manipulation ─────────────────────────────────────

#[test]
fn detects_manually_set_negative_contribution() {
    let (env, _) = setup_happy_path();
    
    let bad_contributor = Address::generate(&amp;env);
    let key = DataKey::Contribution(bad_contributor);
    env.storage().persistent().set(&amp;key, &amp;-100i128);

    let issue = check_arithmetic_safety(&amp;env).unwrap_err();
    assert_eq!(issue.code, error_codes::NEGATIVE_CONTRIBUTION);
}

