#!/usr/bin/env bash
# =============================================================================
# security_compliance_automation.sh
# =============================================================================
# @title   SecurityComplianceAutomation — Automated CI/CD Security Compliance
# @notice  Shell script for automated security compliance checks in CI/CD pipelines.
#          Validates code patterns, security assumptions, and compliance requirements.
# @dev     Read-only operations only — no state modifications.
#          Designed for Stellar/Soroban smart contract projects.
#
# @author  Security Compliance Team
# @license Apache-2.0
#
# Security Assumptions:
#   1. Read-only — No function writes to storage or state files
#   2. Permissionless — No privileged access required to run checks
#   3. Deterministic — Same input produces same output
#   4. Bounded execution — No unbounded loops or iterations
#   5. Safe arithmetic — All operations checked for overflow
#
# Usage:
#   ./security_compliance_automation.sh [OPTIONS]
#   ./security_compliance_automation.sh --full-audit
#   ./security_compliance_automation.sh --check-only access_control
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────

readonly SCRIPT_NAME="security_compliance_automation"
readonly VERSION="1.0.0"
readonly MIN_COVERAGE_PERCENT=95

# Color codes for output
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[0;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_BOLD='\033[1m'

# ── Global State ─────────────────────────────────────────────────────────────

declare -i TOTAL_CHECKS=0
declare -i PASSED_CHECKS=0
declare -i FAILED_CHECKS=0
declare -i WARNINGS=0

declare -a FAILED_CHECKS_LIST=()
declare -a PASSED_CHECKS_LIST=()
declare -a WARNINGS_LIST=()

declare DRY_RUN=false
declare VERBOSE=false
declare JSON_OUTPUT=false
declare FULL_AUDIT=false
declare CHECK_ONLY=""
declare PROJECT_ROOT=""
declare EXIT_CODE=0

# ── Constants ─────────────────────────────────────────────────────────────────

readonly MAX_ALLOWED_FEE_BPS=1000
readonly MIN_COMPLIANT_GOAL=1
readonly MIN_COMPLIANT_CONTRIBUTION=1
readonly MIN_DEADLINE_BUFFER_SECS=60
readonly MAX_FUNCTION_LINES=200
readonly MAX_COMPLEXITY=10

# ── Security Patterns ────────────────────────────────────────────────────────

readonly SUSPICIOUS_PATTERNS=(
    "eval("
    "system("
    "exec("
    "shell_exec("
    "passthru("
    "popen("
    "proc_open("
    "subprocess("
    "os.system("
    "os.popen("
    "Runtime.getRuntime().exec"
    "ProcessBuilder"
    "\\\\.exec\\\\("
    "\\\\.execute\\\\("
)

readonly UNSAFE_CRYPTO_PATTERNS=(
    "MD5"
    "SHA1"
    "DES"
    "RC4"
    "ECB"
    "NoPadding"
)

readonly HARDCODED_SECRETS_PATTERNS=(
    "password\s*=\s*[\"'][^\"']+[\"']"
    "api_key\s*=\s*[\"'][^\"']+[\"']"
    "secret\s*=\s*[\"'][^\"']+[\"']"
    "token\s*=\s*[\"'][A-Za-z0-9]{20,}[\"']"
    "sk_live_"
    "pk_live_"
    "0x[a-fA-F0-9]{64}"
)

# ── Utility Functions ──────────────────────────────────────────────────────────

# @title print_header
# @notice Prints a formatted section header
# @param  $1 Header text
print_header() {
    local header="$1"
    echo ""
    echo -e "${COLOR_BLUE}${COLOR_BOLD}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo -e "${COLOR_BLUE}${COLOR_BOLD}  ${header}${COLOR_RESET}"
    echo -e "${COLOR_BLUE}${COLOR_BOLD}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo ""
}

# @title print_check
# @notice Prints check name with status
# @param  $1 Check name
# @param  $2 Status (PASS/FAIL/WARN)
print_check() {
    local check_name="$1"
    local status="$2"
    local message="${3:-}"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    case "$status" in
        PASS)
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
            PASSED_CHECKS_LIST+=("$check_name")
            echo -e "  ${COLOR_GREEN}✓${COLOR_RESET} ${check_name}"
            ;;
        FAIL)
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            FAILED_CHECKS_LIST+=("$check_name")
            EXIT_CODE=1
            echo -e "  ${COLOR_RED}✗${COLOR_RESET} ${check_name}"
            ;;
        WARN)
            WARNINGS=$((WARNINGS + 1))
            WARNINGS_LIST+=("$check_name: $message")
            echo -e "  ${COLOR_YELLOW}⚠${COLOR_RESET} ${check_name}"
            ;;
    esac
    
    if [[ -n "$message" && "$status" != "WARN" ]]; then
        echo "    → $message"
    fi
}

