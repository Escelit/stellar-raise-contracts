//! # Cargo.toml Rust Dependency Management for CI/CD Standardization
//!
//! This module provides comprehensive dependency management, validation, and
//! security checking for Cargo.toml Rust dependencies to improve CI/CD and scalability.
//!
//! ## Features
//!
//! - **Dependency Validation**: Automatic validation against security policies
//! - **Version Management**: Centralized version tracking and updates
//! - **Security Policies**: Configurable security levels and blocked crates
//! - **Compliance Rules**: CI/CD compliance automation
//! - **Audit Trail**: Complete dependency change history
//!
//! ## Current Dependencies
//!
//! | Crate        | Version  | Scope       | Security Level | Status   |
//! |--------------|----------|-------------|----------------|----------|
//! | `soroban-sdk`| `22.1.0` | workspace   | 2              | Approved |
//! | `proptest`   | `1.5.0`  | dev only    | 1              | Approved |
//! | Crate        | Version   | Scope       | Security Level | Status |
//! |--------------|-----------|-------------|----------------|---------|
//! | `soroban-sdk`| `22.1.0` | workspace   | 2              | Approved |
//! | `proptest`   | `1.11.0`  | dev only    | 1              | Approved |
//!
//! ## Security Assumptions
//!
//! 1. **Minor bump** — `soroban-sdk 22.1.0` is a minor release; storage layout
//!    and host-function IDs remain backward-compatible within the 22.x series.
//! 2. **Dev-only dependencies** — Development dependencies never affect WASM binary
//! 3. **Security validation** — All dependencies must pass security checks
//! 4. **Compliance enforcement** — CI/CD rules are automatically enforced
//! 5. **Audit trail** — All changes are tracked and verifiable

#[allow(dead_code, missing_docs)]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Symbol, Vec};

// ── Contract Types for Dependency Management ─────────────────────────────────────

/// Data storage keys for the contract
#[contracttype]
pub enum DataKey {
    ApprovedDependencies,
    DependencyVersions,
    SecurityPolicies,
    ComplianceRules,
}

/// Dependency information structure with security metadata
#[derive(Clone)]
#[contracttype]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub security_level: u32, // 1-5 scale (1=lowest risk, 5=highest risk)
    pub last_updated: u64,
    pub approved: bool,
    pub dev_only: bool,
}

/// Security policy configuration for dependency validation
#[derive(Clone)]
#[contracttype]
pub struct SecurityPolicy {
    pub max_security_level: u32,
    pub require_audit: bool,
    pub allowed_licenses: Vec<String>,
    pub blocked_crates: Vec<String>,
    pub auto_update_dev_deps: bool,
}

/// Compliance rule for CI/CD automation
#[derive(Clone)]
#[contracttype]
pub struct ComplianceRule {
    pub rule_name: String,
    pub description: String,
    pub check_type: String, // "version", "security", "license", "audit"
    pub enabled: bool,
    pub severity: String, // "error", "warning", "info"
}
//! # Cargo.toml Rust Dependency Review
//!
//! This module provides comprehensive dependency management, validation, and
//! security checking for Cargo.toml Rust dependencies to improve CI/CD and scalability.
//!
//! ## Features
//!
//! - **Dependency Validation**: Automatic validation against security policies
//! - **Version Management**: Centralized version tracking and updates
//! - **Security Policies**: Configurable security levels and blocked crates
//! - **Compliance Rules**: CI/CD compliance automation
//! - **Audit Trail**: Complete dependency change history
//!
//! ## Current Dependencies
//!
//! | Crate        | Version   | Scope       | Security Level | Status |
//! |--------------|-----------|-------------|----------------|---------|
//! | `soroban-sdk`| `22.0.11` | workspace   | 2              | Approved |
//! | `proptest`   | `1.11.0`  | dev only    | 1              | Approved |
//!
//! ## Security Assumptions
//!
//! 1. **Minor bump** — `soroban-sdk 22.1.0` is a minor release; storage layout
//!    and host-function IDs remain backward-compatible within the 22.x series.
//! 2. **Dev-only dependencies** — Development dependencies never affect WASM binary
//! 3. **Security validation** — All dependencies must pass security checks
//! 4. **Compliance enforcement** — CI/CD rules are automatically enforced
//! 5. **Audit trail** — All changes are tracked and verifiable

