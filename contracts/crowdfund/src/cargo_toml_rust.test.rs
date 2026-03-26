//! Comprehensive tests for `cargo_toml_rust` — dependency management and CI/CD standardization.
//!
//! ## Security notes
//! - Version constants are pinned; any accidental change is caught immediately
//! - Security validation prevents unauthorized dependencies
//! - Compliance rules are automatically enforced
//! - Audit trail maintains complete dependency history
//! - Dev-only dependencies are properly isolated from production

#[allow(deprecated)]
use crate::cargo_toml_rust::{
    all_deprecated_versions_replaced, audited_dependencies, CargoTomlRust, ComplianceRule, DataKey,
    DepRecord, DependencyInfo, SecurityPolicy, PROPTEST_VERSION, PROPTEST_VERSION_DEPRECATED,
    SOROBAN_SDK_VERSION, SOROBAN_SDK_VERSION_DEPRECATED,
};
use soroban_sdk::{Env, Map, String, Vec};
//! Tests for `cargo_toml_rust` — dependency policy enforcement.
//!
//! ## Security notes
//! - Version constants are pinned; any accidental change is caught immediately
//! - Security validation prevents unauthorized dependencies
//! - Compliance rules are automatically enforced
//! - Audit trail maintains complete dependency history
//! - Dev-only dependencies are properly isolated from production

#![cfg(test)]

use soroban_sdk::{Env, String, Vec, Map};
use crate::cargo_toml_rust::{
    all_deprecated_versions_replaced, audited_dependencies, DepRecord,
    PROPTEST_VERSION, PROPTEST_VERSION_DEPRECATED, SOROBAN_SDK_VERSION,
    SOROBAN_SDK_VERSION_DEPRECATED, CargoTomlRust, DataKey, DependencyInfo,
    SecurityPolicy, ComplianceRule,
};

// ── Version constant stability ────────────────────────────────────────────────

#[test]
fn soroban_sdk_version_is_pinned() {
    assert_eq!(SOROBAN_SDK_VERSION, "22.1.0");
    assert_eq!(SOROBAN_SDK_VERSION, "22.0.11");
}

#[test]
fn soroban_sdk_deprecated_version_is_recorded() {
    #[allow(deprecated)]
    let v = SOROBAN_SDK_VERSION_DEPRECATED;
    assert_eq!(v, "22.0.11");
}

#[test]
fn proptest_version_is_pinned() {
    assert_eq!(PROPTEST_VERSION, "1.5.0");
}

// ── audited_dependencies (backward compatibility) ────────────────────────────────
    assert_eq!(PROPTEST_VERSION, "1.11.0");
}

#[test]
fn proptest_deprecated_version_is_recorded() {
    #[allow(deprecated)]
    let v = PROPTEST_VERSION_DEPRECATED;
    assert_eq!(v, "1.4");
}

// ── audited_dependencies (backward compatibility) ────────────────────────────────

#[test]
fn audited_dependencies_has_two_entries() {
    assert_eq!(audited_dependencies().len(), 2);
}

#[test]
fn soroban_sdk_dep_is_not_dev_only() {
    let deps = audited_dependencies();
    let sdk = deps.iter().find(|d| d.name == "soroban-sdk").unwrap();
    assert!(!sdk.dev_only);
}

#[test]
fn soroban_sdk_dep_version_matches_constant() {
    let deps = audited_dependencies();
    let sdk = deps.iter().find(|d| d.name == "soroban-sdk").unwrap();
    assert_eq!(sdk.version, SOROBAN_SDK_VERSION);
}

#[test]
fn soroban_sdk_dep_marks_previous_as_deprecated() {
    let deps = audited_dependencies();
    let sdk = deps.iter().find(|d| d.name == "soroban-sdk").unwrap();
    assert!(sdk.deprecated_previous);
}

#[test]
fn proptest_dep_is_dev_only() {
    let deps = audited_dependencies();
    let pt = deps.iter().find(|d| d.name == "proptest").unwrap();
    assert!(pt.dev_only);
}

#[test]
fn proptest_dep_version_matches_constant() {
    let deps = audited_dependencies();
    let pt = deps.iter().find(|d| d.name == "proptest").unwrap();
    assert_eq!(pt.version, PROPTEST_VERSION);
}

#[test]
fn proptest_dep_marks_previous_as_deprecated() {
    let deps = audited_dependencies();
    let pt = deps.iter().find(|d| d.name == "proptest").unwrap();
    assert!(pt.deprecated_previous);
}

