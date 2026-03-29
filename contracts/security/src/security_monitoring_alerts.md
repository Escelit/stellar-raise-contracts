# Security Monitoring Alerts for CI/CD

## Overview

Automated security monitoring and alerting system designed for continuous integration and deployment pipelines. This tool provides real-time security scanning and incident response capabilities to detect vulnerabilities, suspicious patterns, and security violations before they reach production.

## Features

### 1. Dependency Vulnerability Scanning

- Scans all Rust dependencies using `cargo-audit`
- Detects known CVEs and security advisories
- Generates JSON reports for integration with other tools
- Alerts on any vulnerable dependencies

### 2. Secret Detection

Scans for hardcoded secrets and sensitive information:

- API keys
- Passwords
- Tokens
- Private keys
- Other sensitive credentials

### 3. Unsafe Code Pattern Detection

Identifies potentially dangerous code patterns:

- `unsafe` blocks
- `panic!` macros
- `.unwrap()` calls
- `.expect()` calls
- Unchecked arithmetic operations

### 4. Integer Overflow Risk Analysis

- Detects unchecked arithmetic operations
- Recommends using checked arithmetic methods
- Prevents potential overflow vulnerabilities

### 5. File Permission Auditing

- Checks for world-writable files
- Ensures secure file permissions
- Prevents unauthorized access

### 6. Automated Alerting

- Webhook integration for real-time notifications
- Severity-based alert filtering
- Detailed logging for audit trails
- Comprehensive security reports

## Installation

### Prerequisites

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-audit
cargo install cargo-audit

# Optional: Install jq for JSON processing
# Ubuntu/Debian
sudo apt-get install jq

# macOS
brew install jq
```

### Setup

```bash
# Make the script executable
chmod +x contracts/security/src/security_monitoring_alerts.sh

# Run the script
./contracts/security/src/security_monitoring_alerts.sh
```

## Usage

### Basic Usage

```bash
# Run security scan
./contracts/security/src/security_monitoring_alerts.sh
```

### Configuration via Environment Variables

```bash
# Set severity threshold (low, medium, high, critical)
export SEVERITY_THRESHOLD=medium

# Configure webhook for alerts
export ALERT_WEBHOOK=https://your-webhook-url.com/alerts

# Set custom log file location
export LOG_FILE=/var/log/security_alerts.log

# Set custom scan results directory
export SCAN_RESULTS_DIR=.security-scans

# Run with configuration
./contracts/security/src/security_monitoring_alerts.sh
```

### CI/CD Integration

#### GitHub Actions

```yaml
name: Security Monitoring

on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install cargo-audit
        run: cargo install cargo-audit
      
      - name: Run Security Scan
        env:
          ALERT_WEBHOOK: ${{ secrets.SECURITY_WEBHOOK }}
        run: |
          chmod +x contracts/security/src/security_monitoring_alerts.sh
          ./contracts/security/src/security_monitoring_alerts.sh
```

#### GitLab CI

```yaml
security-scan:
  stage: test
  image: rust:latest
  before_script:
    - cargo install cargo-audit
  script:
    - chmod +x contracts/security/src/security_monitoring_alerts.sh
    - ./contracts/security/src/security_monitoring_alerts.sh
  artifacts:
    paths:
      - .security-scans/
    reports:
      junit: .security-scans/security_report.txt