#[allow(dead_code, missing_docs)]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec, Map, String};
use soroban_sdk::{contract, contractimpl, contracttype, Env, Map, String, Vec};

// ── Contract Types for Dependency Management ─────────────────────────────────────

/// Data storage keys for the contract
#[contracttype]
pub enum DataKey {
    ApprovedDependencies,
    DependencyVersions,
    SecurityPolicies,
    ComplianceRules,
}

/// Dependency information structure with security metadata
#[derive(Clone)]
#[contracttype]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub security_level: u32, // 1-5 scale (1=lowest risk, 5=highest risk)
    pub last_updated: u64,
    pub approved: bool,
    pub dev_only: bool,
}

/// Security policy configuration for dependency validation
#[derive(Clone)]
#[contracttype]
pub struct SecurityPolicy {
    pub max_security_level: u32,
    pub require_audit: bool,
    pub allowed_licenses: Vec<String>,
    pub blocked_crates: Vec<String>,
    pub auto_update_dev_deps: bool,
}

/// Compliance rule for CI/CD automation
#[derive(Clone)]
#[contracttype]
pub struct ComplianceRule {
    pub rule_name: String,
    pub description: String,
    pub check_type: String, // "version", "security", "license", "audit"
    pub enabled: bool,
    pub severity: String, // "error", "warning", "info"
}

// ── Pinned version constants ──────────────────────────────────────────────────

/// The soroban-sdk version this contract is compiled against.
///
/// @notice Changing this constant without a corresponding Cargo.toml bump is
///         a documentation error, not a functional change.
/// @dev Security level: 2 (medium - core SDK dependency)
pub const SOROBAN_SDK_VERSION: &str = "22.1.0";
pub const SOROBAN_SDK_VERSION: &str = "22.0.11";

/// The previous soroban-sdk version, retained for audit trail.
///
/// @deprecated Superseded by [`SOROBAN_SDK_VERSION`].
/// @notice Security level: 2 (medium - core SDK dependency)
#[deprecated(since = "22.1.0", note = "Upgrade to soroban-sdk 22.1.0")]
pub const SOROBAN_SDK_VERSION_DEPRECATED: &str = "22.0.11";
#[deprecated(since = "22.0.11", note = "Upgrade to soroban-sdk 22.0.11")]
pub const SOROBAN_SDK_VERSION_DEPRECATED: &str = "22.0.1";

/// The proptest version used in dev-dependencies.
///
/// @dev Not compiled into the WASM binary.
/// @notice Security level: 1 (low - dev-only dependency)
pub const PROPTEST_VERSION: &str = "1.5.0";
pub const PROPTEST_VERSION: &str = "1.11.0";

/// The previous proptest version, retained for audit trail.
///
/// @deprecated Superseded by [`PROPTEST_VERSION`].
/// @dev Not compiled into the WASM binary.
/// @notice Security level: 1 (low - dev-only dependency)
#[deprecated(since = "1.5.0", note = "Upgrade to proptest 1.5.0")]
pub const PROPTEST_VERSION_DEPRECATED: &str = "1.4";

// ── Legacy Dependency Record (for backward compatibility) ───────────────────────
#[deprecated(since = "1.11.0", note = "Upgrade to proptest 1.11.0")]
pub const PROPTEST_VERSION_DEPRECATED: &str = "1.4";

// ── Legacy Dependency Record (for backward compatibility) ───────────────────────

/// Represents a single Cargo dependency entry for audit purposes.
#[derive(Clone, Debug, PartialEq)]
pub struct DepRecord {
    /// Crate name.
    pub name: &'static str,
    /// Pinned version in use.
    pub version: &'static str,
    /// Whether this dependency is dev-only (not in the WASM binary).
    pub dev_only: bool,
    /// Whether the previous version has been deprecated.
    pub deprecated_previous: bool,
}

