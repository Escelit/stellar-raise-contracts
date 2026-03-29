#!/bin/bash

# Security Monitoring Alerts for CI/CD
# Automated security monitoring and alerting system for continuous integration
# Monitors for vulnerabilities, suspicious patterns, and security violations

set -e

# Color codes for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Configuration
SEVERITY_THRESHOLD="${SEVERITY_THRESHOLD:-medium}"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"
LOG_FILE="${LOG_FILE:-security_alerts.log}"
SCAN_RESULTS_DIR="${SCAN_RESULTS_DIR:-.security-scans}"

# Initialize
mkdir -p "$SCAN_RESULTS_DIR"

# Logging function
log() {
    local level=$1
    shift
    local message="$@"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

# Alert function
send_alert() {
    local severity=$1
    local title=$2
    local message=$3
    
    log "$severity" "$title: $message"
    
    # Send webhook notification if configured
    if [ -n "$ALERT_WEBHOOK" ]; then
        curl -X POST "$ALERT_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{\"severity\":\"$severity\",\"title\":\"$title\",\"message\":\"$message\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" \
            2>/dev/null || log "WARN" "Failed to send webhook alert"
    fi
}

# Check for cargo-audit
check_dependencies() {
    log "INFO" "Checking security scanning dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        send_alert "CRITICAL" "Missing Dependency" "cargo is not installed"
        exit 1
    fi
    
    if ! cargo audit --version &> /dev/null; then
        log "WARN" "cargo-audit not found, installing..."
        cargo install cargo-audit || {
            send_alert "ERROR" "Installation Failed" "Failed to install cargo-audit"
            exit 1
        }
    fi
    
    log "INFO" "All dependencies available"
}

# Scan for dependency vulnerabilities
scan_dependencies() {
    log "INFO" "Scanning dependencies for vulnerabilities..."
    
    local output_file="$SCAN_RESULTS_DIR/dependency_scan.json"
    
    if cargo audit --json > "$output_file" 2>&1; then
        log "INFO" "No dependency vulnerabilities found"
        return 0
    else
        local vuln_count=$(jq '.vulnerabilities.count // 0' "$output_file" 2>/dev/null || echo "0")
        
        if [ "$vuln_count" -gt 0 ]; then
            send_alert "HIGH" "Dependency Vulnerabilities" "Found $vuln_count vulnerable dependencies"
            
            # Extract vulnerability details
            if command -v jq &> /dev/null; then
                jq -r '.vulnerabilities.list[] | "- \(.advisory.id): \(.advisory.title) (Severity: \(.advisory.severity))"' "$output_file" | while read line; do
                    log "WARN" "$line"
                done
            fi
            
            return 1
        fi
    fi
}

# Scan for hardcoded secrets
scan_secrets() {
    log "INFO" "Scanning for hardcoded secrets..."
    
    local findings=0
    local patterns=(
        "password\s*=\s*['\"][^'\"]+['\"]"
        "api[_-]?key\s*=\s*['\"][^'\"]+['\"]"
        "secret\s*=\s*['\"][^'\"]+['\"]"
        "token\s*=\s*['\"][^'\"]+['\"]"
        "private[_-]?key\s*=\s*['\"][^'\"]+['\"]"
    )
    
    for pattern in "${patterns[@]}"; do
        if grep -rniE "$pattern" contracts/ --include="*.rs" --include="*.toml" 2>/dev/null | grep -v "test" | grep -v "example"; then
            findings=$((findings + 1))
            send_alert "HIGH" "Potential Secret Detected" "Found pattern matching: $pattern"
        fi
    done
    
    if [ $findings -eq 0 ]; then
        log "INFO" "No hardcoded secrets detected"
        return 0
    else
        log "WARN" "Found $findings potential secret patterns"
        return 1
    fi
}

# Check for unsafe code patterns
scan_unsafe_patterns() {
    log "INFO" "Scanning for unsafe code patterns..."
    
    local unsafe_count=$(grep -r "unsafe" contracts/ --include="*.rs" | grep -v "test" | wc -l)
    
    if [ "$unsafe_count" -gt 0 ]; then
        send_alert "MEDIUM" "Unsafe Code Detected" "Found $unsafe_count instances of unsafe code"
        grep -rn "unsafe" contracts/ --include="*.rs" | grep -v "test" | while read line; do
            log "WARN" "Unsafe code: $line"
        done
        return 1
    else
        log "INFO" "No unsafe code patterns detected"
        return 0
    fi
}

# Check for panic patterns
scan_panic_patterns() {
    log "INFO" "Scanning for panic patterns..."
    
    local panic_patterns=(
        "panic!"
        "unwrap()"
        "expect("
    )
    
    local findings=0
    
    for pattern in "${panic_patterns[@]}"; do
        local count=$(grep -r "$pattern" contracts/ --include="*.rs" | grep -v "test" | grep -v "//.*$pattern" | wc -l)
        if [ "$count" -gt 0 ]; then
            findings=$((findings + count))
            log "WARN" "Found $count instances of $pattern"
        fi
    done
    
    if [ $findings -gt 10 ]; then
        send_alert "MEDIUM" "Excessive Panic Patterns" "Found $findings panic-inducing patterns"
        return 1
    elif [ $findings -gt 0 ]; then
        log "INFO" "Found $findings panic patterns (within acceptable range)"
        return 0
    else
        log "INFO" "No panic patterns detected"
        return 0
    fi
}

# Check for integer overflow risks
scan_integer_overflow() {
    log "INFO" "Scanning for potential integer overflow risks..."
    
    local risky_ops=$(grep -rE "(\+|\-|\*|/)\s*[0-9]+" contracts/ --include="*.rs" | grep -v "test" | grep -v "checked_" | wc -l)
    
    if [ "$risky_ops" -gt 50 ]; then
        send_alert "MEDIUM" "Integer Overflow Risk" "Found $risky_ops unchecked arithmetic operations"
        log "WARN" "Consider using checked arithmetic operations"
        return 1
    else
        log "INFO" "Integer overflow risk within acceptable range"
        return 0
    fi
}

# Check file permissions
check_file_permissions() {
    log "INFO" "Checking file permissions..."
    
    local world_writable=$(find contracts/ -type f -perm -002 2>/dev/null | wc -l)
    
    if [ "$world_writable" -gt 0 ]; then
        send_alert "HIGH" "Insecure File Permissions" "Found $world_writable world-writable files"
        find contracts/ -type f -perm -002 2>/dev/null | while read file; do
            log "WARN" "World-writable file: $file"
        done
        return 1
    else
        log "INFO" "File permissions are secure"
        return 0
    fi
}

# Generate security report
generate_report() {
    log "INFO" "Generating security report..."
    
    local report_file="$SCAN_RESULTS_DIR/security_report.txt"
    
    cat > "$report_file" << EOF
Security Monitoring Report
Generated: $(date)
========================================

Scan Results:
- Dependency vulnerabilities: $([ -f "$SCAN_RESULTS_DIR/dependency_scan.json" ] && echo "Scanned" || echo "Not scanned")
- Secret scanning: Completed
- Unsafe code patterns: Completed
- Panic patterns: Completed
- Integer overflow risks: Completed
- File permissions: Completed

For detailed logs, see: $LOG_FILE

========================================
EOF
    
    log "INFO" "Report generated: $report_file"
    cat "$report_file"
}

# Main execution
main() {
    log "INFO" "Starting security monitoring scan..."
    
    local exit_code=0
    
    # Run all checks
    check_dependencies || exit_code=1
    scan_dependencies || exit_code=1
    scan_secrets || exit_code=1
    scan_unsafe_patterns || exit_code=1
    scan_panic_patterns || exit_code=1
    scan_integer_overflow || exit_code=1
    check_file_permissions || exit_code=1
    
    # Generate report
    generate_report
    
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ Security scan completed successfully${NC}"
        log "INFO" "Security scan completed successfully"
    else
        echo -e "${RED}✗ Security scan found issues${NC}"
        log "ERROR" "Security scan found issues"
        send_alert "HIGH" "Security Scan Failed" "Security monitoring detected issues requiring attention"
    fi
    
    exit $exit_code
}

# Run main function
main "$@"