// ── all_deprecated_versions_replaced ─────────────────────────────────────────

#[test]
fn all_deprecated_versions_replaced_returns_true() {
    assert!(all_deprecated_versions_replaced());
}

#[test]
fn dep_record_with_no_deprecated_previous_fails_check() {
    let dep = DepRecord {
        name: "some-crate",
        version: "1.0.0",
        dev_only: false,
        deprecated_previous: false,
    };
    assert!(!dep.deprecated_previous);
}

// ── DepRecord equality ────────────────────────────────────────────────────────

#[test]
fn dep_record_equality() {
    let a = DepRecord {
        name: "soroban-sdk",
        version: "22.1.0",
        version: "22.0.11",
        dev_only: false,
        deprecated_previous: true,
    };
    let b = DepRecord {
        name: "soroban-sdk",
        version: "22.1.0",
        version: "22.0.11",
        dev_only: false,
        deprecated_previous: true,
    };
    assert_eq!(a, b);
}

#[test]
fn dep_record_inequality_on_version() {
    let a = DepRecord {
        name: "soroban-sdk",
        version: "22.0.11",
        version: "22.0.1",
        dev_only: false,
        deprecated_previous: true,
    };
    let b = DepRecord {
        name: "soroban-sdk",
        version: "22.1.0",
        version: "22.0.11",
        dev_only: false,
        deprecated_previous: true,
    };
    assert_ne!(a, b);
}

// ── Contract Integration Tests ─────────────────────────────────────────────────

fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

#[test]
fn contract_initialization() {
    let env = create_test_env();

    assert!(!env.storage().instance().has(&DataKey::SecurityPolicies));

    CargoTomlRust::initialize(env.clone());

    
    // Contract should not be initialized initially
    assert!(!env.storage().instance().has(&DataKey::SecurityPolicies));

    CargoTomlRust::initialize(env.clone());

    assert!(env.storage().instance().has(&DataKey::SecurityPolicies));
    assert!(env.storage().instance().has(&DataKey::ApprovedDependencies));
    assert!(env.storage().instance().has(&DataKey::DependencyVersions));
    assert!(env.storage().instance().has(&DataKey::ComplianceRules));

    
    // Verify default security policy
    let policy = CargoTomlRust::get_security_policy(env.clone());
    assert_eq!(policy.max_security_level, 3);
    assert!(policy.require_audit);
    assert!(policy.auto_update_dev_deps);
    assert_eq!(policy.allowed_licenses.len(), 4);

    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 2);
    assert_eq!(policy.allowed_licenses.len(), 4); // MIT, Apache-2.0, BSD-3-Clause, 0BSD
    
    // Verify default compliance rules
    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 2);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn contract_double_initialization_panics() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());
    CargoTomlRust::initialize(env);
}

#[test]
fn add_approved_dependency_success() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2,
        1640995200,
        false,
        2, // security level
        1640995200, // timestamp
        false, // not dev-only
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    let dep = deps.get(0).unwrap();
    assert_eq!(dep.name, String::from_str(&env, "soroban-sdk"));
    assert_eq!(dep.version, String::from_str(&env, "22.1.0"));
    assert_eq!(dep.security_level, 2);
    assert!(dep.approved);
    assert!(!dep.dev_only);

    
    // Add soroban-sdk dependency
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2,
        1640995200,
        false,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    let dep = deps.get(0).unwrap();
    assert_eq!(dep.name, String::from_str(&env, "soroban-sdk"));
    assert_eq!(dep.version, String::from_str(&env, "22.1.0"));
    assert_eq!(dep.security_level, 2);
    assert!(dep.approved);
    assert!(!dep.dev_only);

    let versions = CargoTomlRust::get_dependency_versions(env.clone());
    assert_eq!(versions.len(), 1);
    assert_eq!(
        versions.get(String::from_str(&env, "soroban-sdk")).unwrap(),
        String::from_str(&env, "22.1.0")
        String::from_str(&env, "22.0.11")
    );
}

#[test]
#[should_panic(expected = "Security level 5 exceeds maximum allowed 3")]
fn add_dependency_exceeding_security_level_panics() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "risky-crate"),
        String::from_str(&env, "1.0.0"),
        5,
    
    // Try to add dependency with security level > max
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "risky-crate"),
        String::from_str(&env, "1.0.0"),
        5,
        1640995200,
        false,
    );
}

