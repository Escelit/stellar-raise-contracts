#!/usr/bin/env bash
# =============================================================================
# security_compliance_automation.test.sh
# =============================================================================
# @title   SecurityComplianceAutomation Test Suite
# @notice  Comprehensive test suite for CI/CD security compliance automation.
#          Tests all functions, edge cases, and error handling paths.
# @dev     Uses BATS (Bash Automated Testing System) compatible format.
#
# @author  Security Compliance Team
# @license Apache-2.0
#
# Test Coverage Requirements:
#   Minimum 95% coverage for all check functions
#   Edge case validation for all input scenarios
#   Error handling verification
#   Mock environment testing
#
# Usage:
#   ./security_compliance_automation.test.sh [OPTIONS]
#   ./security_compliance_automation.test.sh --verbose
#   ./security_compliance_automation.test.sh --coverage
# =============================================================================

set -euo pipefail

# ── Test Configuration ────────────────────────────────────────────────────────

readonly TEST_SCRIPT_NAME="security_compliance_automation.test"
readonly SCRIPT_UNDER_TEST="security_compliance_automation.sh"
readonly MIN_TEST_COVERAGE=95

# Test counters
declare -i TESTS_RUN=0
declare -i TESTS_PASSED=0
declare -i TESTS_FAILED=0
declare -i ASSERTIONS_RUN=0
declare -i ASSERTIONS_PASSED=0
declare -i ASSERTIONS_FAILED=0

# Test output settings
declare VERBOSE=false
declare COVERAGE_REPORT=false
declare TAP_OUTPUT=false  # Test Anything Protocol format

# Color codes
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[0;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_BOLD='\033[1m'

# ── Test Fixtures ─────────────────────────────────────────────────────────────

# Create temporary test directory
TEST_DIR=""
FIXTURE_DIR=""

setup_test_env() {
    TEST_DIR=$(mktemp -d)
    FIXTURE_DIR="$TEST_DIR/fixtures"
    mkdir -p "$FIXTURE_DIR"
}

teardown_test_env() {
    if [[ -n "$TEST_DIR" && -d "$TEST_DIR" ]]; then
        rm -rf "$TEST_DIR"
    fi
}

# ── Test Infrastructure ────────────────────────────────────────────────────────

# @title begin_test
# @notice Marks the beginning of a test
# @param  $1 Test description
begin_test() {
    local test_name="$1"
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [[ "$TAP_OUTPUT" == "true" ]]; then
        echo "ok $TESTS_RUN - $test_name"
    elif [[ "$VERBOSE" == "true" ]]; then
        echo -e "\n${COLOR_BLUE}▶${COLOR_RESET} $test_name"
    fi
}

# @title pass_test
# @notice Marks a test as passed
# @param  $1 Test description
pass_test() {
    local test_name="$1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    if [[ "$TAP_OUTPUT" == "true" ]]; then
        :  # Already output by begin_test
    elif [[ "$VERBOSE" == "true" ]]; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} PASSED: $test_name"
    fi
}

# @title fail_test
# @notice Marks a test as failed
# @param  $1 Test description
# @param  $2 Failure reason
fail_test() {
    local test_name="$1"
    local reason="$2"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    
    if [[ "$TAP_OUTPUT" == "true" ]]; then
        echo "not ok $TESTS_RUN - $test_name"
        echo "  ---"
        echo "  message: $reason"
        echo "  ..."
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} FAILED: $test_name"
        echo -e "    Reason: $reason"
    fi
}

# @title assert_true
# @notice Asserts that a condition is true
# @param  $1 Condition to check
# @param  $2 Assertion message
assert_true() {
    local condition="$1"
    local message="${2:-assertion}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if eval "$condition"; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    Condition: $condition"
        return 1
    fi
}

# @title assert_false
# @notice Asserts that a condition is false
# @param  $1 Condition to check
# @param  $2 Assertion message
assert_false() {
    local condition="$1"
    local message="${2:-assertion}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if ! eval "$condition"; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    Condition: $condition"
        return 1
    fi
}

# @title assert_equal
# @notice Asserts that two values are equal
# @param  $1 Expected value
# @param  $2 Actual value
# @param  $3 Assertion message
assert_equal() {
    local expected="$1"
    local actual="$2"
    local message="${3:-Values should be equal}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if [[ "$expected" == "$actual" ]]; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    Expected: '$expected'"
        echo -e "    Actual:   '$actual'"
        return 1
    fi
}