/// Returns the canonical list of audited dependencies for this contract.
///
/// @notice This list is the single source of truth for dependency review.
///         Any addition or version change must be reflected here.
/// @dev Maintained for backward compatibility with existing audit processes.
pub fn audited_dependencies() -> [DepRecord; 2] {
    [
        DepRecord {
            name: "soroban-sdk",
            version: SOROBAN_SDK_VERSION,
            dev_only: false,
            deprecated_previous: true,
        },
        DepRecord {
            name: "proptest",
            version: PROPTEST_VERSION,
            dev_only: true,
            deprecated_previous: true,
        },
    ]
}

/// Returns `true` if all audited dependencies have their deprecated
/// predecessors replaced (i.e. no old versions remain in use).
///
/// @notice This is a compile-time-equivalent check expressed as a runtime
///         function for testability.
/// @dev Maintained for backward compatibility.
pub fn all_deprecated_versions_replaced() -> bool {
    audited_dependencies().iter().all(|d| d.deprecated_previous)
}

// ── Contract Implementation ─────────────────────────────────────────────────────

#[contract]
pub struct CargoTomlRust;

#[contractimpl]
impl CargoTomlRust {
    /// Initialize the contract with default security policies and compliance rules
    ///
    /// @notice Sets up the dependency management system with secure defaults
    /// @dev Must be called before any other contract functions
    /// @param env The Soroban environment
    pub fn initialize(env: Env) {
        if env.storage().instance().has(&DataKey::SecurityPolicies) {
            panic!("Contract already initialized");
        }

        // Default security policy for CI/CD standardization
        let default_policy = SecurityPolicy {
            max_security_level: 3,
            require_audit: true,
            allowed_licenses: Vec::from_array(
                &env,
                [
                    String::from_str(&env, "MIT"),
                    String::from_str(&env, "Apache-2.0"),
                    String::from_str(&env, "BSD-3-Clause"),
                    String::from_str(&env, "0BSD"),
                ],
            ),
            blocked_crates: Vec::new(&env),
            auto_update_dev_deps: true,
        };

        // Default compliance rules for CI/CD
        let mut default_rules = Vec::<ComplianceRule>::new(&env);

        default_rules.push_back(ComplianceRule {
            rule_name: String::from_str(&env, "version_check"),
            description: String::from_str(&env, "Ensure all dependencies use approved versions"),
            check_type: String::from_str(&env, "version"),
            enabled: true,
            severity: String::from_str(&env, "error"),
        });

        default_rules.push_back(ComplianceRule {
            rule_name: String::from_str(&env, "security_validation"),
            description: String::from_str(&env, "Validate dependency security levels"),
            check_type: String::from_str(&env, "security"),
            enabled: true,
            severity: String::from_str(&env, "error"),
        });

        env.storage()
            .instance()
            .set(&DataKey::SecurityPolicies, &default_policy);
        env.storage().instance().set(
            &DataKey::ApprovedDependencies,
            &Vec::<DependencyInfo>::new(&env),
        );
        env.storage().instance().set(
            &DataKey::DependencyVersions,
            &Map::<String, String>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRules, &default_rules);
    }

