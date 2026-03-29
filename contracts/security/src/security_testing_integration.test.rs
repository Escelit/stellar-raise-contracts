#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_reentrancy_protection() {
    let env = Env::default();
    let contract_address = Address::generate(&env);

    let result = SecurityTester::test_reentrancy_protection(env.clone(), contract_address);

    assert!(result.passed);
    assert_eq!(result.test_name, String::from_str(&env, "Reentrancy Protection"));
    assert_eq!(result.severity, String::from_str(&env, "CRITICAL"));
}

#[test]
fn test_integer_overflow_protection_safe() {
    let env = Env::default();
    let test_value = 1000i128;

    let result = SecurityTester::test_integer_overflow_protection(env.clone(), test_value);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "HIGH"));
}

#[test]
fn test_integer_overflow_protection_max_value() {
    let env = Env::default();
    let test_value = i128::MAX;

    let result = SecurityTester::test_integer_overflow_protection(env.clone(), test_value);

    assert!(result.passed);
}

#[test]
fn test_access_control_authorized() {
    let env = Env::default();
    let authorized = Address::generate(&env);

    let result = SecurityTester::test_access_control(
        env.clone(),
        authorized.clone(),
        authorized.clone(),
    );

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "CRITICAL"));
}

#[test]
fn test_access_control_unauthorized() {
    let env = Env::default();
    let caller = Address::generate(&env);
    let authorized = Address::generate(&env);

    let result = SecurityTester::test_access_control(env.clone(), caller, authorized);

    assert!(!result.passed);
}

#[test]
fn test_timestamp_manipulation_safe() {
    let env = Env::default();
    let deadline = env.ledger().timestamp() + 86400; // 1 day

    let result = SecurityTester::test_timestamp_manipulation(env.clone(), deadline);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "MEDIUM"));
}

#[test]
fn test_timestamp_manipulation_unsafe() {
    let env = Env::default();
    let deadline = env.ledger().timestamp() + (400 * 24 * 60 * 60); // Over 1 year

    let result = SecurityTester::test_timestamp_manipulation(env.clone(), deadline);

    assert!(!result.passed);
}

#[test]
fn test_dos_protection_safe() {
    let env = Env::default();
    let operation_count = 50u32;

    let result = SecurityTester::test_dos_protection(env.clone(), operation_count);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "HIGH"));
}

#[test]
fn test_dos_protection_unsafe() {
    let env = Env::default();
    let operation_count = 150u32;

    let result = SecurityTester::test_dos_protection(env.clone(), operation_count);

    assert!(!result.passed);
}

#[test]
fn test_frontrunning_protection_enabled() {
    let env = Env::default();

    let result = SecurityTester::test_frontrunning_protection(env.clone(), true);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "MEDIUM"));
}

#[test]
fn test_frontrunning_protection_disabled() {
    let env = Env::default();

    let result = SecurityTester::test_frontrunning_protection(env.clone(), false);

    assert!(!result.passed);
}

#[test]
fn test_external_call_safety_success() {
    let env = Env::default();

    let result = SecurityTester::test_external_call_safety(env.clone(), true);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "HIGH"));
}

#[test]
fn test_external_call_safety_failure() {
    let env = Env::default();

    let result = SecurityTester::test_external_call_safety(env.clone(), false);

    assert!(!result.passed);
}

#[test]
fn test_input_validation_valid() {
    let env = Env::default();
    let input_value = 50i128;
    let min_value = 0i128;
    let max_value = 100i128;

    let result = SecurityTester::test_input_validation(
        env.clone(),
        input_value,
        min_value,
        max_value,
    );

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "HIGH"));
}

#[test]
fn test_input_validation_below_min() {
    let env = Env::default();
    let input_value = -10i128;
    let min_value = 0i128;
    let max_value = 100i128;

    let result = SecurityTester::test_input_validation(
        env.clone(),
        input_value,
        min_value,
        max_value,
    );

    assert!(!result.passed);
}

#[test]
fn test_input_validation_above_max() {
    let env = Env::default();
    let input_value = 150i128;
    let min_value = 0i128;
    let max_value = 100i128;

    let result = SecurityTester::test_input_validation(
        env.clone(),
        input_value,
        min_value,
        max_value,
    );

    assert!(!result.passed);
}

#[test]
fn test_input_validation_boundary_min() {
    let env = Env::default();
    let input_value = 0i128;
    let min_value = 0i128;
    let max_value = 100i128;

    let result = SecurityTester::test_input_validation(
        env.clone(),
        input_value,
        min_value,
        max_value,
    );

    assert!(result.passed);
}

#[test]
fn test_input_validation_boundary_max() {
    let env = Env::default();
    let input_value = 100i128;
    let min_value = 0i128;
    let max_value = 100i128;

    let result = SecurityTester::test_input_validation(
        env.clone(),
        input_value,
        min_value,
        max_value,
    );

    assert!(result.passed);
}

#[test]
fn test_error_handling_enabled() {
    let env = Env::default();

    let result = SecurityTester::test_error_handling(env.clone(), true);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "MEDIUM"));
}

#[test]
fn test_error_handling_disabled() {
    let env = Env::default();

    let result = SecurityTester::test_error_handling(env.clone(), false);

    assert!(!result.passed);
}

#[test]
fn test_storage_collision_safe() {
    let env = Env::default();

    let result = SecurityTester::test_storage_collision(env.clone(), true);

    assert!(result.passed);
    assert_eq!(result.severity, String::from_str(&env, "CRITICAL"));
}