#[test]
fn add_dev_dependency_auto_approval() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "proptest"),
        String::from_str(&env, "1.5.0"),
        1,
        1640995200,
        true,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    
    // Add dev dependency (should be auto-approved)
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "proptest"),
        String::from_str(&env, "1.5.0"),
        1,
        1640995200,
        true,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    let dep = deps.get(0).unwrap();
    assert!(dep.approved);
    assert!(dep.dev_only);
}

#[test]
fn validate_dependency_success() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
    
    // Add a dependency first
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2,
        1640995200,
        false,
    );

    assert!(CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2
    ));

    assert!(!CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.0.11"),
    
    // Validation should succeed
    assert!(CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2
    ));

    assert!(!CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.0.11"),
        2
    ));
}

#[test]
fn validate_dependency_fails_for_blocked() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::block_dependency(env.clone(), String::from_str(&env, "blocked-crate"));

    
    // Block a dependency first
    CargoTomlRust::block_dependency(env.clone(), String::from_str(&env, "blocked-crate"));

    assert!(!CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "blocked-crate"),
        String::from_str(&env, "1.0.0"),
        1
    ));
}

#[test]
fn block_dependency_functionality() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Add a dependency first
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.0.0"),
        2,
        1640995200,
        false,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    CargoTomlRust::block_dependency(env.clone(), String::from_str(&env, "test-crate"));

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 0);

    let policy = CargoTomlRust::get_security_policy(env.clone());
    assert!(policy
        .blocked_crates
        .contains(&String::from_str(&env, "test-crate")));
    
    // Verify it's in approved list
    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    CargoTomlRust::block_dependency(env.clone(), String::from_str(&env, "test-crate"));

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 0);

    let policy = CargoTomlRust::get_security_policy(env.clone());
    assert!(policy.blocked_crates.contains(&String::from_str(&env, "test-crate")));
}

#[test]
fn update_security_policy() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    let new_policy = SecurityPolicy {
        max_security_level: 2,
        require_audit: false,
        allowed_licenses: Vec::from_array(
            &env,
            [
                String::from_str(&env, "MIT"),
                String::from_str(&env, "Apache-2.0"),
            ],
        ),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: false,
    };

    CargoTomlRust::update_security_policy(env.clone(), new_policy);

    
    // Create new policy with stricter settings
    let new_policy = SecurityPolicy {
        max_security_level: 2,
        require_audit: false,
        allowed_licenses: Vec::from_array(&env, [
            String::from_str(&env, "MIT"),
            String::from_str(&env, "Apache-2.0"),
        ]),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: false,
    };

    CargoTomlRust::update_security_policy(env.clone(), new_policy);

    let current_policy = CargoTomlRust::get_security_policy(env.clone());
    assert_eq!(current_policy.max_security_level, 2);
    assert!(!current_policy.require_audit);
    assert_eq!(current_policy.allowed_licenses.len(), 2);
    assert!(!current_policy.auto_update_dev_deps);
}

#[test]
fn add_compliance_rule() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Add new compliance rule
    let new_rule = ComplianceRule {
        rule_name: String::from_str(&env, "license_check"),
        description: String::from_str(&env, "Validate dependency licenses"),
        check_type: String::from_str(&env, "license"),
        enabled: true,
        severity: String::from_str(&env, "warning"),
    };

    CargoTomlRust::add_compliance_rule(env.clone(), new_rule);

    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 3);

    let added_rule = rules
        .iter()
        .find(|r| r.rule_name == String::from_str(&env, "license_check"))
        .unwrap();
    
    CargoTomlRust::add_compliance_rule(env.clone(), new_rule.clone());
    
    // Verify rule was added
    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 3);

    let added_rule = rules.iter().find(|r| r.rule_name == String::from_str(&env, "license_check")).unwrap();
    assert_eq!(added_rule.check_type, String::from_str(&env, "license"));
    assert_eq!(added_rule.severity, String::from_str(&env, "warning"));
}

