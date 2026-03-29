# Automated Security Dashboard for CI/CD

## Overview

The automated security dashboard provides comprehensive security monitoring and reporting for continuous integration and deployment pipelines. It aggregates security metrics, test results, and vulnerability scans into a unified dashboard view.

## Features

### 1. Automated Security Testing
- Runs all security-related tests
- Captures test output and results
- Provides pass/fail status for CI/CD gates

### 2. Vulnerability Scanning
- Integrates with cargo-audit for dependency scanning
- Detects known security vulnerabilities
- Provides remediation recommendations

### 3. Code Quality Analysis
- Runs clippy for static analysis
- Enforces security-related linting rules
- Identifies potential security issues

### 4. Test Coverage Reporting
- Generates test coverage metrics
- Highlights untested security-critical code
- Integrates with cargo-tarpaulin

### 5. HTML Dashboard Generation
- Creates visual security dashboard
- Displays key security metrics
- Provides historical trend analysis

## Usage

### Basic Usage

```bash
./security_dashboard.sh
```

### CI/CD Integration

Add to your CI/CD pipeline:

```yaml
# GitHub Actions example
- name: Run Security Dashboard
  run: |
    chmod +x contracts/security/security_dashboard.sh
    ./contracts/security/security_dashboard.sh
```

```yaml
# GitLab CI example
security_dashboard:
  script:
    - chmod +x contracts/security/security_dashboard.sh
    - ./contracts/security/security_dashboard.sh
  artifacts:
    paths:
      - security-reports/
```

## Output

The script generates several outputs in the `security-reports/` directory:

- `security_dashboard_[timestamp].html` - Visual dashboard
- `test_output.log` - Security test results
- `audit_output.log` - Vulnerability scan results
- `clippy_output.log` - Code quality analysis
- `coverage_output.log` - Test coverage metrics

## Dashboard Sections

### Security Metrics
- Security test pass/fail status
- Number of detected vulnerabilities
- Code quality score
- Test coverage percentage

### Recent Scans
- Dependency audit results
- Static analysis findings
- Coverage analysis

### Trend Analysis
- Historical security metrics
- Vulnerability trends over time
- Test coverage trends

## Dependencies

### Required
- `cargo` - Rust package manager
- `rustc` - Rust compiler

### Optional (for full functionality)
- `cargo-audit` - Dependency vulnerability scanning
  ```bash
  cargo install cargo-audit
  ```

- `cargo-clippy` - Linting and static analysis
  ```bash
  rustup component add clippy
  ```

- `cargo-tarpaulin` - Test coverage analysis
  ```bash
  cargo install cargo-tarpaulin
  ```

## Exit Codes

- `0` - All security checks passed
- `1` - Security tests failed or critical issues detected
- `2` - Missing required dependencies

## Configuration

The script can be configured by modifying variables at the top:

```bash
# Report directory
REPORT_DIR="security-reports"

# Custom report naming
REPORT_FILE="${REPORT_DIR}/custom_name.html"
```

## Security Checks Performed

1. **Dependency Verification**
   - Checks for required tools
   - Validates environment setup

2. **Security Test Execution**
   - Runs all security module tests
   - Validates security implementations

3. **Vulnerability Scanning**
   - Scans dependencies for known CVEs
   - Checks for outdated packages

4. **Code Quality Analysis**
   - Runs static analysis
   - Enforces security linting rules

5. **Coverage Analysis**
   - Measures test coverage
   - Identifies untested code paths

## Best Practices

1. **Run in CI/CD**: Integrate into every build pipeline
2. **Review Reports**: Regularly review generated dashboards
3. **Track Trends**: Monitor security metrics over time
4. **Set Gates**: Fail builds on critical security issues
5. **Update Dependencies**: Keep security tools current

## Troubleshooting

### Script fails with "command not found"
Install missing dependencies listed in the Dependencies section.

### Tests fail unexpectedly
Check `test_output.log` for detailed error messages.

### Dashboard not generated
Ensure write permissions for the `security-reports/` directory.

### Cargo-audit warnings
Update dependencies or review vulnerability details in `audit_output.log`.

## Testing

Run the test suite:

```bash
chmod +x security_dashboard.test.sh
./security_dashboard.test.sh
```

The test suite validates:
- Script structure and syntax
- Required functions presence
- Error handling
- Output generation
- HTML template validity

## Integration Examples

### Jenkins Pipeline

```groovy
stage('Security Dashboard') {
    steps {
        sh 'chmod +x contracts/security/security_dashboard.sh'
        sh './contracts/security/security_dashboard.sh'
    }
    post {
        always {
            publishHTML([
                reportDir: 'security-reports',
                reportFiles: 'security_dashboard_*.html',
                reportName: 'Security Dashboard'
            ])
        }
    }
}
```

### CircleCI

```yaml
- run:
    name: Security Dashboard
    command: |
      chmod +x contracts/security/security_dashboard.sh
      ./contracts/security/security_dashboard.sh
- store_artifacts:
    path: security-reports
```

## Future Enhancements

- Integration with external security services
- Real-time vulnerability alerts
- Automated remediation suggestions
- Historical trend visualization
- Multi-project dashboard aggregation
- Slack/email notifications
- Custom security policy enforcement
