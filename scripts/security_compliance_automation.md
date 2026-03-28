# Security Compliance Automation for CI/CD

## Overview

`security_compliance_automation.sh` provides automated security compliance checks designed for CI/CD pipelines. This shell script validates security patterns, code quality, and compliance requirements for Stellar/Soroban smart contract projects.

## Security Assumptions

1. **Read-only** — No function writes to storage or state files. All checks are safe to run in any context.
2. **Permissionless** — No privileged access required. Automated tooling can run checks without special permissions.
3. **Deterministic** — Same input produces the same output. Results are reproducible.
4. **Bounded execution** — No unbounded loops or iterations. Checks complete in predictable time.
5. **Safe arithmetic** — All operations are checked for overflow conditions.
6. **No side effects** — Checks don't modify source code or configuration files.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_ALLOWED_FEE_BPS` | 1000 | Maximum compliant platform fee (10%) |
| `MIN_COMPLIANT_GOAL` | 1 | Minimum compliant campaign goal |
| `MIN_COMPLIANT_CONTRIBUTION` | 1 | Minimum compliant contribution floor |
| `MIN_DEADLINE_BUFFER_SECS` | 60 | Minimum deadline buffer in seconds |
| `MIN_COVERAGE_PERCENT` | 95 | Minimum required test coverage |
| `MAX_FUNCTION_LINES` | 200 | Maximum function length |
| `MAX_COMPLEXITY` | 10 | Maximum cyclomatic complexity |

## Usage

### Basic Usage

```bash
# Run full security audit
./scripts/security_compliance_automation.sh --full-audit

# Run with verbose output
./scripts/security_compliance_automation.sh --full-audit --verbose

# Generate JSON report for CI/CD integration
./scripts/security_compliance_automation.sh --json > compliance-report.json

# Dry run - show what would be checked
./scripts/security_compliance_automation.sh --dry-run
```

### Targeted Checks

```bash
# Check only access control patterns
./scripts/security_compliance_automation.sh --check-only access_control

# Check only input validation
./scripts/security_compliance_automation.sh --check-only input_validation

# Check only event emission
./scripts/security_compliance_automation.sh --check-only event_emission

# Check only arithmetic safety
./scripts/security_compliance_automation.sh --check-only arithmetic

# Check only security patterns
./scripts/security_compliance_automation.sh --check-only security_patterns

# Check only documentation
./scripts/security_compliance_automation.sh --check-only documentation

# Check only storage integrity
./scripts/security_compliance_automation.sh --check-only storage

# Check only test coverage
./scripts/security_compliance_automation.sh --check-only coverage
```

## Check Categories

### 1. Git Repository Status (`--check-only git`)

Verifies the git repository is in a correct state before CI runs.

- **git_repository_clean** — Ensures no uncommitted changes
- **git_branch_naming** — Validates branch naming conventions

### 2. Code Formatting (`formatting`)

Validates code formatting standards.

- **rust_code_formatting** — Checks Rust formatting with `cargo fmt`
- **shell_script_formatting** — Validates shell scripts with `shellcheck`

### 3. Test Coverage (`--check-only coverage`)

Validates test coverage meets minimum threshold.

- **test_coverage_threshold** — Requires ≥95% coverage

### 4. Security Patterns (`--check-only security_patterns`)

Scans for suspicious security patterns.

- **suspicious_pattern_detection** — Detects `eval()`, `system()`, `exec()` usage
- **weak_crypto_detection** — Finds MD5, SHA1, DES, RC4 usage
- **hardcoded_secrets_detection** — Finds exposed passwords, API keys, tokens

### 5. Access Control (`--check-only access_control`)

Verifies authorization patterns are present.

- **contribute_authorization** — Ensures `require_auth` in contribute function
- **admin_function_authorization** — Validates admin function protection
- **pausable_pattern_implementation** — Checks pause/unpause guards

### 6. Input Validation (`--check-only input_validation`)

Validates input validation patterns.

- **contribution_amount_validation** — Checks amount validation
- **goal_validation** — Verifies goal > 0
- **deadline_validation** — Ensures deadline > now
- **address_validation** — Validates non-zero addresses
- **input_validation_comprehensive** — Overall validation coverage

### 7. Event Emission (`--check-only event_emission`)

Verifies audit trail for state changes.

- **contribution_event_emission** — Contributions are logged
- **withdrawal_event_emission** — Withdrawals are logged
- **status_change_event_emission** — Status changes are logged
- **admin_action_event_emission** — Admin actions are logged
- **compliance_audit_event_emission** — Compliance checks emit events

### 8. Arithmetic Safety (`--check-only arithmetic`)

Verifies overflow-safe arithmetic.

- **direct_addition_safety** — Checks for unchecked addition
- **checked_arithmetic_usage** — Validates `checked_add`, `saturating_add` usage
- **big_number_type_usage** — Ensures `u128`/`i128` for large values