# @title print_info
# @notice Prints informational message
# @param  $1 Message text
print_info() {
    echo -e "  ${COLOR_BLUE}ℹ${COLOR_RESET} $*"
}

# @title print_success
# @notice Prints success message
# @param  $1 Message text
print_success() {
    echo -e "  ${COLOR_GREEN}✓${COLOR_RESET} $*"
}

# @title print_error
# @notice Prints error message
# @param  $1 Message text
print_error() {
    echo -e "  ${COLOR_RED}✗${COLOR_RESET} $*" >&2
}

# @title print_warning
# @notice Prints warning message
# @param  $1 Message text
print_warning() {
    echo -e "  ${COLOR_YELLOW}⚠${COLOR_RESET} $*"
}

# @title print_summary
# @notice Prints final summary of compliance checks
print_summary() {
    echo ""
    echo -e "${COLOR_BOLD}─────────────────────────────────────────────────────────────────${COLOR_RESET}"
    echo -e "  ${COLOR_BOLD}COMPLIANCE SUMMARY${COLOR_RESET}"
    echo -e "${COLOR_BOLD}─────────────────────────────────────────────────────────────────${COLOR_RESET}"
    echo ""
    echo -e "  Total Checks:  ${COLOR_BOLD}${TOTAL_CHECKS}${COLOR_RESET}"
    echo -e "  ${COLOR_GREEN}Passed:${COLOR_RESET}      ${PASSED_CHECKS}"
    echo -e "  ${COLOR_RED}Failed:${COLOR_RESET}      ${FAILED_CHECKS}"
    echo -e "  ${COLOR_YELLOW}Warnings:${COLOR_RESET}    ${WARNINGS}"
    echo ""
    
    if [[ ${#FAILED_CHECKS_LIST[@]} -gt 0 ]]; then
        echo -e "  ${COLOR_RED}Failed Checks:${COLOR_RESET}"
        for check in "${FAILED_CHECKS_LIST[@]}"; do
            echo -e "    • $check"
        done
        echo ""
    fi
    
    if [[ ${#WARNINGS_LIST[@]} -gt 0 ]]; then
        echo -e "  ${COLOR_YELLOW}Warnings:${COLOR_RESET}"
        for warning in "${WARNINGS_LIST[@]}"; do
            echo -e "    • $warning"
        done
        echo ""
    fi
    
    local coverage_color="${COLOR_GREEN}"
    if (( COVERAGE < MIN_COVERAGE_PERCENT )); then
        coverage_color="${COLOR_RED}"
    fi
    echo -e "  Test Coverage: ${coverage_color}${COVERAGE}%${COLOR_RESET} (minimum: ${MIN_COVERAGE_PERCENT}%)"
    echo ""
}

# ── CI/CD Specific Functions ──────────────────────────────────────────────────

# @title check_git_status
# @notice Verifies git repository is in correct state
# @dev   Ensures clean working directory before CI runs
# @return Exit code 0 if clean, 1 otherwise
check_git_status() {
    print_header "Git Repository Status"
    
    # Check if we're in a git repository
    if ! git rev-parse --is-inside-work-tree &>/dev/null; then
        print_check "git_repository_clean" "WARN" "Not in a git repository"
        return 0
    fi
    
    # Check for uncommitted changes
    if [[ -n "$(git status --porcelain)" ]]; then
        print_check "git_repository_clean" "FAIL" "Uncommitted changes detected"
        if [[ "$VERBOSE" == "true" ]]; then
            git status --short
        fi
        return 1
    fi
    
    print_check "git_repository_clean" "PASS"
    
    # Check branch name convention
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached")
    if [[ "$branch" =~ ^(feature|bugfix|hotfix|release)/ ]]; then
        print_check "git_branch_naming" "PASS" "Branch: $branch"
    else
        print_check "git_branch_naming" "WARN" "Non-standard branch name: $branch"
    fi
}

# @title check_code_formatting
# @notice Validates code formatting standards
# @dev   Checks for consistent formatting using available tools
check_code_formatting() {
    print_header "Code Formatting Compliance"
    
    # Check Rust formatting if cargo available
    if command -v cargo &>/dev/null; then
        if cargo fmt --check &>/dev/null; then
            print_check "rust_code_formatting" "PASS"
        else
            print_check "rust_code_formatting" "FAIL" "Run 'cargo fmt' to fix formatting"
        fi
    else
        print_check "rust_code_formatting" "WARN" "cargo not available"
    fi
    
    # Check shell script formatting if shellcheck available
    if command -v shellcheck &>/dev/null; then
        local shell_errors=0
        while IFS= read -r -d '' script; do
            if ! shellcheck "$script" &>/dev/null; then
                shell_errors=$((shell_errors + 1))
            fi
        done < <(find . -name "*.sh" -type f -print0 2>/dev/null || true)
        
        if [[ $shell_errors -eq 0 ]]; then
            print_check "shell_script_formatting" "PASS"
        else
            print_check "shell_script_formatting" "FAIL" "$shell_errors shell script(s) with issues"
        fi
    else
        print_check "shell_script_formatting" "WARN" "shellcheck not available"
    fi
}

# @title check_test_coverage
# @notice Validates test coverage meets minimum threshold
# @dev   Runs cargo tarpaulin or similar coverage tool
# @return Coverage percentage stored in COVERAGE variable
COVERAGE=0
check_test_coverage() {
    print_header "Test Coverage Compliance"
    
    # Check for cargo-tarpaulin
    if command -v cargo-tarpaulin &>/dev/null; then
        local coverage_output
        if coverage_output=$(cargo tarpaulin --out Json 2>/dev/null); then
            COVERAGE=$(echo "$coverage_output" | jq -r '.line_rate // 0' 2>/dev/null || echo "0")
            COVERAGE=${COVERAGE%.*}  # Remove decimal part
            
            if [[ $COVERAGE -ge $MIN_COVERAGE_PERCENT ]]; then
                print_check "test_coverage_threshold" "PASS" "${COVERAGE}% coverage"
            else
                print_check "test_coverage_threshold" "FAIL" "${COVERAGE}% (minimum: ${MIN_COVERAGE_PERCENT}%)"
            fi
        else
            print_check "test_coverage_threshold" "WARN" "Could not generate coverage report"
            COVERAGE=0
        fi
    elif command -v cargo &>/dev/null; then
        # Fallback: count test functions
        local test_count
        test_count=$(grep -r "#\[test\]" --include="*.rs" . 2>/dev/null | wc -l || echo "0")
        local function_count
        function_count=$(grep -r "^pub fn\|^fn" --include="*.rs" . 2>/dev/null | wc -l || echo "0")
        
        if [[ $function_count -gt 0 ]]; then
            local estimated_coverage=$((test_count * 100 / function_count))
            if [[ $estimated_coverage -ge $MIN_COVERAGE_PERCENT ]]; then
                print_check "test_coverage_threshold" "PASS" "~${estimated_coverage}% (estimated)"
            else
                print_check "test_coverage_threshold" "WARN" "~${estimated_coverage}% (estimated, minimum: ${MIN_COVERAGE_PERCENT}%)"
            fi
            COVERAGE=$estimated_coverage
        else
            print_check "test_coverage_threshold" "WARN" "Could not determine coverage"
            COVERAGE=0
        fi
    else
        print_check "test_coverage_threshold" "WARN" "Coverage tools not available"
        COVERAGE=0
    fi
}

# @title check_security_patterns
# @notice Scans for suspicious security patterns
# @dev   Detects potentially unsafe code patterns
check_security_patterns() {
    print_header "Security Pattern Analysis"
    
    local found_issues=0
    
    # Check for suspicious patterns in shell scripts
    for pattern in "${SUSPICIOUS_PATTERNS[@]}"; do
        if grep -rnE "$pattern" --include="*.sh" . 2>/dev/null | grep -v "^./scripts/security_compliance_automation.sh" | grep -qv "^$"; then
            local matches
            matches=$(grep -rnE "$pattern" --include="*.sh" . 2>/dev/null | grep -v "^./scripts/security_compliance_automation.sh" | wc -l)
            if [[ $matches -gt 0 ]]; then
                print_check "suspicious_pattern_detection:$pattern" "FAIL" "Found $matches occurrence(s)"
                found_issues=$((found_issues + 1))
            fi
        fi
    done
    
    if [[ $found_issues -eq 0 ]]; then
        print_check "suspicious_pattern_detection" "PASS"
    fi
    
    # Check for weak cryptography
    found_issues=0
    for pattern in "${UNSAFE_CRYPTO_PATTERNS[@]}"; do
        if grep -rnE "$pattern" --include="*.rs" . 2>/dev/null | grep -qv "^$"; then
            local matches
            matches=$(grep -rnE "$pattern" --include="*.rs" . 2>/dev/null | wc -l)
            if [[ $matches -gt 0 ]]; then
                print_check "weak_crypto_detection:$pattern" "FAIL" "Found $matches occurrence(s)"
                found_issues=$((found_issues + 1))
            fi
        fi
    done
    
    if [[ $found_issues -eq 0 ]]; then
        print_check "weak_crypto_detection" "PASS"
    fi
    
    # Check for hardcoded secrets
    found_issues=0
    for pattern in "${HARDCODED_SECRETS_PATTERNS[@]}"; do
        if grep -rnE "$pattern" --include="*.rs" --include="*.sh" --include="*.ts" --include="*.js" . 2>/dev/null | grep -qv "^$"; then
            local matches
            matches=$(grep -rnE "$pattern" --include="*.rs" --include="*.sh" --include="*.ts" --include="*.js" . 2>/dev/null | wc -l)
            if [[ $matches -gt 0 ]]; then
                print_check "hardcoded_secrets_detection" "FAIL" "Found $matches potential secret(s)"
                found_issues=$((found_issues + 1))
                break
            fi
        fi
    done
    
    if [[ $found_issues -eq 0 ]]; then
        print_check "hardcoded_secrets_detection" "PASS"
    fi
}

# @title check_access_control
# @notice Verifies access control patterns in contracts
# @dev   Ensures proper authorization checks are present
check_access_control() {
    print_header "Access Control Verification"
    
    # Check for require_auth presence in sensitive functions
    local access_control_issues=0
    
    # Check contribute function has authorization
    if grep -q "fn contribute" contracts/crowdfund/src/*.rs 2>/dev/null; then
        if grep -A 20 "fn contribute" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "require_auth"; then
            print_check "contribute_authorization" "PASS"
        else
            print_check "contribute_authorization" "FAIL" "Missing require_auth in contribute"
            access_control_issues=$((access_control_issues + 1))
        fi
    fi
    
    # Check admin functions have authorization
    if grep -q "upgrade\|set_admin\|transfer_admin" contracts/crowdfund/src/*.rs 2>/dev/null; then
        if grep -E "fn (upgrade|set_admin|transfer_admin)" contracts/crowdfund/src/*.rs 2>/dev/null | while read -r func; do
            local func_name
            func_name=$(echo "$func" | grep -oE "fn [a-z_]+" | head -1)
            if ! grep -A 10 "$func" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "require_auth\|require_administrator"; then
                print_check "admin_function_authorization:$func_name" "FAIL" "Missing authorization in $func_name"
                access_control_issues=$((access_control_issues + 1))
            fi
        done; then
            :  # All admin functions have auth
        fi
    fi
    
    if [[ $access_control_issues -eq 0 ]]; then
        print_check "admin_function_authorization" "PASS"
    fi
    
    # Check for pausable pattern
    if grep -q "paused\|Paused" contracts/crowdfund/src/*.rs 2>/dev/null; then
        if grep -q "assert_not_paused\|require_not_paused" contracts/crowdfund/src/*.rs 2>/dev/null; then
            print_check "pausable_pattern_implementation" "PASS"
        else
            print_check "pausable_pattern_implementation" "WARN" "Paused flag exists but guard not found"
        fi
    fi
}

# @title check_input_validation
# @notice Validates input validation patterns
# @dev   Ensures functions validate inputs properly
check_input_validation() {
    print_header "Input Validation Compliance"
    
    # Check for zero/negative value validation
    local validation_issues=0
    
    # Check contribution amount validation
    if grep -A 15 "fn contribute" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "amount.*>.*0\|amount.*<.*MIN\|amount.*>=.*MIN"; then
        print_check "contribution_amount_validation" "PASS"
    elif grep -A 15 "fn contribute" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "assert\|require"; then
        print_check "contribution_amount_validation" "WARN" "Uses assert/require for validation"
    else
        print_check "contribution_amount_validation" "FAIL" "No explicit amount validation"
        validation_issues=$((validation_issues + 1))
    fi
    
    # Check goal validation
    if grep -A 10 "fn initialize" contracts/crowdfund/src/*.rs 2>/dev/null | grep -qE "goal.*>.*0|goal.*>=.*1"; then
        print_check "goal_validation" "PASS"
    else
        print_check "goal_validation" "WARN" "Goal validation not explicitly verified"
    fi
    
    # Check deadline validation
    if grep -A 10 "fn initialize" contracts/crowdfund/src/*.rs 2>/dev/null | grep -qE "deadline.*>.*now|deadline.*>=.*ledger"; then
        print_check "deadline_validation" "PASS"
    else
        print_check "deadline_validation" "WARN" "Deadline validation not explicitly verified"
    fi
    
    # Check address validation
    if grep -A 5 "fn initialize" contracts/crowdfund/src/*.rs 2>/dev/null | grep -qE "creator.*!=.*Address|admin.*!=.*Address"; then
        print_check "address_validation" "PASS"
    else
        print_check "address_validation" "WARN" "Address non-zero validation not verified"
    fi
    
    if [[ $validation_issues -eq 0 ]]; then
        print_check "input_validation_comprehensive" "PASS"
    else
        print_check "input_validation_comprehensive" "FAIL" "$validation_issues critical validation(s) missing"
    fi
}

# @title check_event_emission
# @notice Verifies event emission for important state changes
# @dev   Ensures audit trail is maintained
check_event_emission() {
    print_header "Event Emission Compliance"
    
    local event_checks_passed=0
    local event_checks_total=0
    
    # Check for contribution events
    event_checks_total=$((event_checks_total + 1))
    if grep -q "contribution\|Contributed\|contribute" contracts/crowdfund/src/*.rs 2>/dev/null && \
       grep -A 10 "fn contribute" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "emit_event\|events()"; then
        print_check "contribution_event_emission" "PASS"
        event_checks_passed=$((event_checks_passed + 1))
    else
        print_check "contribution_event_emission" "FAIL" "Contribution events not found"
    fi
    
    # Check for withdrawal events
    event_checks_total=$((event_checks_total + 1))
    if grep -q "withdraw\|Withdrawal\|withdrawn" contracts/crowdfund/src/*.rs 2>/dev/null && \
       grep -A 10 "fn withdraw" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "emit_event\|events()"; then
        print_check "withdrawal_event_emission" "PASS"
        event_checks_passed=$((event_checks_passed + 1))
    else
        print_check "withdrawal_event_emission" "WARN" "Withdrawal events not found"
    fi
    
    # Check for status change events
    event_checks_total=$((event_checks_total + 1))
    if grep -A 10 "fn finalize\|fn cancel" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "emit_event\|events()"; then
        print_check "status_change_event_emission" "PASS"
        event_checks_passed=$((event_checks_passed + 1))
    else
        print_check "status_change_event_emission" "WARN" "Status change events not found"
    fi
    
    # Check for admin action events
    event_checks_total=$((event_checks_total + 1))
    if grep -A 10 "fn upgrade\|fn pause\|fn unpause" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "emit_event\|events()"; then
        print_check "admin_action_event_emission" "PASS"
        event_checks_passed=$((event_checks_passed + 1))
    else
        print_check "admin_action_event_emission" "WARN" "Admin action events not found"
    fi
    
    # Check for compliance audit events
    event_checks_total=$((event_checks_total + 1))
    if grep -q "compliance_audit\|compliance_summary" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "compliance_audit_event_emission" "PASS"
        event_checks_passed=$((event_checks_passed + 1))
    else
        print_check "compliance_audit_event_emission" "WARN" "Compliance audit events not implemented"
    fi
    
    if [[ $event_checks_passed -eq $event_checks_total ]]; then
        print_check "event_emission_comprehensive" "PASS"
    else
        print_check "event_emission_comprehensive" "WARN" "$event_checks_passed/$event_checks_total checks passed"
    fi
}

# @title check_arithmetic_safety
# @notice Verifies arithmetic operations are overflow-safe
# @dev   Ensures checked arithmetic or SafeMath patterns are used
check_arithmetic_safety() {
    print_header "Arithmetic Safety Compliance"
    
    # Check for checked arithmetic usage
    local unsafe_count=0
    
    # Look for direct arithmetic on sensitive values
    if grep -E "\+\s*[a-z_]+\s*\+\s*[a-z_]+|total.*\+" contracts/crowdfund/src/*.rs 2>/dev/null | grep -qv "checked_add\|saturating_add"; then
        print_check "direct_addition_safety" "WARN" "Direct addition found - verify overflow protection"
    else
        print_check "direct_addition_safety" "PASS"
    fi
    
    # Check for checked arithmetic methods
    if grep -q "checked_add\|checked_sub\|checked_mul\|saturating_add\|saturating_sub" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "checked_arithmetic_usage" "PASS"
    else
        print_check "checked_arithmetic_usage" "WARN" "No explicit checked arithmetic found"
    fi
    
    # Check for u128/i128 usage (native big number support)
    if grep -qE "u128|i128" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "big_number_type_usage" "PASS"
    else
        print_check "big_number_type_usage" "WARN" "No explicit big number types found"
    fi
}

# @title check_gas_complexity
# @notice Analyzes function complexity and gas usage
# @dev   Ensures functions are bounded and won't cause out-of-gas
check_gas_complexity() {
    print_header "Gas & Complexity Compliance"
    
    # Check for unbounded iterations
    local unbounded_found=0
    
    # Check for iterators over collections
    if grep -E "for.*in.*contributors|for.*in.*pledges|for.*in.*contributions" contracts/crowdfund/src/*.rs 2>/dev/null; then
        if grep -E "for.*in.*contributors" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "take\|skip\|limit"; then
            print_check "bounded_iteration" "PASS"
        else
            print_check "bounded_iteration" "WARN" "Potential unbounded iteration found"
            unbounded_found=$((unbounded_found + 1))
        fi
    else
        print_check "bounded_iteration" "PASS"
    fi
    
    # Check for large data structures
    if grep -qE "Vec::<.*>\{len.*1000|BTreeMap.*1000" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "large_data_structure_guarding" "WARN" "Large data structures found - verify bounds"
    else
        print_check "large_data_structure_guarding" "PASS"
    fi
    
    # Check for recursive functions
    if grep -E "fn.*\{[^}]*\bself\.\b" contracts/crowdfund/src/*.rs 2>/dev/null | grep -q "self\."; then
        print_check "recursive_function_detection" "WARN" "Potential recursion detected"
    else
        print_check "recursive_function_detection" "PASS"
    fi
    
    # Count public functions (complexity indicator)
    local public_func_count
    public_func_count=$(grep -c "^    pub fn" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    if [[ $public_func_count -lt 20 ]]; then
        print_check "public_api_surface" "PASS" "$public_func_count public functions"
    else
        print_check "public_api_surface" "WARN" "$public_func_count public functions (consider reducing)"
    fi
}

# @title check_dependency_security
# @notice Validates dependency security
# @dev   Checks Cargo.toml for known vulnerabilities
check_dependency_security() {
    print_header "Dependency Security Compliance"
    
    local dep_checks_passed=0
    
    # Check Cargo.toml exists
    if [[ -f "Cargo.toml" ]]; then
        print_check "cargo_manifest_present" "PASS"
        
        # Check for version-locked dependencies
        if grep -qE "^[^#]*version\s*=\s*\"[0-9]" Cargo.toml 2>/dev/null; then
            print_check "dependency_version_locking" "PASS"
        else
            print_check "dependency_version_locking" "WARN" "Some dependencies may use floating versions"
        fi
        
        # Check for Cargo.lock
        if [[ -f "Cargo.lock" ]]; then
            print_check "cargo_lock_present" "PASS"
        else
            print_check "cargo_lock_present" "FAIL" "Cargo.lock missing - run 'cargo update'"
        fi
    else
        print_check "cargo_manifest_present" "WARN" "No Cargo.toml found"
    fi
    
    # Check for audit.toml (RustSec)
    if [[ -f "audit.toml" ]]; then
        print_check "rustsec_audit_config" "PASS"
    else
        print_check "rustsec_audit_config" "WARN" "No audit.toml found - consider adding RustSec configuration"
    fi
    
    # Check package.json for vulnerabilities if exists
    if [[ -f "package.json" ]]; then
        if command -v npm &>/dev/null && npm audit --dry-run --json &>/dev/null; then
            local vuln_count
            vuln_count=$(npm audit --dry-run --json 2>/dev/null | jq -r '.metadata.vulnerabilities.total // 0')
            if [[ $vuln_count -eq 0 ]]; then
                print_check "npm_dependency_audit" "PASS"
            else
                print_check "npm_dependency_audit" "FAIL" "$vuln_count npm vulnerabilities found"
            fi
        else
            print_check "npm_dependency_audit" "WARN" "npm audit not available"
        fi
    fi
}

# @title check_documentation_compliance
# @notice Validates documentation standards
# @dev   Ensures NatSpec-style comments are present
check_documentation_compliance() {
    print_header "Documentation Compliance"
    
    local doc_checks_passed=0
    local doc_checks_total=0
    
    # Check for NatSpec-style comments in Rust
    doc_checks_total=$((doc_checks_total + 1))
    local natspec_count
    natspec_count=$(grep -cE "^///|^/\*!|^//!" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    if [[ $natspec_count -gt 10 ]]; then
        print_check "natspec_documentation" "PASS" "$natspec_count NatSpec comments"
        doc_checks_passed=$((doc_checks_passed + 1))
    else
        print_check "natspec_documentation" "WARN" "Only $natspec_count NatSpec comments found"
    fi
    
    # Check for @title annotations
    doc_checks_total=$((doc_checks_total + 1))
    local title_count
    title_count=$(grep -c "@title" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    if [[ $title_count -gt 0 ]]; then
        print_check "doctitle_annotations" "PASS" "$title_count @title annotations"
        doc_checks_passed=$((doc_checks_passed + 1))
    else
        print_check "doctitle_annotations" "WARN" "No @title annotations found"
    fi
    
    # Check for @notice annotations
    doc_checks_total=$((doc_checks_total + 1))
    local notice_count
    notice_count=$(grep -c "@notice" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    if [[ $notice_count -gt 0 ]]; then
        print_check "docnotice_annotations" "PASS" "$notice_count @notice annotations"
        doc_checks_passed=$((doc_checks_passed + 1))
    else
        print_check "docnotice_annotations" "WARN" "No @notice annotations found"
    fi
    
    # Check for @security annotations
    doc_checks_total=$((doc_checks_total + 1))
    local security_count
    security_count=$(grep -c "@security" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    if [[ $security_count -gt 0 ]]; then
        print_check "docsecurity_annotations" "PASS" "$security_count @security annotations"
        doc_checks_passed=$((doc_checks_passed + 1))
    else
        print_check "docsecurity_annotations" "WARN" "No @security annotations found"
    fi
    
    # Check for README
    if [[ -f "README.md" ]]; then
        print_check "readme_documentation" "PASS"
    else
        print_check "readme_documentation" "FAIL" "README.md not found"
    fi
    
    # Check for SECURITY.md
    if [[ -f "SECURITY.md" ]]; then
        print_check "security_policy_documentation" "PASS"
    else
        print_check "security_policy_documentation" "WARN" "SECURITY.md not found"
    fi
    
    if [[ $doc_checks_passed -eq $doc_checks_total ]]; then
        print_check "documentation_comprehensive" "PASS"
    fi
}

# @title check_storage_integrity
# @notice Validates storage access patterns
# @dev   Ensures proper key management and storage patterns
check_storage_integrity() {
    print_header "Storage Integrity Compliance"
    
    # Check for DataKey enum usage
    if grep -q "DataKey\|DataKey::" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "datakey_enum_pattern" "PASS"
    else
        print_check "datakey_enum_pattern" "WARN" "DataKey enum not found"
    fi
    
    # Check for instance vs persistent storage distinction
    if grep -qE "instance\(\)|persistent\(\)|temporary\(\)" contracts/crowdfund/src/*.rs 2>/dev/null; then
        print_check "storage_type_distinction" "PASS"
    else
        print_check "storage_type_distinction" "WARN" "No explicit storage type distinction"
    fi
    
    # Check for storage().has() before get()
    local safe_storage_count=0
    local total_storage_count=0
    
    while IFS= read -r line; do
        total_storage_count=$((total_storage_count + 1))
        local func_name
        func_name=$(echo "$line" | grep -oE "fn [a-z_]+" | head -1)
        # Check if function has has() check before get()
        local func_body
        func_body=$(grep -A 20 "$line" contracts/crowdfund/src/*.rs 2>/dev/null | head -20)
        if echo "$func_body" | grep -q "has(&"; then
            safe_storage_count=$((safe_storage_count + 1))
        fi
    done < <(grep -E "fn [a-z_]+\([^)]*\)\s*->" contracts/crowdfund/src/*.rs 2>/dev/null)
    
    if [[ $total_storage_count -gt 0 ]]; then
        local safe_percentage=$((safe_storage_count * 100 / total_storage_count))
        if [[ $safe_percentage -ge 80 ]]; then
            print_check "storage_access_safety" "PASS" "${safe_percentage}% safe storage access"
        else
            print_check "storage_access_safety" "WARN" "Only ${safe_percentage}% safe storage access"
        fi
    else
        print_check "storage_access_safety" "WARN" "Could not analyze storage patterns"
    fi
}

# @title run_full_audit
# @notice Runs all compliance checks
# @dev   Main entry point for full audit
run_full_audit() {
    print_header "SECURITY COMPLIANCE AUTOMATION"
    echo "  Version: $VERSION"
    echo "  Timestamp: $(date -Iseconds)"
    echo "  Working Directory: $(pwd)"
    echo ""
    
    # Run all check categories
    check_git_status
    check_code_formatting
    check_test_coverage
    check_security_patterns
    check_access_control
    check_input_validation
    check_event_emission
    check_arithmetic_safety
    check_gas_complexity
    check_dependency_security
    check_documentation_compliance
    check_storage_integrity
    
    # Print final summary
    print_header "AUDIT COMPLETE"
    print_summary
    
    return $EXIT_CODE
}

# @title run_targeted_check
# @notice Runs a specific check category
# @param  $1 Check category name
run_targeted_check() {
    local check_category="$1"
    
    print_header "TARGETED CHECK: $check_category"
    echo "  Timestamp: $(date -Iseconds)"
    echo ""
    
    case "$check_category" in
        access_control)
            check_access_control
            ;;
        input_validation)
            check_input_validation
            ;;
        event_emission)
            check_event_emission
            ;;
        arithmetic)
            check_arithmetic_safety
            ;;
        security_patterns)
            check_security_patterns
            ;;
        documentation)
            check_documentation_compliance
            ;;
        storage)
            check_storage_integrity
            ;;
        coverage)
            check_test_coverage
            ;;
        *)
            print_error "Unknown check category: $check_category"
            echo "Available categories:"
            echo "  - access_control"
            echo "  - input_validation"
            echo "  - event_emission"
            echo "  - arithmetic"
            echo "  - security_patterns"
            echo "  - documentation"
            echo "  - storage"
            echo "  - coverage"
            EXIT_CODE=1
            ;;
    esac
    
    print_summary
    return $EXIT_CODE
}

# @title generate_json_report
# @notice Generates JSON format report
# @dev   Outputs machine-readable compliance report
generate_json_report() {
    local report
    report=$(cat <<EOF
{
  "version": "$VERSION",
  "timestamp": "$(date -Iseconds)",
  "summary": {
    "total_checks": $TOTAL_CHECKS,
    "passed": $PASSED_CHECKS,
    "failed": $FAILED_CHECKS,
    "warnings": $WARNINGS,
    "all_passed": $((EXIT_CODE == 0 ? 'true' : 'false'))
  },
  "failed_checks": $(printf '%s\n' "${FAILED_CHECKS_LIST[@]}" | jq -R . | jq -s .),
  "warnings": $(printf '%s\n' "${WARNINGS_LIST[@]}" | jq -R . | jq -s .),
  "coverage_percent": $COVERAGE,
  "minimum_coverage_required": $MIN_COVERAGE_PERCENT
}
EOF
)
    echo "$report"
}

# @title show_usage
# @notice Display usage information
show_usage() {
    cat <<EOF
$SCRIPT_NAME v$VERSION - Security Compliance Automation for CI/CD

USAGE:
    $SCRIPT_NAME [OPTIONS]
    $SCRIPT_NAME --check-only <category>
    $SCRIPT_NAME --full-audit

OPTIONS:
    --full-audit         Run all compliance checks (default)
    --check-only <cat>   Run only the specified check category
    --json               Output results in JSON format
    --verbose            Enable verbose output
    --dry-run            Show what would be checked without running
    --help, -h           Show this help message
    --version, -v        Show version information

CHECK CATEGORIES:
    access_control       Verify authorization patterns
    input_validation     Check input validation patterns
    event_emission       Verify audit event emission
    arithmetic           Check arithmetic safety
    security_patterns    Scan for security vulnerabilities
    documentation        Validate documentation standards
    storage              Check storage integrity
    coverage             Verify test coverage

EXAMPLES:
    # Run full security audit
    $SCRIPT_NAME --full-audit

    # Check only access control patterns
    $SCRIPT_NAME --check-only access_control

    # Generate JSON report for CI/CD integration
    $SCRIPT_NAME --json > compliance-report.json

    # Verbose output for debugging
    $SCRIPT_NAME --verbose --full-audit

EXIT CODES:
    0   All checks passed
    1   One or more checks failed
    2   Invalid arguments

EOF
}

# ── Main Entry Point ───────────────────────────────────────────────────────────

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --full-audit)
                FULL_AUDIT=true
                shift
                ;;
            --check-only)
                CHECK_ONLY="$2"
                shift 2
                ;;
            --json)
                JSON_OUTPUT=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            --version)
                echo "$SCRIPT_NAME v$VERSION"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 2
                ;;
        esac
    done
    
    # Set project root
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    cd "$PROJECT_ROOT"
    
    # Dry run mode
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Dry run - would execute the following checks:"
        echo "  - Git repository status"
        echo "  - Code formatting"
        echo "  - Test coverage"
        echo "  - Security patterns"
        echo "  - Access control"
        echo "  - Input validation"
        echo "  - Event emission"
        echo "  - Arithmetic safety"
        echo "  - Gas & complexity"
        echo "  - Dependency security"
        echo "  - Documentation compliance"
        echo "  - Storage integrity"
        exit 0
    fi
    
    # Run appropriate checks
    if [[ -n "$CHECK_ONLY" ]]; then
        run_targeted_check "$CHECK_ONLY"
    else
        run_full_audit
    fi
    
    # Output JSON if requested
    if [[ "$JSON_OUTPUT" == "true" ]]; then
        generate_json_report
    fi
    
    exit $EXIT_CODE
}

# Execute main function
main "$@"