```

#### Jenkins

```groovy
pipeline {
    agent any
    
    stages {
        stage('Security Scan') {
            steps {
                sh '''
                    cargo install cargo-audit
                    chmod +x contracts/security/src/security_monitoring_alerts.sh
                    ./contracts/security/src/security_monitoring_alerts.sh
                '''
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: '.security-scans/**/*', allowEmptyArchive: true
        }
    }
}
```

## Testing

Run the comprehensive test suite:

```bash
# Make test script executable
chmod +x contracts/security/src/security_monitoring_alerts.test.sh

# Run tests
./contracts/security/src/security_monitoring_alerts.test.sh
```

### Test Coverage

The test suite includes:

- ✅ Log file creation
- ✅ Scan results directory creation
- ✅ Secret pattern detection
- ✅ Unsafe code detection
- ✅ Panic pattern detection
- ✅ Clean code validation
- ✅ Integer overflow detection
- ✅ File permission checks
- ✅ World-writable file detection
- ✅ Multiple secret patterns
- ✅ Edge cases (empty directories)

## Output

### Console Output

The script provides color-coded console output:

- 🟢 Green: Success messages
- 🟡 Yellow: Warnings
- 🔴 Red: Errors and critical issues

### Log Files

Detailed logs are written to `security_alerts.log` (or custom location):

```
[2024-01-15 10:30:45] [INFO] Starting security monitoring scan...
[2024-01-15 10:30:46] [INFO] Checking security scanning dependencies...
[2024-01-15 10:30:47] [INFO] Scanning dependencies for vulnerabilities...
[2024-01-15 10:30:50] [WARN] Found 2 vulnerable dependencies
[2024-01-15 10:30:51] [HIGH] Dependency Vulnerabilities: Found 2 vulnerable dependencies
```

### Security Reports

Generated in `.security-scans/security_report.txt`:

```
Security Monitoring Report
Generated: Mon Jan 15 10:30:52 UTC 2024
========================================

Scan Results:
- Dependency vulnerabilities: Scanned
- Secret scanning: Completed
- Unsafe code patterns: Completed
- Panic patterns: Completed
- Integer overflow risks: Completed
- File permissions: Completed

For detailed logs, see: security_alerts.log
========================================
```

## Alert Severity Levels

- **CRITICAL**: System-breaking issues (missing dependencies, installation failures)
- **HIGH**: Security vulnerabilities (secrets, vulnerable dependencies, insecure permissions)
- **MEDIUM**: Code quality issues (unsafe code, excessive panics, integer overflow risks)
- **WARN**: Minor issues and informational messages
- **INFO**: Normal operation messages

## Webhook Integration

### Webhook Payload Format

```json
{
  "severity": "HIGH",
  "title": "Dependency Vulnerabilities",
  "message": "Found 2 vulnerable dependencies",
  "timestamp": "2024-01-15T10:30:51Z"
}
```

### Supported Webhook Services

- Slack
- Discord
- Microsoft Teams
- Custom webhook endpoints
- PagerDuty
- Opsgenie

### Example: Slack Integration

```bash
export ALERT_WEBHOOK=https://hooks.slack.com/services/YOUR/WEBHOOK/URL
./contracts/security/src/security_monitoring_alerts.sh
```

## Best Practices

1. **Run on Every Commit**: Integrate into CI/CD pipeline to catch issues early
2. **Set Appropriate Thresholds**: Configure severity levels based on your risk tolerance
3. **Monitor Logs**: Regularly review security logs for patterns
4. **Update Dependencies**: Keep cargo-audit and other tools up to date
5. **Respond Quickly**: Set up alerts to notify security team immediately
6. **Document Exceptions**: If certain patterns are acceptable, document why
7. **Regular Audits**: Periodically review and update scanning rules

## Troubleshooting

### cargo-audit Not Found

```bash
cargo install cargo-audit
```

### Permission Denied

```bash
chmod +x contracts/security/src/security_monitoring_alerts.sh
```

### Webhook Failures

- Check webhook URL is correct
- Verify network connectivity
- Check webhook service is operational
- Review webhook payload format

### False Positives

Edit the script to exclude specific patterns:

```bash
# Example: Exclude test files from secret scanning
grep -rniE "$pattern" contracts/ --include="*.rs" | grep -v "test"
```

## Performance Considerations

- Scans typically complete in under 1 minute for small projects
- Larger codebases may take 2-5 minutes
- Dependency scanning is the most time-consuming operation
- Consider running in parallel with other CI jobs

## Security Considerations

- Script requires read access to source code
- Webhook URLs should be stored as secrets
- Log files may contain sensitive information
- Scan results should be stored securely
- Consider encrypting webhook payloads

## Future Enhancements

Planned improvements:

1. SARIF format output for GitHub Security tab
2. Integration with vulnerability databases
3. Custom rule definitions
4. Machine learning for anomaly detection
5. Historical trend analysis
6. Automated remediation suggestions

## Contributing

To add new security checks:

1. Add scanning function following existing patterns
2. Add corresponding tests
3. Update documentation
4. Submit pull request

## License

This tool is part of the stellar-raise-contracts project and follows the same license.

## Support

For issues or questions:
- Open an issue on GitHub
- Check existing documentation
- Review test cases for examples