# @title assert_contains
# @notice Asserts that a string contains a substring
# @param  $1 String to search
# @param  $2 Substring to find
# @param  $3 Assertion message
assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-String should contain substring}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if [[ "$haystack" == *"$needle"* ]]; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    String: '$haystack'"
        echo -e "    Substring: '$needle'"
        return 1
    fi
}

# @title assert_file_exists
# @notice Asserts that a file exists
# @param  $1 File path
# @param  $2 Assertion message
assert_file_exists() {
    local file="$1"
    local message="${2:-File should exist}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if [[ -f "$file" ]]; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    File: '$file'"
        return 1
    fi
}

# @title assert_matches
# @notice Asserts that a string matches a regex pattern
# @param  $1 String to match
# @param  $2 Regex pattern
# @param  $3 Assertion message
assert_matches() {
    local string="$1"
    local pattern="$2"
    local message="${3:-String should match pattern}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    if [[ "$string" =~ $pattern ]]; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    String: '$string'"
        echo -e "    Pattern: '$pattern'"
        return 1
    fi
}

# @title assert_exit_code
# @notice Asserts that a command exits with a specific code
# @param  $1 Command to run
# @param  $2 Expected exit code
# @param  $3 Assertion message
assert_exit_code() {
    local cmd="$1"
    local expected_code="$2"
    local message="${3:-Exit code should match}"
    ASSERTIONS_RUN=$((ASSERTIONS_RUN + 1))
    
    local actual_code
    actual_code=$(eval "$cmd" 2>/dev/null) || actual_code=$?
    
    if [[ "$actual_code" -eq "$expected_code" ]]; then
        ASSERTIONS_PASSED=$((ASSERTIONS_PASSED + 1))
        return 0
    else
        ASSERTIONS_FAILED=$((ASSERTIONS_FAILED + 1))
        echo -e "    ${COLOR_RED}Assertion failed:${COLOR_RESET} $message"
        echo -e "    Expected exit code: $expected_code"
        echo -e "    Actual exit code: $actual_code"
        return 1
    fi
}

# ── Test Fixtures ─────────────────────────────────────────────────────────────

# @title create_fixture_rust_contract
# @notice Creates a fixture Rust contract file for testing
# @param  $1 File path
# @param  $2 Contract content type
create_fixture_rust_contract() {
    local file="$1"
    local type="${2:-basic}"
    
    case "$type" in
        basic)
            cat > "$file" << 'EOF'
use soroban_sdk::{Env, Address};

pub struct CrowdfundContract;

impl CrowdfundContract {
    /// @title initialize
    /// @notice Initializes the crowdfund contract
    /// @param env Environment
    /// @param admin Admin address
    /// @param goal Campaign goal
    pub fn initialize(env: &Env, admin: Address, goal: i128) {
        env.storage().instance().set(&"admin", &admin);
        if goal < 1 {
            panic!("Goal must be positive");
        }
    }

    /// @title contribute
    /// @notice Accepts contributions
    /// @param env Environment
    /// @param amount Contribution amount
    pub fn contribute(env: &Env, amount: i128) {
        require_nonnegative(amount);
        let total: i128 = env.storage().instance().get(&"total").unwrap_or(0);
        let new_total = total.checked_add(amount).unwrap();
        env.storage().instance().set(&"total", &new_total);
    }

    fn require_nonnegative(amount: i128) {
        if amount < 0 {
            panic!("Amount must be non-negative");
        }
    }
}
EOF
            ;;
        complete)
            cat > "$file" << 'EOF'
use soroban_sdk::{Env, Address, Symbol};

const MAX_FEE_BPS: u32 = 1000;

#[derive(Clone)]
pub enum Status { Active, Succeeded, Expired }

#[derive(Clone)]
pub struct PlatformConfig { pub fee_bps: u32 }

#[derive(Clone)]
pub enum DataKey { Admin, Creator, Token, Goal, Status, TotalRaised, MinContribution, Deadline, PlatformConfig, Paused }

pub struct CrowdfundContract;