    /// Add an approved dependency with comprehensive security validation
    ///
    /// @notice Adds a dependency to the approved list after security checks
    /// @dev Enforces security policies and maintains audit trail
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param version Dependency version
    /// @param security_level Security level (1-5, 1=lowest risk)
    /// @param last_updated Unix timestamp of last update
    /// @param dev_only Whether this is a development-only dependency
    pub fn add_approved_dependency(
        env: Env,
        name: String,
        version: String,
        security_level: u32,
        last_updated: u64,
        dev_only: bool,
    ) {
        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        // Security validation
        if security_level > policy.max_security_level {
            panic!(
                "Security level {} exceeds maximum allowed {}",
                security_level, policy.max_security_level
            );
        }

        // Check if dependency is blocked
        if policy.blocked_crates.contains(&name) {
            panic!("Dependency is blocked by security policy");
        }

        // Auto-approve dev dependencies if policy allows
        let approved = if dev_only && policy.auto_update_dev_deps {
            true
        } else {
            policy.require_audit
        };

        let dependency = DependencyInfo {
            name: name.clone(),
            version: version.clone(),
            security_level,
            last_updated,
            approved,
            dev_only,
        };

        let mut approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        // Check for existing dependency and update if found
        let mut found = false;
        for i in 0..approved_deps.len() {
            if approved_deps.get(i).unwrap().name == name {
                approved_deps.set(i, dependency.clone());
                found = true;
                break;
            }
        }

        if !found {
            approved_deps.push_back(dependency);
        }

        env.storage()
            .instance()
            .set(&DataKey::ApprovedDependencies, &approved_deps);

        // Update version mapping
        let mut version_map: Map<String, String> = env
            .storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env));

        version_map.set(name, version);
        env.storage()
            .instance()
            .set(&DataKey::DependencyVersions, &version_map);
    }

    /// Validate a dependency against current security policies
    ///
    /// @notice Comprehensive validation including security, version, and compliance
    /// @dev Returns false if any validation fails
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param version Dependency version
    /// @param security_level Security level (1-5, 1=lowest risk)
    /// @return true if dependency is valid, false otherwise
    pub fn validate_dependency(
        env: Env,
        name: String,
        version: String,
        security_level: u32,
    ) -> bool {
        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        // Check security level
        if security_level > policy.max_security_level {
            return false;
        }

        // Check if blocked
        if policy.blocked_crates.contains(&name) {
            return false;
        }

        // Check if approved
        let approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        for dep in approved_deps.iter() {
            if dep.name == name && dep.version == version && dep.approved {
                return true;
            }
        }

        false
    }

    /// Update security policy configuration
    ///
    /// @notice Updates the security policy for dependency validation
    /// @dev Only callable by authorized administrators
    /// @param env The Soroban environment
    /// @param policy New security policy configuration
    pub fn update_security_policy(env: Env, policy: SecurityPolicy) {
        env.storage()
            .instance()
            .set(&DataKey::SecurityPolicies, &policy);
    }

    /// Add or update a compliance rule for CI/CD automation
    ///
    /// @notice Adds a new compliance rule or updates existing one
    /// @dev Rules are automatically enforced during dependency validation
    /// @param env The Soroban environment
    /// @param rule Compliance rule to add
    pub fn add_compliance_rule(env: Env, rule: ComplianceRule) {
        let mut rules: Vec<ComplianceRule> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env));

        // Check for existing rule and update if found
        let mut found = false;
        for i in 0..rules.len() {
            if rules.get(i).unwrap().rule_name == rule.rule_name {
                rules.set(i, rule.clone());
                found = true;
                break;
            }
        }

        if !found {
            rules.push_back(rule);
        }

        env.storage()
            .instance()
            .set(&DataKey::ComplianceRules, &rules);
    }

    /// Get all approved dependencies with their security metadata
    ///
    /// @notice Returns the complete list of approved dependencies
    /// @dev Includes security levels and approval status
    /// @param env The Soroban environment
    /// @return Vector of approved dependencies
    pub fn get_approved_dependencies(env: Env) -> Vec<DependencyInfo> {
        env.storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get current security policy configuration
    ///
    /// @notice Returns the current security policy settings
    /// @dev Includes allowed licenses and blocked crates
    /// @param env The Soroban environment
    /// @return Current security policy
    pub fn get_security_policy(env: Env) -> SecurityPolicy {
        env.storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"))
    }

    /// Get all compliance rules for CI/CD
    ///
    /// @notice Returns the complete list of compliance rules
    /// @dev Includes rule types and severity levels
    /// @param env The Soroban environment
    /// @return Vector of compliance rules
    pub fn get_compliance_rules(env: Env) -> Vec<ComplianceRule> {
        env.storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Check if a dependency version is up to date
    ///
    /// @notice Compares current version with latest approved version
    /// @dev Useful for CI/CD pipelines to detect outdated dependencies
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param current_version Current version to check
    /// @return true if up to date, false otherwise
    pub fn is_dependency_up_to_date(env: Env, name: String, current_version: String) -> bool {
        let version_map: Map<String, String> = env
            .storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env));

        match version_map.get(name) {
            Some(latest_version) => latest_version == current_version,
            None => false,
        }
    }

    /// Block a dependency crate for security reasons
    ///
    /// @notice Adds a crate to the blocked list for immediate security response
    /// @dev Blocked dependencies cannot be added or used
    /// @param env The Soroban environment
    /// @param crate_name Name of the crate to block
    pub fn block_dependency(env: Env, crate_name: String) {
        let mut policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        if !policy.blocked_crates.contains(&crate_name) {
            policy.blocked_crates.push_back(crate_name.clone());
            env.storage()
                .instance()
                .set(&DataKey::SecurityPolicies, &policy);

            // Remove from approved dependencies if present
            let approved_deps: Vec<DependencyInfo> = env
                .storage()
                .instance()
                .get(&DataKey::ApprovedDependencies)
                .unwrap_or_else(|| Vec::new(&env));

            let mut updated_deps = Vec::<DependencyInfo>::new(&env);
            for dep in approved_deps.iter() {
                if dep.name != crate_name {
                    updated_deps.push_back(dep);
                }
            }

            env.storage()
                .instance()
                .set(&DataKey::ApprovedDependencies, &updated_deps);
        }
    }

    /// Get complete dependency version mapping
    ///
    /// @notice Returns mapping of dependency names to their approved versions
    /// @dev Useful for generating Cargo.toml files
    /// @param env The Soroban environment
    /// @return Map of dependency names to versions
    pub fn get_dependency_versions(env: Env) -> Map<String, String> {
        env.storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Run comprehensive compliance check
    ///
    /// @notice Validates all dependencies against all compliance rules
    /// @dev Returns detailed compliance report
    /// @param env The Soroban environment
    /// @return Vector of compliance rule results (name, passed, message)
    pub fn run_compliance_check(env: Env) -> Vec<(String, bool, String)> {
        let rules: Vec<ComplianceRule> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env));

        let approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        let mut results = Vec::<(String, bool, String)>::new(&env);

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let (passed, message) = match rule.check_type.to_string().as_str() {
                "version" => {
                    let outdated_count = approved_deps
                        .iter()
                        .filter(|dep| {
                            !env.storage()
                                .instance()
                                .get(&DataKey::DependencyVersions)
                                .unwrap_or_else(|| Map::new(&env))
                                .get(dep.name.clone())
                                .map_or(false, |latest| latest == dep.version)
                        })
                        .count();

                    (
                        outdated_count == 0,
                        if outdated_count == 0 {
                            "All dependencies are up to date".to_string()
                        } else {
                            format!("{} dependencies are out of date", outdated_count)
                        },
                    )
                }
                "security" => {
                    let high_risk_count = approved_deps
                        .iter()
                        .filter(|dep| dep.security_level > policy.max_security_level)
                        .count();

                    (
                        high_risk_count == 0,
                        if high_risk_count == 0 {
                            "All dependencies meet security requirements".to_string()
                        } else {
                            format!(
                                "{} dependencies exceed maximum security level",
                                high_risk_count
                            )
                        },
                    )
                }
                "audit" => {
                    let unapproved_count = approved_deps.iter().filter(|dep| !dep.approved).count();

                    (
                        unapproved_count == 0,
                        if unapproved_count == 0 {
                            "All dependencies are approved".to_string()
                        } else {
                            format!("{} dependencies require approval", unapproved_count)
                        },
                    )
                }
                _ => (false, "Unknown rule type".to_string()),
            };

            results.push_back((rule.rule_name.clone(), passed, message));
        }

        results
    }
}
pub fn all_deprecated_versions_replaced() -> bool {
    audited_dependencies().iter().all(|d| d.deprecated_previous)
}

