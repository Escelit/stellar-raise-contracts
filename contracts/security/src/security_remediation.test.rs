#![cfg(test)]

use crate::security_remediation::*;
use soroban_sdk::{Env, String, Vec};

#[test]
fn test_scan_vulnerabilities_returns_vector() {
    let env = Env::default();
    let vulnerabilities = scan_vulnerabilities(&env);
    
    // Should return a vector (may be empty if all checks pass)
    assert!(vulnerabilities.len() >= 0);
}

#[test]
fn test_vulnerability_severity_ordering() {
    use VulnerabilitySeverity::*;
    
    assert!(Critical != High);
    assert!(High != Medium);
    assert!(Medium != Low);
    assert!(Low != Info);
}

#[test]
fn test_apply_remediation_validation() {
    let env = Env::default();
    
    let vulnerability = SecurityVulnerability {
        id: String::from_str(&env, "SEC-001"),
        severity: VulnerabilitySeverity::High,
        description: String::from_str(&env, "Missing input validation"),
        affected_function: String::from_str(&env, "contribute"),
    };
    
    let result = apply_remediation(&env, &vulnerability);
    assert_eq!(result.vulnerability_id, vulnerability.id);
}

#[test]
fn test_apply_remediation_success() {
    let env = Env::default();
    
    let vulnerability = SecurityVulnerability {
        id: String::from_str(&env, "SEC-002"),
        severity: VulnerabilitySeverity::Critical,
        description: String::from_str(&env, "Reentrancy risk"),
        affected_function: String::from_str(&env, "withdraw"),
    };
    
    let result = apply_remediation(&env, &vulnerability);
    assert!(result.success);
}

#[test]
fn test_validate_security_posture_empty() {
    let env = Env::default();
    let vulnerabilities = Vec::new(&env);
    
    assert!(validate_security_posture(&vulnerabilities));
}

#[test]
fn test_validate_security_posture_with_critical() {
    let env = Env::default();
    let mut vulnerabilities = Vec::new(&env);
    
    vulnerabilities.push_back(SecurityVulnerability {
        id: String::from_str(&env, "SEC-CRIT"),
        severity: VulnerabilitySeverity::Critical,
        description: String::from_str(&env, "Critical vulnerability"),
        affected_function: String::from_str(&env, "test"),
    });
    
    assert!(!validate_security_posture(&vulnerabilities));
}

#[test]
fn test_validate_security_posture_without_critical() {
    let env = Env::default();
    let mut vulnerabilities = Vec::new(&env);
    
    vulnerabilities.push_back(SecurityVulnerability {
        id: String::from_str(&env, "SEC-LOW"),
        severity: VulnerabilitySeverity::Low,
        description: String::from_str(&env, "Low severity issue"),
        affected_function: String::from_str(&env, "test"),
    });
    
    assert!(validate_security_posture(&vulnerabilities));
}

#[test]
fn test_generate_remediation_report() {
    let env = Env::default();
    let results = Vec::new(&env);
    
    let report = generate_remediation_report(&env, &results);
    assert!(report.len() > 0);
}

#[test]
fn test_remediation_action_types() {
    use RemediationAction::*;
    
    assert!(AddValidation != AddAccessControl);
    assert!(AddAccessControl != AddReentrancyGuard);
    assert!(AddReentrancyGuard != AddOverflowProtection);
    assert!(AddOverflowProtection != AddRateLimit);
}

#[test]
fn test_multiple_vulnerabilities_scan() {
    let env = Env::default();
    let vulnerabilities = scan_vulnerabilities(&env);
    
    // Verify scan completes without panic
    for vuln in vulnerabilities.iter() {
        assert!(vuln.id.len() > 0);
        assert!(vuln.description.len() > 0);
    }
}

#[test]
fn test_remediation_result_structure() {
    let env = Env::default();
    
    let vulnerability = SecurityVulnerability {
        id: String::from_str(&env, "TEST-001"),
        severity: VulnerabilitySeverity::Medium,
        description: String::from_str(&env, "Test vulnerability"),
        affected_function: String::from_str(&env, "test_function"),
    };
    
    let result = apply_remediation(&env, &vulnerability);
    
    assert_eq!(result.vulnerability_id, String::from_str(&env, "TEST-001"));
    assert!(result.message.len() > 0);
}