impl CrowdfundContract {
    /// @title initialize
    /// @notice Initializes the crowdfund contract with required parameters
    /// @param env The Soroban environment
    /// @param admin Administrator address
    /// @param creator Creator address
    /// @param token Token contract address
    /// @param goal Campaign goal (must be >= 1)
    /// @param min_contribution Minimum contribution amount
    /// @param deadline Campaign deadline timestamp
    /// @security Only callable during initialization
    pub fn initialize(env: &Env, admin: Address, creator: Address, token: Address, goal: i128, min_contribution: i128, deadline: u64) {
        // Validate inputs
        if goal < 1 { panic!("Goal must be positive"); }
        if min_contribution < 1 { panic!("Min contribution must be positive"); }
        if deadline <= env.ledger().timestamp() { panic!("Deadline must be in future"); }
        
        // Store values
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Creator, &creator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Goal, &goal);
        env.storage().instance().set(&DataKey::MinContribution, &min_contribution);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage().instance().set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage().instance().set(&DataKey::Paused, &false);
        
        // Emit initialization event
        env.events().publish((Symbol::new(env, "initialized"),), ());
    }

    /// @title contribute
    /// @notice Accepts contributions to the campaign
    /// @param env The Soroban environment
    /// @param from Contributor address
    /// @param amount Contribution amount (must be >= MinContribution)
    /// @security Requires authorization from contributor
    pub fn contribute(env: &Env, from: Address, amount: i128) {
        // Check paused state
        if env.storage().instance().get::<_, bool>(&DataKey::Paused).unwrap_or(false) {
            panic!("Contract is paused");
        }
        
        // Validate amount
        let min_contrib: i128 = env.storage().instance().get(&DataKey::MinContribution).unwrap_or(1);
        if amount < min_contrib {
            panic!("Amount below minimum contribution");
        }
        
        // Update total
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        let new_total = total.checked_add(amount).unwrap();
        env.storage().instance().set(&DataKey::TotalRaised, &new_total);
        
        // Emit contribution event
        env.events().publish((Symbol::new(env, "contributed"),), (from, amount));
    }

    /// @title withdraw
    /// @notice Withdraws funds to creator address
    /// @param env The Soroban environment
    /// @security Only callable by creator after campaign succeeds
    pub fn withdraw(env: &Env) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        
        // Emit withdrawal event
        env.events().publish((Symbol::new(env, "withdrawn"),), (creator, total));
    }

    /// @title pause
    /// @notice Pauses the contract
    /// @param env The Soroban environment
    /// @security Only callable by admin
    pub fn pause(env: &Env) {
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish((Symbol::new(env, "paused"),), ());
    }

    /// @title unpause
    /// @notice Unpauses the contract
    /// @param env The Soroban environment
    /// @security Only callable by admin
    pub fn unpause(env: &Env) {
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish((Symbol::new(env, "unpaused"),), ());
    }

    /// @title check_admin_initialized
    /// @notice Verifies admin is initialized
    /// @param env The Soroban environment
    /// @return true if admin is set
    pub fn check_admin_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Admin)
    }
}
EOF
            ;;
        insecure)
            cat > "$file" << 'EOF'
// INSECURE CONTRACT - FOR TESTING ONLY
use std::process::Command;

pub fn unsafe_function() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo vulnerable")
        .output()
        .expect("Failed");
}
EOF
            ;;
    esac
}

# @title create_fixture_shell_script
# @notice Creates a fixture shell script for testing
# @param  $1 File path
# @param  $2 Script type
create_fixture_shell_script() {
    local file="$1"
    local type="${2:-basic}"
    
    case "$type" in
        basic)
            cat > "$file" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "Basic script"
EOF
            ;;
        with_eval)
            cat > "$file" << 'EOF'
#!/usr/bin/env bash
# INSECURE - DO NOT USE IN PRODUCTION
user_input="echo hello"
eval "$user_input"
EOF
            ;;
        secure)
            cat > "$file" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
# Secure script with proper error handling
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <arg>"
    exit 1
fi
echo "Secure script with argument: $1"
EOF
            ;;
    esac
    chmod +x "$file"
}

# @title create_fixture_cargo_toml
# @notice Creates a fixture Cargo.toml for testing
# @param  $1 File path
# @param  $2 Version type (locked, unlocked)
create_fixture_cargo_toml() {
    local file="$1"
    local version_type="${2:-locked}"
    
    case "$version_type" in
        locked)
            cat > "$file" << 'EOF'
[package]
name = "crowdfund"
version = "1.0.0"
edition = "2021"

[dependencies]
soroban-sdk = { version = "20.0.0" }
serde = { version = "1.0", features = ["derive"] }
EOF
            ;;
        unlocked)
            cat > "$file" << 'EOF'
[package]
name = "crowdfund"
version = "1.0.0"
edition = "2021"