### 9. Gas & Complexity (`complexity`)

Analyzes function complexity and gas usage.

- **bounded_iteration** — Ensures no unbounded loops
- **large_data_structure_guarding** — Validates large collection limits
- **recursive_function_detection** — Detects potential recursion
- **public_api_surface** — Counts public functions

### 10. Dependency Security (`--check-only dependencies`)

Validates dependency security.

- **cargo_manifest_present** — `Cargo.toml` exists
- **dependency_version_locking** — Versions are pinned
- **cargo_lock_present** — `Cargo.lock` committed
- **rustsec_audit_config** — RustSec audit configured
- **npm_dependency_audit** — NPM vulnerabilities checked

### 11. Documentation (`--check-only documentation`)

Validates documentation standards.

- **natspec_documentation** — NatSpec comments present
- **doctitle_annotations** — `@title` annotations used
- **docnotice_annotations** — `@notice` annotations used
- **docsecurity_annotations** — `@security` annotations used
- **readme_documentation** — README.md exists
- **security_policy_documentation** — SECURITY.md exists

### 12. Storage Integrity (`--check-only storage`)

Validates storage access patterns.

- **datakey_enum_pattern** — DataKey enum used
- **storage_type_distinction** — Instance vs persistent storage
- **storage_access_safety** — `has()` before `get()` pattern

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | All checks passed |
| 1 | One or more checks failed |
| 2 | Invalid arguments |

## JSON Output Format

When using `--json`, the output is machine-readable:

```json
{
  "version": "1.0.0",
  "timestamp": "2024-01-15T10:30:00+00:00",
  "summary": {
    "total_checks": 45,
    "passed": 42,
    "failed": 3,
    "warnings": 5,
    "all_passed": false
  },
  "failed_checks": [
    "test_coverage_threshold",
    "cargo_lock_present",
    "rustsec_audit_config"
  ],
  "warnings": [
    "npm_dependency_audit: npm audit not available",
    "public_api_surface: 25 public functions (consider reducing)"
  ],
  "coverage_percent": 87,
  "minimum_coverage_required": 95
}
```

## CI/CD Integration Examples

### GitHub Actions

```yaml
name: Security Compliance
on: [push, pull_request]

jobs:
  security-compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Security Compliance
        run: ./scripts/security_compliance_automation.sh --json > compliance-report.json
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: compliance-report
          path: compliance-report.json
      - name: Fail on Non-Compliance
        run: |
          if grep -q '"all_passed": false' compliance-report.json; then
            echo "Security compliance checks failed"
            exit 1
          fi
```

### GitLab CI

```yaml
security_compliance:
  stage: test
  script:
    - ./scripts/security_compliance_automation.sh --json > compliance-report.json
  artifacts:
    reports:
      json: compliance-report.json
    when: always
  allow_failure: false
```

### Jenkins

```groovy
pipeline {
    stages {
        stage('Security Compliance') {
            steps {
                sh './scripts/security_compliance_automation.sh --json > compliance-report.json'
                archiveArtifacts artifacts: 'compliance-report.json'
            }
            post {
                failure {
                    error 'Security compliance checks failed'
                }
            }
        }
    }
}
```

## Security Considerations

- All checks are read-only and cannot modify source code
- No network requests are made during execution
- Temporary files are created and cleaned up in test mode
- Color output is automatically disabled for non-TTY environments
- JSON output is deterministic for caching purposes

## Performance Notes

- Full audit typically completes in <30 seconds
- Targeted checks complete in <5 seconds
- No network I/O required
- Parallel execution safe for multiple instances

## Requirements

- Bash 4.0+ or compatible shell
- Optional: `cargo` for Rust formatting checks
- Optional: `shellcheck` for shell script linting
- Optional: `cargo-tarpaulin` for coverage reports
- Optional: `npm` for npm vulnerability scanning
- Optional: `jq` for JSON parsing in tests

## Troubleshooting

### Colors not showing

Ensure your terminal supports ANSI escape codes or use `--json` for machine-readable output.

### Tests failing on Windows

These scripts are designed for Unix/Linux environments. Use WSL, Docker, or CI/CD runners with bash.

### Coverage below threshold

Increase test coverage by adding unit tests for uncovered functions. Target ≥95% line coverage.

### False positives on security patterns

Review flagged code manually. Some patterns may be acceptable with proper input sanitization.

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial release with comprehensive security checks |

## License

Apache-2.0

## See Also

- [Rust Security Compliance Module](../contracts/crowdfund/src/security_compliance_automation.rs)
- [Rust Security Compliance Tests](../contracts/crowdfund/src/security_compliance_automation.test.rs)
- [Rust Security Compliance Docs](../contracts/crowdfund/src/security_compliance_automation.md)
- [Test Suite](./security_compliance_automation.test.sh)