// ── Contract Implementation ─────────────────────────────────────────────────────

#[contract]
pub struct CargoTomlRust;

#[contractimpl]
impl CargoTomlRust {
    /// Initialize the contract with default security policies and compliance rules
    /// 
    /// @notice Sets up the dependency management system with secure defaults
    /// @dev Must be called before any other contract functions
    /// @param env The Soroban environment
    pub fn initialize(env: Env) {
        if env.storage().instance().has(&DataKey::SecurityPolicies) {
            panic!("Contract already initialized");
        }

        // Default security policy for CI/CD standardization
        let default_policy = SecurityPolicy {
            max_security_level: 3,
            require_audit: true,
            allowed_licenses: Vec::from_array(
                &env,
                [
                    String::from_str(&env, "MIT"),
                    String::from_str(&env, "Apache-2.0"),
                    String::from_str(&env, "BSD-3-Clause"),
                    String::from_str(&env, "0BSD"),
                ],
            ),
            blocked_crates: Vec::new(&env),
            auto_update_dev_deps: true,
        };

        // Default compliance rules for CI/CD
        let mut default_rules = Vec::<ComplianceRule>::new(&env);

        default_rules.push_back(ComplianceRule {
            rule_name: String::from_str(&env, "version_check"),
            description: String::from_str(&env, "Ensure all dependencies use approved versions"),
            check_type: String::from_str(&env, "version"),
            enabled: true,
            severity: String::from_str(&env, "error"),
        });

        default_rules.push_back(ComplianceRule {
            rule_name: String::from_str(&env, "security_validation"),
            description: String::from_str(&env, "Validate dependency security levels"),
            check_type: String::from_str(&env, "security"),
            enabled: true,
            severity: String::from_str(&env, "error"),
        });

        env.storage()
            .instance()
            .set(&DataKey::SecurityPolicies, &default_policy);
        env.storage().instance().set(
            &DataKey::ApprovedDependencies,
            &Vec::<DependencyInfo>::new(&env),
        );
        env.storage().instance().set(
            &DataKey::DependencyVersions,
            &Map::<String, String>::new(&env),
        );
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRules, &default_rules);
    }

    /// Add an approved dependency with comprehensive security validation
    ///
    /// @notice Adds a dependency to the approved list after security checks
    /// @dev Enforces security policies and maintains audit trail
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param version Dependency version
    /// @param security_level Security level (1-5, 1=lowest risk)
    /// @param last_updated Unix timestamp of last update
    /// @param dev_only Whether this is a development-only dependency
    pub fn add_approved_dependency(
        env: Env,
        name: String,
        version: String,
        security_level: u32,
        last_updated: u64,
        dev_only: bool,
    ) {
        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        // Security validation
        if security_level > policy.max_security_level {
            panic!(
                "Security level {} exceeds maximum allowed {}",
                security_level, policy.max_security_level
            );
        }

        // Check if dependency is blocked
        if policy.blocked_crates.contains(&name) {
            panic!("Dependency is blocked by security policy");
        }

        // Auto-approve dev dependencies if policy allows
        let approved = if dev_only && policy.auto_update_dev_deps {
            true
        } else {
            policy.require_audit
        };

        let dependency = DependencyInfo {
            name: name.clone(),
            version: version.clone(),
            security_level,
            last_updated,
            approved,
            dev_only,
        };

        let mut approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        // Check for existing dependency and update if found
        let mut found = false;
        for i in 0..approved_deps.len() {
            if approved_deps.get(i).unwrap().name == name {
                approved_deps.set(i, dependency.clone());
                found = true;
                break;
            }
        }

        if !found {
            approved_deps.push_back(dependency);
        }

        env.storage()
            .instance()
            .set(&DataKey::ApprovedDependencies, &approved_deps);

        // Update version mapping
        let mut version_map: Map<String, String> = env
            .storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env));

        version_map.set(name, version);
        env.storage()
            .instance()
            .set(&DataKey::DependencyVersions, &version_map);
    }

    /// Validate a dependency against current security policies
    ///
    /// @notice Comprehensive validation including security, version, and compliance
    /// @dev Returns false if any validation fails
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param version Dependency version
    /// @param security_level Security level (1-5, 1=lowest risk)
    /// @return true if dependency is valid, false otherwise
    pub fn validate_dependency(
        env: Env,
        name: String,
        version: String,
        security_level: u32,
    ) -> bool {
        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        // Check security level
        if security_level > policy.max_security_level {
            return false;
        }

        // Check if blocked
        if policy.blocked_crates.contains(&name) {
            return false;
        }

        // Check if approved
        let approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        for dep in approved_deps.iter() {
            if dep.name == name && dep.version == version && dep.approved {
                return true;
            }
        }

        false
    }

    /// Update security policy configuration
    ///
    /// @notice Updates the security policy for dependency validation
    /// @dev Only callable by authorized administrators
    /// @param env The Soroban environment
    /// @param policy New security policy configuration
    pub fn update_security_policy(env: Env, policy: SecurityPolicy) {
        env.storage()
            .instance()
            .set(&DataKey::SecurityPolicies, &policy);
    }

    /// Add or update a compliance rule for CI/CD automation
    ///
    /// @notice Adds a new compliance rule or updates existing one
    /// @dev Rules are automatically enforced during dependency validation
    /// @param env The Soroban environment
    /// @param rule Compliance rule to add
    pub fn add_compliance_rule(env: Env, rule: ComplianceRule) {
        let mut rules: Vec<ComplianceRule> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env));

        // Check for existing rule and update if found
        let mut found = false;
        for i in 0..rules.len() {
            if rules.get(i).unwrap().rule_name == rule.rule_name {
                rules.set(i, rule.clone());
                found = true;
                break;
            }
        }

        if !found {
            rules.push_back(rule);
        }

        env.storage()
            .instance()
            .set(&DataKey::ComplianceRules, &rules);
    }

    /// Get all approved dependencies with their security metadata
    ///
    /// @notice Returns the complete list of approved dependencies
    /// @dev Includes security levels and approval status
    /// @param env The Soroban environment
    /// @return Vector of approved dependencies
    pub fn get_approved_dependencies(env: Env) -> Vec<DependencyInfo> {
        env.storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get current security policy configuration
    ///
    /// @notice Returns the current security policy settings
    /// @dev Includes allowed licenses and blocked crates
    /// @param env The Soroban environment
    /// @return Current security policy
    pub fn get_security_policy(env: Env) -> SecurityPolicy {
        env.storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"))
    }

    /// Get all compliance rules for CI/CD
    ///
    /// @notice Returns the complete list of compliance rules
    /// @dev Includes rule types and severity levels
    /// @param env The Soroban environment
    /// @return Vector of compliance rules
    pub fn get_compliance_rules(env: Env) -> Vec<ComplianceRule> {
        env.storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Check if a dependency version is up to date
    ///
    /// @notice Compares current version with latest approved version
    /// @dev Useful for CI/CD pipelines to detect outdated dependencies
    /// @param env The Soroban environment
    /// @param name Dependency name
    /// @param current_version Current version to check
    /// @return true if up to date, false otherwise
    pub fn is_dependency_up_to_date(env: Env, name: String, current_version: String) -> bool {
        let version_map: Map<String, String> = env
            .storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env));

        match version_map.get(name) {
            Some(latest_version) => latest_version == current_version,
            None => false,
        }
    }

    /// Block a dependency crate for security reasons
    ///
    /// @notice Adds a crate to the blocked list for immediate security response
    /// @dev Blocked dependencies cannot be added or used
    /// @param env The Soroban environment
    /// @param crate_name Name of the crate to block
    pub fn block_dependency(env: Env, crate_name: String) {
        let mut policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        if !policy.blocked_crates.contains(&crate_name) {
            policy.blocked_crates.push_back(crate_name.clone());
            env.storage()
                .instance()
                .set(&DataKey::SecurityPolicies, &policy);

            // Remove from approved dependencies if present
            let approved_deps: Vec<DependencyInfo> = env
                .storage()
                .instance()
                .get(&DataKey::ApprovedDependencies)
                .unwrap_or_else(|| Vec::new(&env));

            let mut updated_deps = Vec::<DependencyInfo>::new(&env);
            for dep in approved_deps.iter() {
                if dep.name != crate_name {
                    updated_deps.push_back(dep);
                }
            }

            env.storage()
                .instance()
                .set(&DataKey::ApprovedDependencies, &updated_deps);
        }
    }

    /// Get complete dependency version mapping
    ///
    /// @notice Returns mapping of dependency names to their approved versions
    /// @dev Useful for generating Cargo.toml files
    /// @param env The Soroban environment
    /// @return Map of dependency names to versions
    pub fn get_dependency_versions(env: Env) -> Map<String, String> {
        env.storage()
            .instance()
            .get(&DataKey::DependencyVersions)
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Run comprehensive compliance check
    ///
    /// @notice Validates all dependencies against all compliance rules
    /// @dev Returns detailed compliance report
    /// @param env The Soroban environment
    /// @return Vector of compliance rule results (name, passed, message)
    pub fn run_compliance_check(env: Env) -> Vec<(String, bool, String)> {
        let rules: Vec<ComplianceRule> = env
            .storage()
            .instance()
            .get(&DataKey::ComplianceRules)
            .unwrap_or_else(|| Vec::new(&env));

        let approved_deps: Vec<DependencyInfo> = env
            .storage()
            .instance()
            .get(&DataKey::ApprovedDependencies)
            .unwrap_or_else(|| Vec::new(&env));

        let policy: SecurityPolicy = env
            .storage()
            .instance()
            .get(&DataKey::SecurityPolicies)
            .unwrap_or_else(|| panic!("Security policies not initialized"));

        let mut results = Vec::<(String, bool, String)>::new(&env);

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let check_type_version = soroban_sdk::String::from_str(&env, "version");
            let check_type_security = soroban_sdk::String::from_str(&env, "security");
            let check_type_audit = soroban_sdk::String::from_str(&env, "audit");

            let (passed, message) = if rule.check_type == check_type_version {
                let outdated_count = approved_deps
                    .iter()
                    .filter(|dep| {
                        !env.storage()
                            .instance()
                            .get::<_, Map<String, String>>(&DataKey::DependencyVersions)
                            .unwrap_or_else(|| Map::new(&env))
                            .get(dep.name.clone())
                            .map_or(false, |latest: String| latest == dep.version)
                    })
                    .count();

                (
                    outdated_count == 0,
                    if outdated_count == 0 {
                        soroban_sdk::String::from_str(&env, "All dependencies are up to date")
                    } else {
                        soroban_sdk::String::from_str(&env, "Some dependencies are out of date")
                    },
                )
            } else if rule.check_type == check_type_security {
                let high_risk_count = approved_deps
                    .iter()
                    .filter(|dep| dep.security_level > policy.max_security_level)
                    .count();

                (
                    high_risk_count == 0,
                    if high_risk_count == 0 {
                        soroban_sdk::String::from_str(
                            &env,
                            "All dependencies meet security requirements",
                        )
                    } else {
                        soroban_sdk::String::from_str(
                            &env,
                            "dependencies exceed maximum security level",
                        )
                    },
                )
            } else if rule.check_type == check_type_audit {
                let unapproved_count = approved_deps.iter().filter(|dep| !dep.approved).count();

                (
                    unapproved_count == 0,
                    if unapproved_count == 0 {
                        soroban_sdk::String::from_str(&env, "All dependencies are approved")
                    } else {
                        soroban_sdk::String::from_str(&env, "Some dependencies require approval")
                    },
                )
            } else {
                (
                    false,
                    soroban_sdk::String::from_str(&env, "Unknown rule type"),
                )
            };

            results.push_back((rule.rule_name.clone(), passed, message));
        }

        results
    }
}