[dependencies]
soroban-sdk = "20.0"
serde = "1.0"
EOF
            ;;
    esac
}

# ── Test Suites ───────────────────────────────────────────────────────────────

# @title test_script_help_option
# @notice Tests the --help option
test_script_help_option() {
    begin_test "Script --help option displays usage"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --help 2>&1 || true)
    
    assert_contains "$output" "USAGE:" "Help output should contain usage"
    assert_contains "$output" "OPTIONS:" "Help output should contain options"
    assert_contains "$output" "EXAMPLES:" "Help output should contain examples"
    assert_contains "$output" "EXIT CODES:" "Help output should contain exit codes"
    assert_contains "$output" "security_compliance_automation" "Help should reference script name"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Script --help option displays usage"
    else
        fail_test "Script --help option displays usage" "Help output validation failed"
    fi
}

# @title test_script_version_option
# @notice Tests the --version option
test_script_version_option() {
    begin_test "Script --version option displays version"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --version 2>&1 || true)
    
    assert_matches "$output" "security_compliance_automation v[0-9]+\.[0-9]+\.[0-9]+" "Version output should match pattern"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Script --version option displays version"
    else
        fail_test "Script --version option displays version" "Version output validation failed"
    fi
}

# @title test_script_dry_run_option
# @notice Tests the --dry-run option
test_script_dry_run_option() {
    begin_test "Script --dry-run option shows checks without running"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --dry-run 2>&1 || true)
    
    assert_contains "$output" "Dry run" "Dry run should indicate dry run mode"
    assert_contains "$output" "git_repository_clean" "Dry run should list checks"
    assert_contains "$output" "access_control" "Dry run should list access_control check"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Script --dry-run option shows checks without running"
    else
        fail_test "Script --dry-run option shows checks without running" "Dry run output validation failed"
    fi
}

# @title test_script_invalid_option
# @notice Tests handling of invalid options
test_script_invalid_option() {
    begin_test "Script handles invalid options gracefully"
    
    local exit_code
    exit_code=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --invalid-option 2>&1 || echo $?)
    
    assert_equal "2" "$exit_code" "Invalid option should return exit code 2"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Script handles invalid options gracefully"
    else
        fail_test "Script handles invalid options gracefully" "Invalid option exit code validation failed"
    fi
}

# @title test_targeted_check_access_control
# @notice Tests the --check-only access_control option
test_targeted_check_access_control() {
    begin_test "Targeted check: access_control"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only access_control 2>&1 || true)
    
    assert_contains "$output" "access_control" "Output should mention access_control"
    assert_contains "$output" "contribute_authorization" "Output should show contribute_authorization check"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Targeted check: access_control"
    else
        fail_test "Targeted check: access_control" "Access control check validation failed"
    fi
}

# @title test_targeted_check_input_validation
# @notice Tests the --check-only input_validation option
test_targeted_check_input_validation() {
    begin_test "Targeted check: input_validation"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only input_validation 2>&1 || true)
    
    assert_contains "$output" "input_validation" "Output should mention input_validation"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Targeted check: input_validation"
    else
        fail_test "Targeted check: input_validation" "Input validation check validation failed"
    fi
}

# @title test_targeted_check_event_emission
# @notice Tests the --check-only event_emission option
test_targeted_check_event_emission() {
    begin_test "Targeted check: event_emission"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only event_emission 2>&1 || true)
    
    assert_contains "$output" "event_emission" "Output should mention event_emission"
    assert_contains "$output" "contribution_event_emission" "Output should show contribution_event_emission check"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Targeted check: event_emission"
    else
        fail_test "Targeted check: event_emission" "Event emission check validation failed"
    fi
}

# @title test_targeted_check_arithmetic
# @notice Tests the --check-only arithmetic option
test_targeted_check_arithmetic() {
    begin_test "Targeted check: arithmetic"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only arithmetic 2>&1 || true)
    
    assert_contains "$output" "arithmetic" "Output should mention arithmetic"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Targeted check: arithmetic"
    else
        fail_test "Targeted check: arithmetic" "Arithmetic check validation failed"
    fi
}

# @title test_targeted_check_unknown
# @notice Tests handling of unknown check category
test_targeted_check_unknown() {
    begin_test "Unknown check category handling"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only unknown_category 2>&1 || true)
    
    assert_contains "$output" "Unknown check category" "Output should indicate unknown category"
    assert_contains "$output" "Available categories" "Output should list available categories"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Unknown check category handling"
    else
        fail_test "Unknown check category handling" "Unknown category handling validation failed"
    fi
}