#[test]
fn test_storage_collision_unsafe() {
    let env = Env::default();

    let result = SecurityTester::test_storage_collision(env.clone(), false);

    assert!(!result.passed);
}

#[test]
fn test_run_security_test_suite_all_pass() {
    let env = Env::default();
    let mut test_results = Vec::new(&env);

    // Add passing tests
    for i in 0..5 {
        test_results.push_back(SecurityTestResult {
            test_name: String::from_str(&env, "Test"),
            passed: true,
            severity: String::from_str(&env, "HIGH"),
            description: String::from_str(&env, "Test passed"),
        });
    }

    let suite = SecurityTester::run_security_test_suite(env.clone(), test_results);

    assert_eq!(suite.total_tests, 5);
    assert_eq!(suite.passed_tests, 5);
    assert_eq!(suite.failed_tests, 0);
    assert_eq!(suite.critical_failures, 0);
}

#[test]
fn test_run_security_test_suite_some_fail() {
    let env = Env::default();
    let mut test_results = Vec::new(&env);

    // Add 3 passing tests
    for i in 0..3 {
        test_results.push_back(SecurityTestResult {
            test_name: String::from_str(&env, "Test"),
            passed: true,
            severity: String::from_str(&env, "HIGH"),
            description: String::from_str(&env, "Test passed"),
        });
    }

    // Add 2 failing tests
    for i in 0..2 {
        test_results.push_back(SecurityTestResult {
            test_name: String::from_str(&env, "Test"),
            passed: false,
            severity: String::from_str(&env, "MEDIUM"),
            description: String::from_str(&env, "Test failed"),
        });
    }

    let suite = SecurityTester::run_security_test_suite(env.clone(), test_results);

    assert_eq!(suite.total_tests, 5);
    assert_eq!(suite.passed_tests, 3);
    assert_eq!(suite.failed_tests, 2);
    assert_eq!(suite.critical_failures, 0);
}

#[test]
fn test_run_security_test_suite_critical_failures() {
    let env = Env::default();
    let mut test_results = Vec::new(&env);

    // Add 2 passing tests
    for i in 0..2 {
        test_results.push_back(SecurityTestResult {
            test_name: String::from_str(&env, "Test"),
            passed: true,
            severity: String::from_str(&env, "HIGH"),
            description: String::from_str(&env, "Test passed"),
        });
    }

    // Add 1 critical failure
    test_results.push_back(SecurityTestResult {
        test_name: String::from_str(&env, "Critical Test"),
        passed: false,
        severity: String::from_str(&env, "CRITICAL"),
        description: String::from_str(&env, "Critical failure"),
    });

    let suite = SecurityTester::run_security_test_suite(env.clone(), test_results);

    assert_eq!(suite.total_tests, 3);
    assert_eq!(suite.passed_tests, 2);
    assert_eq!(suite.failed_tests, 1);
    assert_eq!(suite.critical_failures, 1);
}

#[test]
fn test_calculate_security_score_perfect() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 10,
        failed_tests: 0,
        critical_failures: 0,
    };

    let score = SecurityTester::calculate_security_score(&suite);

    assert_eq!(score, 100);
}

#[test]
fn test_calculate_security_score_half_pass() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 5,
        failed_tests: 5,
        critical_failures: 0,
    };

    let score = SecurityTester::calculate_security_score(&suite);

    assert_eq!(score, 50);
}

#[test]
fn test_calculate_security_score_with_critical() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 8,
        failed_tests: 2,
        critical_failures: 1,
    };

    let score = SecurityTester::calculate_security_score(&suite);

    assert_eq!(score, 70); // 80 - 10 penalty
}

#[test]
fn test_calculate_security_score_zero_tests() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 0,
        passed_tests: 0,
        failed_tests: 0,
        critical_failures: 0,
    };

    let score = SecurityTester::calculate_security_score(&suite);

    assert_eq!(score, 0);
}

#[test]
fn test_meets_security_requirements_pass() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 9,
        failed_tests: 1,
        critical_failures: 0,
    };

    let meets = SecurityTester::meets_security_requirements(&suite, 80);

    assert!(meets);
}

#[test]
fn test_meets_security_requirements_fail_score() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 5,
        failed_tests: 5,
        critical_failures: 0,
    };

    let meets = SecurityTester::meets_security_requirements(&suite, 80);

    assert!(!meets);
}

#[test]
fn test_meets_security_requirements_fail_critical() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 9,
        failed_tests: 1,
        critical_failures: 1,
    };

    let meets = SecurityTester::meets_security_requirements(&suite, 80);

    assert!(!meets); // Fails due to critical failure
}

#[test]
fn test_generate_security_report_all_pass() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 10,
        failed_tests: 0,
        critical_failures: 0,
    };

    let report = SecurityTester::generate_security_report(env.clone(), &suite);

    assert_eq!(report, String::from_str(&env, "All security tests passed"));
}

#[test]
fn test_generate_security_report_critical_issues() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 8,
        failed_tests: 2,
        critical_failures: 1,
    };

    let report = SecurityTester::generate_security_report(env.clone(), &suite);

    assert_eq!(report, String::from_str(&env, "Critical security issues detected"));
}

#[test]
fn test_generate_security_report_some_failures() {
    let env = Env::default();
    let suite = SecurityTestSuite {
        suite_name: String::from_str(&env, "Test Suite"),
        total_tests: 10,
        passed_tests: 8,
        failed_tests: 2,
        critical_failures: 0,
    };

    let report = SecurityTester::generate_security_report(env.clone(), &suite);

    assert_eq!(report, String::from_str(&env, "Some security tests failed"));
}