#[test]
fn update_existing_compliance_rule() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Update existing rule
    let updated_rule = ComplianceRule {
        rule_name: String::from_str(&env, "version_check"),
        description: String::from_str(&env, "Updated version check description"),
        check_type: String::from_str(&env, "version"),
        enabled: false,
        severity: String::from_str(&env, "warning"),
    };

    CargoTomlRust::add_compliance_rule(env.clone(), updated_rule);

    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 2); // still 2, not duplicated

    let version_rule = rules
        .iter()
        .find(|r| r.rule_name == String::from_str(&env, "version_check"))
        .unwrap();
        enabled: false, // disable it
        severity: String::from_str(&env, "warning"), // change severity
    };

    CargoTomlRust::add_compliance_rule(env.clone(), updated_rule);

    let rules = CargoTomlRust::get_compliance_rules(env.clone());
    assert_eq!(rules.len(), 2); // still 2, not duplicated

    let version_rule = rules.iter().find(|r| r.rule_name == String::from_str(&env, "version_check")).unwrap();
    assert!(!version_rule.enabled);
    assert_eq!(version_rule.severity, String::from_str(&env, "warning"));
}

#[test]
fn is_dependency_up_to_date() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Add a dependency
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.2.3"),
        2,
        1640995200,
        false,
    );

    
    // Should be up to date
    assert!(CargoTomlRust::is_dependency_up_to_date(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.2.3")
    ));

    
    // Should not be up to date with different version
    assert!(!CargoTomlRust::is_dependency_up_to_date(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.2.2")
    ));

    
    // Should return false for unknown dependency
    assert!(!CargoTomlRust::is_dependency_up_to_date(
        env.clone(),
        String::from_str(&env, "unknown-crate"),
        String::from_str(&env, "1.0.0")
    ));
}

#[test]
fn run_compliance_check_all_passing() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2,
        2, // within max level 3
        1640995200,
        false,
    );

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "proptest"),
        String::from_str(&env, "1.5.0"),
        1,
        1640995200,
        true,
    );

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    for (rule_name, passed, message) in results.iter() {
        assert!(passed, "A compliance rule failed");
    
    // Add some compliant dependencies
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "soroban-sdk"),
        String::from_str(&env, "22.1.0"),
        2,
        1640995200,
        false,
    );

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "proptest"),
        String::from_str(&env, "1.5.0"),
        1,
        1640995200,
        true,
    );

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    for (rule_name, passed, message) in results.iter() {
        assert!(passed, "Rule {} should pass: {}", rule_name, message);
    }
}

#[test]
fn run_compliance_check_security_failure() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    // Raise the max security level so we can add a high-risk dep, then lower it
    // to simulate a policy tightening scenario.
    let permissive_policy = SecurityPolicy {
        max_security_level: 5,
        require_audit: true,
        allowed_licenses: Vec::from_array(&env, [String::from_str(&env, "MIT")]),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: true,
    };
    CargoTomlRust::update_security_policy(env.clone(), permissive_policy);

fn run_compliance_check_with_failures() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    // Raise the max security level so we can add a high-risk dep, then lower it
    // to simulate a policy tightening scenario.
    let permissive_policy = SecurityPolicy {
        max_security_level: 5,
        require_audit: true,
        allowed_licenses: Vec::from_array(&env, [
            String::from_str(&env, "MIT"),
        ]),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: true,
    };
    CargoTomlRust::update_security_policy(env.clone(), permissive_policy);

    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "risky-crate"),
        String::from_str(&env, "1.0.0"),
        5,
        1640995200,
        false,
    );

    // Tighten the policy back to max level 3
    let strict_policy = SecurityPolicy {
        max_security_level: 3,
        require_audit: true,
        allowed_licenses: Vec::from_array(&env, [String::from_str(&env, "MIT")]),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: true,
    };
    CargoTomlRust::update_security_policy(env.clone(), strict_policy);

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    let security_result = results
        .iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "security_validation"))
        .unwrap();

    assert!(!security_result.1);
    assert!(security_result
        .2
        .contains("dependencies exceed maximum security level"));
        5, // exceeds max level 3
        1640995200,
        false,
    );

    // Tighten the policy back to max level 3
    let strict_policy = SecurityPolicy {
        max_security_level: 3,
        require_audit: true,
        allowed_licenses: Vec::from_array(&env, [
            String::from_str(&env, "MIT"),
        ]),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: true,
    };
    CargoTomlRust::update_security_policy(env.clone(), strict_policy);

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    let security_result = results.iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "security_validation"))
        .unwrap();

    assert!(!security_result.1);
    assert!(security_result.2.contains("dependencies exceed maximum security level"));
}