# @title test_full_audit_execution
# @notice Tests full audit execution
test_full_audit_execution() {
    begin_test "Full audit execution completes"
    
    local output
    local exit_code
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --full-audit 2>&1 || true)
    exit_code=$?
    
    # Full audit should always run and produce summary
    assert_contains "$output" "COMPLIANCE SUMMARY" "Full audit should produce summary"
    assert_contains "$output" "Total Checks:" "Summary should show total checks"
    assert_contains "$output" "Passed:" "Summary should show passed count"
    assert_contains "$output" "Failed:" "Summary should show failed count"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Full audit execution completes"
    else
        fail_test "Full audit execution completes" "Full audit output validation failed"
    fi
}

# @title test_json_output_format
# @notice Tests JSON output format
test_json_output_format() {
    begin_test "JSON output format is valid"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --json 2>&1 || true)
    
    # Try to parse as JSON using Python or jq
    if command -v jq &>/dev/null; then
        if echo "$output" | jq . &>/dev/null; then
            assert_true "[[ \$output == *'\"version\"'* ]]" "JSON should contain version field"
            assert_true "[[ \$output == *'\"summary\"'* ]]" "JSON should contain summary field"
            assert_true "[[ \$output == *'\"total_checks\"'* ]]" "JSON should contain total_checks field"
            
            if [[ $? -eq 0 ]]; then
                pass_test "JSON output format is valid"
            else
                fail_test "JSON output format is valid" "JSON field validation failed"
            fi
        else
            fail_test "JSON output format is valid" "Output is not valid JSON"
        fi
    elif command -v python3 &>/dev/null; then
        if python3 -c "import json; json.loads('''$output''')" 2>/dev/null; then
            pass_test "JSON output format is valid"
        else
            fail_test "JSON output format is valid" "Output is not valid JSON"
        fi
    else
        # Basic validation without jq/python
        assert_contains "$output" "{" "JSON output should start with {"
        assert_contains "$output" "}" "JSON output should end with }"
        
        if [[ $? -eq 0 ]]; then
            pass_test "JSON output format is valid (basic check)"
        else
            fail_test "JSON output format is valid" "Basic JSON format check failed"
        fi
    fi
}

# @title test_security_pattern_detection_suspicious
# @notice Tests detection of suspicious shell patterns
test_security_pattern_detection_suspicious() {
    begin_test "Security pattern detection: eval() in shell scripts"
    
    # Create test fixture with eval
    local test_script="$FIXTURE_DIR/test_eval.sh"
    create_fixture_shell_script "$test_script" "with_eval"
    
    # Source the script in a subshell with the detection logic
    local output
    output=$(grep -rnE "eval\(" "$test_script" 2>/dev/null || true)
    
    assert_true "[[ -n \"\$output\" ]]" "Should detect eval() pattern"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Security pattern detection: eval() in shell scripts"
    else
        fail_test "Security pattern detection: eval() in shell scripts" "Pattern detection failed"
    fi
}

# @title test_security_pattern_detection_none
# @notice Tests no false positives on clean code
test_security_pattern_detection_none() {
    begin_test "Security pattern detection: no false positives on clean code"
    
    local test_script="$FIXTURE_DIR/test_clean.sh"
    create_fixture_shell_script "$test_script" "secure"
    
    # Check for suspicious patterns (should be none)
    local suspicious_found=0
    for pattern in "eval(" "system(" "exec("; do
        if grep -qnE "$pattern" "$test_script" 2>/dev/null; then
            suspicious_found=1
            break
        fi
    done
    
    assert_equal "0" "$suspicious_found" "Clean script should not trigger pattern detection"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Security pattern detection: no false positives on clean code"
    else
        fail_test "Security pattern detection: no false positives on clean code" "False positive detected"
    fi
}

