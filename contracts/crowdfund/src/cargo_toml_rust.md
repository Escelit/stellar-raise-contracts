# Cargo.toml Rust Dependency Management for CI/CD Standardization

## Overview

This contract provides comprehensive dependency management, validation, and security checking for Cargo.toml Rust dependencies to improve CI/CD and scalability. It implements a standardized approach to dependency governance with automated compliance enforcement.

## Features

- **Dependency Validation**: Automatic validation against security policies
- **Version Management**: Centralized version tracking and updates
- **Security Policies**: Configurable security levels and blocked crates
- **Compliance Rules**: CI/CD compliance automation
- **Audit Trail**: Complete dependency change history
- **Security Enforcement**: Real-time blocking of vulnerable dependencies

## Architecture

### Core Components

1. **DependencyInfo**: Stores metadata for each dependency including security level and approval status
2. **SecurityPolicy**: Defines security constraints and validation rules
3. **ComplianceRule**: Configurable CI/CD compliance checks
4. **CargoTomlRust**: Main contract implementing dependency management logic

### Data Storage

- `ApprovedDependencies`: List of approved dependencies with metadata
- `DependencyVersions`: Mapping of dependency names to approved versions
- `SecurityPolicies`: Current security policy configuration
- `ComplianceRules`: Active compliance rules for CI/CD

## Security Model

### Security Levels

Dependencies are categorized by security risk (1-5 scale):
- **Level 1**: Low risk (dev-only dependencies, well-audited libraries)
- **Level 2**: Medium risk (core SDK dependencies, widely used libraries)
- **Level 3**: Medium-high risk (network libraries, crypto utilities)
- **Level 4**: High risk (system-level libraries, experimental features)
- **Level 5**: Critical risk (unaudited code, recent vulnerabilities)

### Security Assumptions

1. **Patch-only bump** — All version changes follow semantic versioning
2. **Dev-only dependencies** — Development dependencies never affect WASM binary
3. **Security validation** — All dependencies must pass security checks
4. **Compliance enforcement** — CI/CD rules are automatically enforced
5. **Audit trail** — All changes are tracked and verifiable

### Threat Mitigation

- **Supply Chain Attacks**: Blocked crate list prevents known malicious dependencies
- **Version Conflicts**: Centralized version management eliminates conflicts
- **Compliance Violations**: Automated enforcement prevents policy breaches
- **Audit Gaps**: Complete history tracking ensures transparency

## API Reference

### Contract Functions

#### `initialize(env: Env)`
**@notice** Sets up the dependency management system with secure defaults
**@dev** Must be called before any other contract functions
**@param env** The Soroban environment

#### `add_approved_dependency(env, name, version, security_level, last_updated, dev_only)`
**@notice** Adds a dependency to the approved list after security checks
**@dev** Enforces security policies and maintains audit trail
**@param name** Dependency name
**@param version** Dependency version
**@param security_level** Security level (1-5, 1=lowest risk)
**@param last_updated** Unix timestamp of last update
**@param dev_only** Whether this is a development-only dependency

#### `validate_dependency(env, name, version, security_level) -> bool`
**@notice** Comprehensive validation including security, version, and compliance
**@dev** Returns false if any validation fails
**@param name** Dependency name
**@param version** Dependency version
**@param security_level** Security level (1-5, 1=lowest risk)
**@return** true if dependency is valid, false otherwise

#### `update_security_policy(env, policy)`
**@notice** Updates the security policy for dependency validation
**@dev** Only callable by authorized administrators
**@param policy** New security policy configuration

#### `add_compliance_rule(env, rule)`
**@notice** Adds a new compliance rule or updates existing one
**@dev** Rules are automatically enforced during dependency validation
**@param rule** Compliance rule to add

#### `block_dependency(env, crate_name)`
**@notice** Adds a crate to the blocked list for immediate security response
**@dev** Blocked dependencies cannot be added or used
**@param crate_name** Name of the crate to block

#### `run_compliance_check(env) -> Vec<(String, bool, String)>`
**@notice** Validates all dependencies against all compliance rules
**@dev** Returns detailed compliance report
**@return** Vector of compliance rule results (name, passed, message)

### Query Functions

#### `get_approved_dependencies(env) -> Vec<DependencyInfo>`
**@notice** Returns the complete list of approved dependencies
**@dev** Includes security levels and approval status
**@return** Vector of approved dependencies

#### `get_security_policy(env) -> SecurityPolicy`
**@notice** Returns the current security policy settings
**@dev** Includes allowed licenses and blocked crates
**@return** Current security policy

#### `get_compliance_rules(env) -> Vec<ComplianceRule>`
**@notice** Returns the complete list of compliance rules
**@dev** Includes rule types and severity levels
**@return** Vector of compliance rules

#### `is_dependency_up_to_date(env, name, current_version) -> bool`
**@notice** Compares current version with latest approved version
**@dev** Useful for CI/CD pipelines to detect outdated dependencies
**@param name** Dependency name
**@param current_version** Current version to check
**@return** true if up to date, false otherwise

#### `get_dependency_versions(env) -> Map<String, String>`
**@notice** Returns mapping of dependency names to their approved versions
**@dev** Useful for generating Cargo.toml files
**@return** Map of dependency names to versions

## Data Structures

### DependencyInfo
```rust
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub security_level: u32, // 1-5 scale (1=lowest risk, 5=highest risk)
    pub last_updated: u64,
    pub approved: bool,
    pub dev_only: bool,
}
```