#[test]
fn dependency_update_functionality() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Add initial dependency
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.0.0"),
        2,
        1640995200,
        false,
    );

    
    // Update the same dependency with new version
    CargoTomlRust::add_approved_dependency(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.1.0"),
        2,
        1640995300,
        false,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    let dep = deps.get(0).unwrap();
    assert_eq!(dep.version, String::from_str(&env, "1.1.0"));
    assert_eq!(dep.last_updated, 1640995300);

        1640995300, // different timestamp
        false,
    );

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 1);

    let dep = deps.get(0).unwrap();
    assert_eq!(dep.version, String::from_str(&env, "1.1.0"));
    assert_eq!(dep.last_updated, 1640995300);

    let versions = CargoTomlRust::get_dependency_versions(env.clone());
    assert_eq!(
        versions.get(String::from_str(&env, "test-crate")).unwrap(),
        String::from_str(&env, "1.1.0")
    );
}

#[test]
fn edge_case_empty_dependency_lists() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 0);

    let versions = CargoTomlRust::get_dependency_versions(env.clone());
    assert_eq!(versions.len(), 0);

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    let version_result = results
        .iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "version_check"))
        .unwrap();
    
    // Test with empty approved dependencies
    let deps = CargoTomlRust::get_approved_dependencies(env.clone());
    assert_eq!(deps.len(), 0);

    let versions = CargoTomlRust::get_dependency_versions(env.clone());
    assert_eq!(versions.len(), 0);

    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 2);

    let version_result = results.iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "version_check"))
        .unwrap();
    assert!(version_result.1);
}

#[test]
fn security_policy_edge_cases() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    let strict_policy = SecurityPolicy {
        max_security_level: 0,
        require_audit: true,
        allowed_licenses: Vec::new(&env),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: false,
    };

    CargoTomlRust::update_security_policy(env.clone(), strict_policy);

    
    // Test with zero max security level
    let strict_policy = SecurityPolicy {
        max_security_level: 0,
        require_audit: true,
        allowed_licenses: Vec::new(&env),
        blocked_crates: Vec::new(&env),
        auto_update_dev_deps: false,
    };

    CargoTomlRust::update_security_policy(env.clone(), strict_policy);

    assert!(!CargoTomlRust::validate_dependency(
        env.clone(),
        String::from_str(&env, "test-crate"),
        String::from_str(&env, "1.0.0"),
        1
    ));
}

#[test]
fn compliance_rule_edge_cases() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Add rule with unknown check type
    let unknown_rule = ComplianceRule {
        rule_name: String::from_str(&env, "unknown_check"),
        description: String::from_str(&env, "Unknown check type"),
        check_type: String::from_str(&env, "unknown"),
        enabled: true,
        severity: String::from_str(&env, "error"),
    };

    CargoTomlRust::add_compliance_rule(env.clone(), unknown_rule);

    let results = CargoTomlRust::run_compliance_check(env.clone());

    let unknown_result = results
        .iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "unknown_check"))
        .unwrap();

    assert!(!unknown_result.1);
    // Accept any non-empty failure message for unknown rule type
    assert!(unknown_result.2.len() > 0);
    
    CargoTomlRust::add_compliance_rule(env.clone(), unknown_rule);

    let results = CargoTomlRust::run_compliance_check(env.clone());

    let unknown_result = results.iter()
        .find(|(name, _, _)| name == &String::from_str(&env, "unknown_check"))
        .unwrap();

    assert!(!unknown_result.1);
    assert!(unknown_result.2.contains("Unknown rule type"));
}

#[test]
fn disabled_compliance_rules_are_skipped() {
    let env = create_test_env();
    CargoTomlRust::initialize(env.clone());

    
    // Disable version_check rule
    let disabled_rule = ComplianceRule {
        rule_name: String::from_str(&env, "version_check"),
        description: String::from_str(&env, "Disabled version check"),
        check_type: String::from_str(&env, "version"),
        enabled: false,
        severity: String::from_str(&env, "error"),
    };

    CargoTomlRust::add_compliance_rule(env.clone(), disabled_rule);

    // Disabled rules are skipped in run_compliance_check, so result count drops to 1
    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 1); // only security_validation runs
    
    CargoTomlRust::add_compliance_rule(env.clone(), disabled_rule);

    // Disabled rules are skipped in run_compliance_check, so result count drops to 1
    let results = CargoTomlRust::run_compliance_check(env.clone());
    assert_eq!(results.len(), 1); // only security_validation runs
}