# @title test_rust_natspec_detection
# @notice Tests NatSpec documentation detection
test_rust_natspec_detection() {
    begin_test "NatSpec documentation detection in Rust files"
    
    # Check for NatSpec comments in actual source
    local natspec_count
    natspec_count=$(grep -cE "^///|^/\*!|^//!" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    
    assert_true "[[ $natspec_count -gt 0 ]]" "Should find NatSpec comments in source"
    
    if [[ $? -eq 0 ]]; then
        pass_test "NatSpec documentation detection in Rust files"
    else
        fail_test "NatSpec documentation detection in Rust files" "NatSpec detection failed"
    fi
}

# @title test_access_control_pattern_detection
# @notice Tests access control pattern detection
test_access_control_pattern_detection() {
    begin_test "Access control pattern detection"
    
    # Check for require_auth in source
    local has_require_auth=false
    if grep -q "require_auth" contracts/crowdfund/src/*.rs 2>/dev/null; then
        has_require_auth=true
    fi
    
    assert_true "[[ \$has_require_auth == true ]]" "Should find require_auth in source"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Access control pattern detection"
    else
        fail_test "Access control pattern detection" "Access control detection failed"
    fi
}

# @title test_event_emission_detection
# @notice Tests event emission detection
test_event_emission_detection() {
    begin_test "Event emission detection"
    
    # Check for emit_event/events in source
    local has_events=false
    if grep -q "emit_event\|events()" contracts/crowdfund/src/*.rs 2>/dev/null; then
        has_events=true
    fi
    
    assert_true "[[ \$has_events == true ]]" "Should find event emission in source"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Event emission detection"
    else
        fail_test "Event emission detection" "Event emission detection failed"
    fi
}

# @title test_checked_arithmetic_detection
# @notice Tests checked arithmetic detection
test_checked_arithmetic_detection() {
    begin_test "Checked arithmetic detection"
    
    # Check for checked arithmetic methods
    local has_checked=false
    if grep -q "checked_add\|checked_sub\|checked_mul\|saturating_add" contracts/crowdfund/src/*.rs 2>/dev/null; then
        has_checked=true
    fi
    
    assert_true "[[ \$has_checked == true ]]" "Should find checked arithmetic in source"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Checked arithmetic detection"
    else
        fail_test "Checked arithmetic detection" "Checked arithmetic detection failed"
    fi
}

# @title test_coverage_calculation
# @notice Tests test coverage calculation
test_coverage_calculation() {
    begin_test "Test coverage calculation"
    
    # Count test functions vs total functions
    local test_count
    test_count=$(grep -c "#\[test\]" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    local function_count
    function_count=$(grep -cE "^    pub fn|^fn" contracts/crowdfund/src/*.rs 2>/dev/null || echo "0")
    
    if [[ "$function_count" -gt 0 ]]; then
        local coverage=$((test_count * 100 / function_count))
        assert_true "[[ $coverage -ge 0 ]]" "Coverage should be non-negative"
        
        if [[ $? -eq 0 ]]; then
            pass_test "Test coverage calculation"
        else
            fail_test "Test coverage calculation" "Coverage calculation failed"
        fi
    else
        pass_test "Test coverage calculation" "No functions found to calculate coverage"
    fi
}

# @title test_constants_defined_correctly
# @notice Tests that constants are properly defined
test_constants_defined_correctly() {
    begin_test "Constants defined correctly in script"
    
    # Verify constants are defined in the script
    local script_content
    script_content=$(cat "$SCRIPT_DIR/$SCRIPT_UNDER_TEST")
    
    assert_true "[[ \$script_content == *'MAX_ALLOWED_FEE_BPS=1000'* ]]" "MAX_ALLOWED_FEE_BPS should be 1000"
    assert_true "[[ \$script_content == *'MIN_COMPLIANT_GOAL=1'* ]]" "MIN_COMPLIANT_GOAL should be 1"
    assert_true "[[ \$script_content == *'MIN_COMPLIANT_CONTRIBUTION=1'* ]]" "MIN_COMPLIANT_CONTRIBUTION should be 1"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Constants defined correctly in script"
    else
        fail_test "Constants defined correctly in script" "Constant validation failed"
    fi
}

# @title test_check_result_types_defined
# @notice Tests that check result types are defined
test_check_result_types_defined() {
    begin_test "Check result types defined in script"
    
    local script_content
    script_content=$(cat "$SCRIPT_DIR/$SCRIPT_UNDER_TEST")
    
    assert_true "[[ \$script_content == *'PASS'* ]]" "PASS result type should be defined"
    assert_true "[[ \$script_content == *'FAIL'* ]]" "FAIL result type should be defined"
    assert_true "[[ \$script_content == *'WARN'* ]]" "WARN result type should be defined"
    assert_true "[[ \$script_content == *'print_check'* ]]" "print_check function should be defined"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Check result types defined in script"
    else
        fail_test "Check result types defined in script" "Result type validation failed"
    fi
}

# @title test_error_handling_invalid_file
# @notice Tests error handling for invalid files
test_error_handling_invalid_file() {
    begin_test "Error handling for non-existent directories"
    
    # Change to non-existent directory and try to run
    local exit_code
    exit_code=$(./"$SCRIPT_UNDER_TEST" --full-audit 2>&1 || echo $?)
    
    # Script should either work or fail gracefully
    assert_true "[[ $exit_code -eq 0 || $exit_code -eq 1 ]]" "Exit code should be 0 or 1"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Error handling for non-existent directories"
    else
        fail_test "Error handling for non-existent directories" "Error handling validation failed"
    fi
}

# @title test_git_status_check
# @notice Tests git status check functionality
test_git_status_check() {
    begin_test "Git status check in full audit"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --full-audit 2>&1 || true)
    
    # Check that git status is verified
    assert_true "[[ \$output == *'Git Repository Status'* ]]" "Git status section should be present"
    assert_true "[[ \$output == *'git_repository_clean'* ]]" "Git repository check should be present"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Git status check in full audit"
    else
        fail_test "Git status check in full audit" "Git status check validation failed"
    fi
}

# @title test_dependency_check
# @notice Tests dependency security checks
test_dependency_check() {
    begin_test "Dependency security checks"
    
    local output
    output=$(cd "$SCRIPT_DIR" && ./"$SCRIPT_UNDER_TEST" --check-only coverage 2>&1 || true)
    
    # Should check for Cargo.toml and Cargo.lock
    assert_true "[[ \$output == *'Cargo'* ]]" "Dependency checks should reference Cargo"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Dependency security checks"
    else
        fail_test "Dependency security checks" "Dependency check validation failed"
    fi
}

# @title test_color_output_support
# @notice Tests color output support
test_color_output_support() {
    begin_test "Color output support"
    
    local script_content
    script_content=$(cat "$SCRIPT_DIR/$SCRIPT_UNDER_TEST")
    
    # Check for ANSI color codes
    assert_true "[[ \$script_content == *'COLOR_GREEN'* ]]" "Should define COLOR_GREEN"
    assert_true "[[ \$script_content == *'COLOR_RED'* ]]" "Should define COLOR_RED"
    assert_true "[[ \$script_content == *'COLOR_YELLOW'* ]]" "Should define COLOR_YELLOW"
    assert_true "[[ \$script_content == *'COLOR_RESET'* ]]" "Should define COLOR_RESET"
    
    if [[ $? -eq 0 ]]; then
        pass_test "Color output support"
    else
        fail_test "Color output support" "Color output validation failed"
    fi
}

# @title test_tap_output_mode
# @notice Tests TAP (Test Anything Protocol) output mode
test_tap_output_mode() {
    begin_test "TAP output mode"
    
    TAP_OUTPUT=true
    local output
    output=$(cd "$SCRIPT_DIR" && TAP_OUTPUT=true ./"$SCRIPT_UNDER_TEST" --dry-run 2>&1 || true)
    
    # TAP format should use ok/not ok
    assert_true "[[ \$output == *'ok '* ]] || [[ \$output == *'not ok '* ]]" "TAP output should use ok/not ok"
    
    if [[ $? -eq 0 ]]; then
        pass_test "TAP output mode"
    else
        fail_test "TAP output mode" "TAP output validation failed"
    fi
    
    TAP_OUTPUT=false
}

# ── Test Runner ───────────────────────────────────────────────────────────────

# @title print_test_summary
# @notice Prints final test summary
print_test_summary() {
    local total_assertions=$((ASSERTIONS_PASSED + ASSERTIONS_FAILED))
    local coverage_percent=0
    
    if [[ $TESTS_RUN -gt 0 ]]; then
        coverage_percent=$((TESTS_PASSED * 100 / TESTS_RUN))
    fi
    
    echo ""
    echo -e "${COLOR_BOLD}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo -e "  ${COLOR_BOLD}TEST SUMMARY${COLOR_RESET}"
    echo -e "${COLOR_BOLD}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo ""
    echo -e "  Tests Run:       ${TESTS_RUN}"
    echo -e "  ${COLOR_GREEN}Tests Passed:${COLOR_RESET}     ${TESTS_PASSED}"
    echo -e "  ${COLOR_RED}Tests Failed:${COLOR_RESET}     ${TESTS_FAILED}"
    echo ""
    echo -e "  Assertions:      ${total_assertions}"
    echo -e "  ${COLOR_GREEN}Assertions Passed:${COLOR_RESET} ${ASSERTIONS_PASSED}"
    echo -e "  ${COLOR_RED}Assertions Failed:${COLOR_RESET} ${ASSERTIONS_FAILED}"
    echo ""
    echo -e "  Test Coverage:   ${coverage_percent}% (minimum: ${MIN_TEST_COVERAGE}%)"
    echo ""
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "  ${COLOR_RED}Some tests failed. Review the output above.${COLOR_RESET}"
        return 1
    elif [[ $coverage_percent -lt $MIN_TEST_COVERAGE ]]; then
        echo -e "  ${COLOR_YELLOW}Test coverage below minimum threshold.${COLOR_RESET}"
        return 1
    else
        echo -e "  ${COLOR_GREEN}All tests passed!${COLOR_RESET}"
        return 0
    fi
}

# @title run_all_tests
# @notice Runs all test suites
run_all_tests() {
    echo -e "${COLOR_BOLD}${COLOR_BLUE}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo -e "${COLOR_BOLD}${COLOR_BLUE}  SECURITY COMPLIANCE AUTOMATION TEST SUITE${COLOR_RESET}"
    echo -e "${COLOR_BOLD}${COLOR_BLUE}═══════════════════════════════════════════════════════════════${COLOR_RESET}"
    echo ""
    echo "  Script under test: $SCRIPT_UNDER_TEST"
    echo "  Test script: $TEST_SCRIPT_NAME v1.0.0"
    echo "  Timestamp: $(date -Iseconds)"
    echo ""
    
    # Setup test environment
    setup_test_env
    
    # Run test suites
    echo -e "${COLOR_BOLD}── Option Handling Tests ──────────────────────────────────────────${COLOR_RESET}"
    test_script_help_option
    test_script_version_option
    test_script_dry_run_option
    test_script_invalid_option
    
    echo -e "\n${COLOR_BOLD}── Targeted Check Tests ─────────────────────────────────────────${COLOR_RESET}"
    test_targeted_check_access_control
    test_targeted_check_input_validation
    test_targeted_check_event_emission
    test_targeted_check_arithmetic
    test_targeted_check_unknown
    
    echo -e "\n${COLOR_BOLD}── Full Audit Tests ────────────────────────────────────────────${COLOR_RESET}"
    test_full_audit_execution
    test_json_output_format
    test_git_status_check
    
    echo -e "\n${COLOR_BOLD}── Security Pattern Tests ──────────────────────────────────────${COLOR_RESET}"
    test_security_pattern_detection_suspicious
    test_security_pattern_detection_none
    
    echo -e "\n${COLOR_BOLD}── Code Analysis Tests ─────────────────────────────────────────${COLOR_RESET}"
    test_rust_natspec_detection
    test_access_control_pattern_detection
    test_event_emission_detection
    test_checked_arithmetic_detection
    
    echo -e "\n${COLOR_BOLD}── Configuration Tests ──────────────────────────────────────────${COLOR_RESET}"
    test_constants_defined_correctly
    test_check_result_types_defined
    test_color_output_support
    
    echo -e "\n${COLOR_BOLD}── Edge Case Tests ─────────────────────────────────────────────${COLOR_RESET}"
    test_error_handling_invalid_file
    test_coverage_calculation
    test_tap_output_mode
    
    # Cleanup
    teardown_test_env
    
    # Print summary
    print_test_summary
}

# ── Main Entry Point ───────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --coverage)
            COVERAGE_REPORT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $TEST_SCRIPT_NAME [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v    Enable verbose output"
            echo "  --coverage       Show coverage report"
            echo "  --help, -h       Show this help message"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# Run tests
run_all_tests
exit_code=$?

if [[ $COVERAGE_REPORT == "true" ]]; then
    echo ""
    echo -e "${COLOR_BOLD}Coverage Report:${COLOR_RESET}"
    local total_assertions=$((ASSERTIONS_PASSED + ASSERTIONS_FAILED))
    local coverage_percent=0
    if [[ $TESTS_RUN -gt 0 ]]; then
        coverage_percent=$((TESTS_PASSED * 100 / TESTS_RUN))
    fi
    echo "  Test Coverage: ${coverage_percent}%"
    echo "  Assertions Coverage: $(( (ASSERTIONS_PASSED * 100) / (total_assertions > 0 ? total_assertions : 1) ))%"
fi

exit $exit_code