### SecurityPolicy
```rust
pub struct SecurityPolicy {
    pub max_security_level: u32,
    pub require_audit: bool,
    pub allowed_licenses: Vec<String>,
    pub blocked_crates: Vec<String>,
    pub auto_update_dev_deps: bool,
}
```

### ComplianceRule
```rust
pub struct ComplianceRule {
    pub rule_name: String,
    pub description: String,
    pub check_type: String, // "version", "security", "license", "audit"
    pub enabled: bool,
    pub severity: String,   // "error", "warning", "info"
}
```

## Usage Examples

### Basic Setup
```rust
// Initialize the contract
CargoTomlRust::initialize(env);

// Add approved dependencies
CargoTomlRust::add_approved_dependency(
    env,
    String::from_str(&env, "soroban-sdk"),
    String::from_str(&env, "22.1.0"),
    2, // security level
    1640995200, // timestamp
    false, // not dev-only
);
```

### Security Policy Configuration
```rust
let strict_policy = SecurityPolicy {
    max_security_level: 2,
    require_audit: true,
    allowed_licenses: Vec::from_array(&env, [
        String::from_str(&env, "MIT"),
        String::from_str(&env, "Apache-2.0"),
    ]),
    blocked_crates: Vec::new(&env),
    auto_update_dev_deps: false,
};

CargoTomlRust::update_security_policy(env, strict_policy);
```

### Compliance Checking
```rust
// Run comprehensive compliance check
let results = CargoTomlRust::run_compliance_check(env);

for (rule_name, passed, message) in results.iter() {
    if !passed {
        println!("Rule {} failed: {}", rule_name, message);
    }
}
```

### Security Response
```rust
// Immediately block a vulnerable dependency
CargoTomlRust::block_dependency(env, String::from_str(&env, "vulnerable-crate"));

// Verify it's blocked
assert!(!CargoTomlRust::validate_dependency(
    env,
    String::from_str(&env, "vulnerable-crate"),
    String::from_str(&env, "1.0.0"),
    1
));
```

## Integration with CI/CD

### GitHub Actions Example
```yaml
- name: Check Dependencies
  run: |
    # Run compliance check
    cargo run --bin cargo_toml_rust -- check-compliance
    
    # Fail build if compliance fails
    if [ $? -ne 0 ]; then
      echo "Dependency compliance check failed"
      exit 1
    fi
```

### Dependency Update Pipeline
```rust
// Check for outdated dependencies
let versions = CargoTomlRust::get_dependency_versions(env);

for (name, approved_version) in versions.iter() {
    let current_version = get_current_version(&name);
    
    if !CargoTomlRust::is_dependency_up_to_date(env, name.clone(), current_version) {
        println!("Dependency {} is outdated: {} -> {}", name, current_version, approved_version);
    }
}
```

## Security Considerations

### Access Control
- Contract initialization is protected against double-initialization
- Security policy updates should be restricted to authorized addresses
- Dependency blocking provides immediate security response capability

### Input Validation
- All dependency names and versions are validated
- Security levels are enforced to be within 1-5 range
- Timestamps are validated for reasonableness

### Audit Trail
- All dependency changes are permanently recorded
- Version history is maintained for compliance audits
- Security policy changes are tracked with timestamps

## Testing

The contract includes comprehensive tests covering:
- Contract initialization and configuration
- Dependency addition and validation
- Security policy enforcement
- Compliance rule management
- Edge cases and error conditions
- Integration scenarios

Run tests with:
```bash
cargo test cargo_toml_rust
```

## Migration Guide

### From Manual Dependency Management
1. Initialize the contract with current dependencies
2. Set appropriate security policies
3. Configure compliance rules
4. Migrate existing dependency approvals
5. Update CI/CD pipelines to use contract validation

### Upgrading Security Policies
1. Review current security levels
2. Update policy using `update_security_policy`
3. Run compliance check to identify affected dependencies
4. Update or remove non-compliant dependencies
5. Verify all compliance checks pass

## Best Practices

### Security Levels
- Use level 1 for well-audited, dev-only dependencies
- Use level 2-3 for production dependencies with good security track records
- Use level 4+ only for necessary dependencies with thorough review
- Block dependencies with known vulnerabilities immediately

### Compliance Rules
- Enable version checking for production deployments
- Use security validation for all environments
- Configure license checking for legal compliance
- Set appropriate severity levels for your organization

### Monitoring
- Regularly run compliance checks
- Monitor for new dependency vulnerabilities
- Review security policy effectiveness
- Maintain up-to-date dependency versions

## Troubleshooting

### Common Issues

**"Security level exceeds maximum allowed"**
- Check the current security policy
- Verify the dependency's security level assessment
- Consider updating the policy or finding alternatives

**"Dependency is blocked by security policy"**
- Check the blocked crates list in the security policy
- Verify if the block is intentional
- Consider unblocking if the dependency is now safe

**"Contract already initialized"**
- The contract has already been set up
- Use existing contract instance
- Deploy new contract instance for fresh start

### Debug Information
Use the query functions to inspect contract state:
- `get_security_policy()` to check current security settings
- `get_approved_dependencies()` to see approved dependencies
- `get_compliance_rules()` to review active rules
- `run_compliance_check()` to identify compliance issues

## License

This contract is part of the stellar-raise-contracts project and follows the same license terms.

## Contributing

When contributing to this contract:
1. Add comprehensive tests for new features
2. Update documentation with NatSpec comments
3. Follow security best practices
4. Ensure backward compatibility when possible
5. Update this README with any API changes
